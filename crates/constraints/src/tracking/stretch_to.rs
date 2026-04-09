//! Stretch To constraint: stretches the owner toward the target.
//!
//! Matches Blender's `stretchto_evaluate`: aims one axis at the target
//! and scales along that axis by the ratio of current distance to rest
//! distance, optionally preserving volume on the other two axes.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use crate::tracking::TrackAxis;
use serde::{Deserialize, Serialize};

/// Volume preservation mode for Stretch To.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VolumeMode {
    /// No volume preservation.
    None,
    /// Preserve volume on the X and Z axes (when tracking along Y).
    XZ,
    /// Preserve volume on the X axis only.
    X,
    /// Preserve volume on the Z axis only.
    Z,
}

impl Default for VolumeMode {
    fn default() -> Self {
        Self::XZ
    }
}

/// Stretch To constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StretchTo {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    /// Which axis to aim at the target.
    pub track_axis: TrackAxis,
    /// Rest length (distance at which scale = 1.0).
    pub rest_length: f32,
    /// Volume preservation mode.
    pub volume: VolumeMode,
    /// Volume variation factor (Blender's `bulge`).
    pub bulge: f32,
}

impl StretchTo {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            track_axis: TrackAxis::PosY,
            rest_length: 0.0,
            volume: VolumeMode::XZ,
            bulge: 1.0,
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
        let original = ctx.owner_transform;

        // Direction from owner to target.
        let dx = target.location[0] - ctx.owner_transform.location[0];
        let dy = target.location[1] - ctx.owner_transform.location[1];
        let dz = target.location[2] - ctx.owner_transform.location[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist < f32::EPSILON {
            return;
        }

        // Compute the stretch scale along the tracking axis.
        let rest = if self.rest_length > f32::EPSILON {
            self.rest_length
        } else {
            // Auto rest length from the original bone length would be used;
            // fallback to dist for no-stretch behavior.
            dist
        };

        let stretch_scale = dist / rest;

        // Volume preservation: scale the other axes inversely.
        let volume_scale = if stretch_scale > f32::EPSILON {
            match self.volume {
                VolumeMode::None => 1.0,
                _ => (1.0 / stretch_scale).sqrt().powf(self.bulge),
            }
        } else {
            1.0
        };

        // Determine axis indices.
        let track_idx = match self.track_axis {
            TrackAxis::PosX | TrackAxis::NegX => 0,
            TrackAxis::PosY | TrackAxis::NegY => 1,
            TrackAxis::PosZ | TrackAxis::NegZ => 2,
        };

        // Apply stretch scale along the track axis.
        let mut new_scale = ctx.owner_transform.scale;
        new_scale[track_idx] *= stretch_scale;

        // Apply volume preservation on the appropriate axes.
        match self.volume {
            VolumeMode::None => {}
            VolumeMode::XZ => {
                for i in 0..3 {
                    if i != track_idx {
                        new_scale[i] *= volume_scale;
                    }
                }
            }
            VolumeMode::X => {
                if track_idx != 0 {
                    new_scale[0] *= volume_scale;
                }
            }
            VolumeMode::Z => {
                if track_idx != 2 {
                    new_scale[2] *= volume_scale;
                }
            }
        }

        // Aim the tracking axis at the target (same as DampedTrack).
        let target_dir = [dx / dist, dy / dist, dz / dist];
        let inv_owner = quat_conjugate(ctx.owner_transform.rotation);
        let local_target = quat_transform_vec(inv_owner, target_dir);

        let track_dir = match self.track_axis {
            TrackAxis::PosX => [1.0, 0.0, 0.0],
            TrackAxis::PosY => [0.0, 1.0, 0.0],
            TrackAxis::PosZ => [0.0, 0.0, 1.0],
            TrackAxis::NegX => [-1.0, 0.0, 0.0],
            TrackAxis::NegY => [0.0, -1.0, 0.0],
            TrackAxis::NegZ => [0.0, 0.0, -1.0],
        };

        let dot = track_dir[0] * local_target[0]
            + track_dir[1] * local_target[1]
            + track_dir[2] * local_target[2];

        let delta_quat = if dot > 0.9999 {
            [0.0, 0.0, 0.0, 1.0]
        } else if dot < -0.9999 {
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

        let new_rotation = quat_mul(ctx.owner_transform.rotation, delta_quat);

        // Apply with influence blending.
        ctx.owner_transform.rotation = nlerp_quat(original.rotation, new_rotation, influence);
        for i in 0..3 {
            ctx.owner_transform.scale[i] =
                original.scale[i] + (new_scale[i] - original.scale[i]) * influence;
        }
    }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn quat_conjugate(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

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
