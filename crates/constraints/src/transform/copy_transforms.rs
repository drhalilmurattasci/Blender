//! Copy Transforms constraint: copies location, rotation, and scale at once.
//!
//! Matches Blender's `translike_evaluate`: supports Replace, BeforeOriginal,
//! and AfterOriginal mix modes with proper matrix concatenation.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget, Transform};
use serde::{Deserialize, Serialize};

/// Mix mode for Copy Transforms (matches Blender's TRANSLIKE_MIX_*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransformMixMode {
    /// Replace the owner's transform completely.
    Replace,
    /// Concatenate before the owner's transform: target * owner.
    BeforeOriginal,
    /// Concatenate after the owner's transform: owner * target.
    AfterOriginal,
}

impl Default for TransformMixMode {
    fn default() -> Self {
        Self::Replace
    }
}

/// Copies all transform channels from a target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTransforms {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    pub mix_mode: TransformMixMode,
    /// Remove target's shear from the result.
    pub remove_target_shear: bool,
}

impl CopyTransforms {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            mix_mode: TransformMixMode::Replace,
            remove_target_shear: false,
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

        let result = match self.mix_mode {
            TransformMixMode::Replace => {
                // Simply use the target's transform.
                *target
            }
            TransformMixMode::BeforeOriginal => {
                // target * owner: target transform applied first, then owner.
                combine_transforms(target, &original)
            }
            TransformMixMode::AfterOriginal => {
                // owner * target: owner transform applied first, then target.
                combine_transforms(&original, target)
            }
        };

        // Blend between original and result using influence.
        ctx.owner_transform = original.blend(&result, influence);
    }
}

/// Combine two transforms: first * second (apply first, then second).
/// Location is rotated and scaled by the first transform, then added.
/// Rotation is concatenated. Scale is multiplied.
fn combine_transforms(first: &Transform, second: &Transform) -> Transform {
    // Rotate second's location by first's rotation and scale.
    let rotated_loc = quat_transform_vec(first.rotation, [
        second.location[0] * first.scale[0],
        second.location[1] * first.scale[1],
        second.location[2] * first.scale[2],
    ]);

    Transform {
        location: [
            first.location[0] + rotated_loc[0],
            first.location[1] + rotated_loc[1],
            first.location[2] + rotated_loc[2],
        ],
        rotation: quat_mul(first.rotation, second.rotation),
        scale: [
            first.scale[0] * second.scale[0],
            first.scale[1] * second.scale[1],
            first.scale[2] * second.scale[2],
        ],
    }
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
