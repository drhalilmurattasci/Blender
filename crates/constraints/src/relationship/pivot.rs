//! Pivot constraint: rotates the owner around a pivot point.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use serde::{Deserialize, Serialize};

/// Pivot rotation range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PivotRotationRange {
    /// Always use the pivot.
    Always,
    /// Only when rotation is negative on the given axis.
    NegativeX,
    NegativeY,
    NegativeZ,
    /// Only when rotation is positive on the given axis.
    PositiveX,
    PositiveY,
    PositiveZ,
}

impl Default for PivotRotationRange {
    fn default() -> Self {
        Self::Always
    }
}

/// Pivot constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pivot {
    pub base: ConstraintBase,
    pub target: Option<ConstraintTarget>,
    /// Offset from the target (or world origin if no target).
    pub offset: [f32; 3],
    /// When to activate the pivot.
    pub rotation_range: PivotRotationRange,
}

impl Pivot {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target: None,
            offset: [0.0; 3],
            rotation_range: PivotRotationRange::Always,
        }
    }

    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        let influence = self.base.effective_influence();

        // Check rotation range condition.
        let euler = quat_to_euler_xyz(ctx.owner_transform.rotation);
        let should_activate = match self.rotation_range {
            PivotRotationRange::Always => true,
            PivotRotationRange::NegativeX => euler[0] < 0.0,
            PivotRotationRange::NegativeY => euler[1] < 0.0,
            PivotRotationRange::NegativeZ => euler[2] < 0.0,
            PivotRotationRange::PositiveX => euler[0] > 0.0,
            PivotRotationRange::PositiveY => euler[1] > 0.0,
            PivotRotationRange::PositiveZ => euler[2] > 0.0,
        };

        if !should_activate {
            return;
        }

        // Compute the pivot point.
        let pivot = if let Some(target) = &ctx.target_transform {
            [
                target.location[0] + self.offset[0],
                target.location[1] + self.offset[1],
                target.location[2] + self.offset[2],
            ]
        } else {
            self.offset
        };

        // Blender's pivotcon_evaluate: rotate the owner's location around the pivot
        // by the owner's rotation (the rotation between rest and current).
        // offset = owner_loc - pivot
        // rotated_offset = owner_rotation * offset
        // new_loc = pivot + rotated_offset
        let offset = [
            ctx.owner_transform.location[0] - pivot[0],
            ctx.owner_transform.location[1] - pivot[1],
            ctx.owner_transform.location[2] - pivot[2],
        ];

        let rotated = quat_rotate(ctx.owner_transform.rotation, offset);
        let new_loc = [
            pivot[0] + rotated[0],
            pivot[1] + rotated[1],
            pivot[2] + rotated[2],
        ];

        // Blend with influence.
        let original_loc = ctx.owner_transform.location;
        for i in 0..3 {
            ctx.owner_transform.location[i] =
                original_loc[i] + (new_loc[i] - original_loc[i]) * influence;
        }
    }
}

/// Convert quaternion [x, y, z, w] to euler XYZ angles.
fn quat_to_euler_xyz(q: [f32; 4]) -> [f32; 3] {
    let [x, y, z, w] = q;
    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xz = x * z;
    let yz = y * z;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;

    let m02 = 2.0 * (xz + wy);
    let m12 = 2.0 * (yz - wx);
    let m22 = 1.0 - 2.0 * (xx + yy);
    let m00 = 1.0 - 2.0 * (yy + zz);
    let m01 = 2.0 * (x * y - wz);
    let m11 = 1.0 - 2.0 * (xx + zz);
    let m21 = 2.0 * (yz + wx);

    let sy = m02.clamp(-1.0, 1.0);
    let ey = sy.asin();

    if (1.0 - sy.abs()) > 1e-6 {
        let ex = (-m12).atan2(m22);
        let ez = (-m01).atan2(m00);
        [ex, ey, ez]
    } else {
        let ex = m21.atan2(m11);
        [ex, ey, 0.0]
    }
}

/// Rotate a vector by a quaternion [x, y, z, w].
fn quat_rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
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
