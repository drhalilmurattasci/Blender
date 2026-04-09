//! Track To constraint: aims one axis at the target while keeping another as up.
//!
//! Matches Blender's `trackto_evaluate`: builds a rotation matrix from the
//! direction vector using the selected track and up axis pair via `vectomat()`.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use crate::tracking::{TrackAxis, UpAxis};
use serde::{Deserialize, Serialize};

/// Track To constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackTo {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    /// Which owner axis points toward the target.
    pub track_axis: TrackAxis,
    /// Which owner axis remains as up reference.
    pub up_axis: UpAxis,
    /// Whether to use the target's up direction.
    pub use_target_z: bool,
}

impl TrackTo {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            track_axis: TrackAxis::NegZ,
            up_axis: UpAxis::Y,
            use_target_z: false,
        }
    }

    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        let Some(target) = &ctx.target_transform else {
            return;
        };

        let influence = self.base.effective_influence();

        // Compute direction from target to owner (Blender: sub_v3_v3v3(vec, cob->matrix[3], ct->matrix[3])).
        // Note: Blender uses owner - target (pointing FROM target TO owner).
        let vec = [
            ctx.owner_transform.location[0] - target.location[0],
            ctx.owner_transform.location[1] - target.location[1],
            ctx.owner_transform.location[2] - target.location[2],
        ];

        let dist = (vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2]).sqrt();
        if dist < f32::EPSILON {
            return;
        }

        // Up vector: either target's Z axis or world up based on up_axis.
        let up = if self.use_target_z {
            // Use target's local Z axis. Since we only have decomposed transforms,
            // approximate from the target's rotation quaternion.
            quat_transform_vec(target.rotation, [0.0, 0.0, 1.0])
        } else {
            // World up based on the selected up axis.
            match self.up_axis {
                UpAxis::X => [1.0, 0.0, 0.0],
                UpAxis::Y => [0.0, 1.0, 0.0],
                UpAxis::Z => [0.0, 0.0, 1.0],
            }
        };

        // Build rotation matrix using vectomat algorithm.
        // This creates a 3x3 matrix where the track axis column = normalized direction,
        // and the up axis column is the closest orthogonal vector to the given up hint.
        let rot_mat = vectomat(vec, up, self.track_axis, self.up_axis);

        // Convert rotation matrix to quaternion.
        let result_quat = mat3_to_quat(&rot_mat);

        // Blend with original rotation using influence.
        let original = ctx.owner_transform.rotation;
        ctx.owner_transform.rotation = nlerp_quat(original, result_quat, influence);
    }
}

/// Build a 3x3 rotation matrix from a direction vector and up hint,
/// placing the direction along the specified track axis and the up
/// hint along the specified up axis.
///
/// This is equivalent to Blender's `vectomat()`.
fn vectomat(
    vec: [f32; 3],
    up_hint: [f32; 3],
    track_axis: TrackAxis,
    up_axis: UpAxis,
) -> [[f32; 3]; 3] {
    // Normalize the direction vector.
    let len = (vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2]).sqrt();
    if len < f32::EPSILON {
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }
    let mut n = [vec[0] / len, vec[1] / len, vec[2] / len];

    // For negative track axes, negate the direction.
    let negate = matches!(track_axis, TrackAxis::NegX | TrackAxis::NegY | TrackAxis::NegZ);
    if negate {
        n = [-n[0], -n[1], -n[2]];
    }

    // Determine which column index the track axis occupies.
    let track_col = match track_axis {
        TrackAxis::PosX | TrackAxis::NegX => 0,
        TrackAxis::PosY | TrackAxis::NegY => 1,
        TrackAxis::PosZ | TrackAxis::NegZ => 2,
    };

    let up_col = match up_axis {
        UpAxis::X => 0,
        UpAxis::Y => 1,
        UpAxis::Z => 2,
    };

    // The remaining column index.
    let other_col = 3 - track_col - up_col;

    // Gram-Schmidt: make up perpendicular to track direction.
    let dot_up_n = up_hint[0] * n[0] + up_hint[1] * n[1] + up_hint[2] * n[2];
    let mut u = [
        up_hint[0] - dot_up_n * n[0],
        up_hint[1] - dot_up_n * n[1],
        up_hint[2] - dot_up_n * n[2],
    ];

    let u_len = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    if u_len < f32::EPSILON {
        // Up is parallel to direction; pick arbitrary perpendicular.
        u = perpendicular(n);
    } else {
        u = [u[0] / u_len, u[1] / u_len, u[2] / u_len];
    }

    // Third axis = cross(track, up) or cross(up, track) to maintain right-handedness.
    // We need: cross product of the two known axes to get the third.
    // The order matters for handedness.
    let v = if (track_col + 1) % 3 == up_col {
        // track_col -> up_col is a "positive" step, so other = cross(track, up)
        cross(n, u)
    } else {
        // other = cross(up, track)
        cross(u, n)
    };

    // Assemble matrix columns.
    let mut mat = [[0.0_f32; 3]; 3];
    mat[track_col] = n;
    mat[up_col] = u;
    mat[other_col] = v;
    mat
}

/// Find a perpendicular vector to v (for degenerate up-axis case).
fn perpendicular(v: [f32; 3]) -> [f32; 3] {
    let abs_v = [v[0].abs(), v[1].abs(), v[2].abs()];
    let other = if abs_v[0] <= abs_v[1] && abs_v[0] <= abs_v[2] {
        [1.0, 0.0, 0.0]
    } else if abs_v[1] <= abs_v[2] {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let c = cross(v, other);
    let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    if len > f32::EPSILON {
        [c[0] / len, c[1] / len, c[2] / len]
    } else {
        [0.0, 1.0, 0.0]
    }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Convert a 3x3 rotation matrix (column-major: mat[col][row]) to quaternion [x,y,z,w].
fn mat3_to_quat(m: &[[f32; 3]; 3]) -> [f32; 4] {
    let trace = m[0][0] + m[1][1] + m[2][2];

    if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        let inv = 1.0 / s;
        [
            (m[1][2] - m[2][1]) * inv,
            (m[2][0] - m[0][2]) * inv,
            (m[0][1] - m[1][0]) * inv,
            0.25 * s,
        ]
    } else if m[0][0] > m[1][1] && m[0][0] > m[2][2] {
        let s = (1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt() * 2.0;
        let inv = 1.0 / s;
        [
            0.25 * s,
            (m[0][1] + m[1][0]) * inv,
            (m[2][0] + m[0][2]) * inv,
            (m[1][2] - m[2][1]) * inv,
        ]
    } else if m[1][1] > m[2][2] {
        let s = (1.0 + m[1][1] - m[0][0] - m[2][2]).sqrt() * 2.0;
        let inv = 1.0 / s;
        [
            (m[0][1] + m[1][0]) * inv,
            0.25 * s,
            (m[1][2] + m[2][1]) * inv,
            (m[2][0] - m[0][2]) * inv,
        ]
    } else {
        let s = (1.0 + m[2][2] - m[0][0] - m[1][1]).sqrt() * 2.0;
        let inv = 1.0 / s;
        [
            (m[2][0] + m[0][2]) * inv,
            (m[1][2] + m[2][1]) * inv,
            0.25 * s,
            (m[0][1] - m[1][0]) * inv,
        ]
    }
}

/// Rotate a vector by a quaternion [x, y, z, w].
fn quat_transform_vec(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let [qx, qy, qz, qw] = q;
    let tx = 2.0 * (qy * v[2] - qz * v[1]);
    let ty = 2.0 * (qz * v[0] - qx * v[2]);
    let tz = 2.0 * (qx * v[1] - qy * v[0]);
    [
        v[0] + qw * tx + (qy * tz - qz * ty),
        v[1] + qw * ty + (qz * tx - qx * tz),
        v[2] + qw * tz + (qx * ty - qy * tx),
    ]
}

fn nlerp_quat(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    if t <= 0.0 {
        return a;
    }
    if t >= 1.0 {
        return b;
    }
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let b = if dot < 0.0 { [-b[0], -b[1], -b[2], -b[3]] } else { b };
    let inv = 1.0 - t;
    let mut r = [
        a[0] * inv + b[0] * t,
        a[1] * inv + b[1] * t,
        a[2] * inv + b[2] * t,
        a[3] * inv + b[3] * t,
    ];
    let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2] + r[3] * r[3]).sqrt();
    if len > f32::EPSILON {
        for v in &mut r {
            *v /= len;
        }
    }
    r
}
