//! Locked Track constraint: aims at the target but locks one axis.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use crate::tracking::{TrackAxis, UpAxis};
use serde::{Deserialize, Serialize};

/// Locked Track constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedTrack {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    pub track_axis: TrackAxis,
    pub lock_axis: UpAxis,
}

impl LockedTrack {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            track_axis: TrackAxis::NegZ,
            lock_axis: UpAxis::Y,
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        let Some(target) = &ctx.target_transform else {
            return;
        };

        let influence = self.base.effective_influence();

        // Direction from owner to target.
        let dx = target.location[0] - ctx.owner_transform.location[0];
        let dy = target.location[1] - ctx.owner_transform.location[1];
        let dz = target.location[2] - ctx.owner_transform.location[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist < f32::EPSILON {
            return;
        }

        let dir = [dx / dist, dy / dist, dz / dist];

        // Get the track axis direction in the owner's current local space.
        let track_local = match self.track_axis {
            TrackAxis::PosX => [1.0, 0.0, 0.0],
            TrackAxis::PosY => [0.0, 1.0, 0.0],
            TrackAxis::PosZ => [0.0, 0.0, 1.0],
            TrackAxis::NegX => [-1.0, 0.0, 0.0],
            TrackAxis::NegY => [0.0, -1.0, 0.0],
            TrackAxis::NegZ => [0.0, 0.0, -1.0],
        };

        // Get the lock axis direction in the owner's current space.
        let lock_local = match self.lock_axis {
            UpAxis::X => [1.0, 0.0, 0.0],
            UpAxis::Y => [0.0, 1.0, 0.0],
            UpAxis::Z => [0.0, 0.0, 1.0],
        };

        // Transform the lock axis into world space using current owner rotation.
        let lock_world = quat_transform_vec(ctx.owner_transform.rotation, lock_local);

        // Project the target direction onto the plane perpendicular to the locked axis.
        let dot_lock = dir[0] * lock_world[0] + dir[1] * lock_world[1] + dir[2] * lock_world[2];
        let projected = [
            dir[0] - dot_lock * lock_world[0],
            dir[1] - dot_lock * lock_world[1],
            dir[2] - dot_lock * lock_world[2],
        ];

        let proj_len = (projected[0] * projected[0]
            + projected[1] * projected[1]
            + projected[2] * projected[2])
            .sqrt();

        if proj_len < f32::EPSILON {
            // Target is along the locked axis; can't determine a track direction.
            return;
        }

        let projected_dir = [
            projected[0] / proj_len,
            projected[1] / proj_len,
            projected[2] / proj_len,
        ];

        // Compute the current track axis direction in world space.
        let track_world = quat_transform_vec(ctx.owner_transform.rotation, track_local);

        // Compute the rotation from the current track direction to the projected target direction.
        let dot_track = track_world[0] * projected_dir[0]
            + track_world[1] * projected_dir[1]
            + track_world[2] * projected_dir[2];

        if dot_track > 0.9999 {
            // Already aligned.
            return;
        }

        let delta_quat = if dot_track < -0.9999 {
            // 180 degree rotation around the lock axis.
            let lw = lock_world;
            [lw[0], lw[1], lw[2], 0.0]
        } else {
            let c = cross(track_world, projected_dir);
            let w = 1.0 + dot_track;
            let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2] + w * w).sqrt();
            [c[0] / len, c[1] / len, c[2] / len, w / len]
        };

        // Blend delta with identity by influence.
        let identity = [0.0_f32, 0.0, 0.0, 1.0];
        let blended = nlerp_quat(identity, delta_quat, influence);

        // Apply: new_rotation = delta * owner_rotation (world space rotation).
        ctx.owner_transform.rotation =
            quat_mul(blended, ctx.owner_transform.rotation);
    }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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
