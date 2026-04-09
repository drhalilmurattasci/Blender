//! Damped Track constraint: smoothly aims one axis at the target.
//!
//! Matches Blender's `damptrack_evaluate`: computes a minimal swing rotation
//! from the tracking axis to the target direction, then applies it to
//! the owner's existing rotation.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use crate::tracking::TrackAxis;
use serde::{Deserialize, Serialize};

/// Damped Track constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DampedTrack {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    pub track_axis: TrackAxis,
}

impl DampedTrack {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            track_axis: TrackAxis::NegZ,
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

        // Direction from owner to target in world space.
        let dx = target.location[0] - ctx.owner_transform.location[0];
        let dy = target.location[1] - ctx.owner_transform.location[1];
        let dz = target.location[2] - ctx.owner_transform.location[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist < f32::EPSILON {
            return;
        }

        let target_dir = [dx / dist, dy / dist, dz / dist];

        // Transform the target direction into the owner's local space
        // by applying the inverse of the owner's rotation.
        let inv_owner = quat_conjugate(ctx.owner_transform.rotation);
        let local_target = quat_transform_vec(inv_owner, target_dir);

        // Get the tracking axis direction in local space.
        let track_dir = match self.track_axis {
            TrackAxis::PosX => [1.0, 0.0, 0.0],
            TrackAxis::PosY => [0.0, 1.0, 0.0],
            TrackAxis::PosZ => [0.0, 0.0, 1.0],
            TrackAxis::NegX => [-1.0, 0.0, 0.0],
            TrackAxis::NegY => [0.0, -1.0, 0.0],
            TrackAxis::NegZ => [0.0, 0.0, -1.0],
        };

        // Compute the swing rotation from track_dir to local_target.
        let dot = track_dir[0] * local_target[0]
            + track_dir[1] * local_target[1]
            + track_dir[2] * local_target[2];

        let delta_quat = if dot > 0.9999 {
            // Already aligned.
            [0.0, 0.0, 0.0, 1.0]
        } else if dot < -0.9999 {
            // 180 degree rotation: find a perpendicular axis.
            let perp = if track_dir[0].abs() < 0.9 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            let c = cross(track_dir, perp);
            let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            [c[0] / len, c[1] / len, c[2] / len, 0.0]
        } else {
            let c = cross(track_dir, local_target);
            let w = 1.0 + dot;
            let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2] + w * w).sqrt();
            [c[0] / len, c[1] / len, c[2] / len, w / len]
        };

        // Blend the delta rotation with identity using influence (nlerp).
        let identity = [0.0_f32, 0.0, 0.0, 1.0];
        let blended_delta = nlerp_quat(identity, delta_quat, influence);

        // Apply: new_rotation = owner_rotation * blended_delta
        // (rotate the owner by the delta in local space).
        ctx.owner_transform.rotation =
            quat_mul(ctx.owner_transform.rotation, blended_delta);
    }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Conjugate (inverse for unit quaternions) of [x, y, z, w].
fn quat_conjugate(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
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

/// Multiply two quaternions: a * b. [x, y, z, w] layout.
fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
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
