//! Copy Rotation constraint.
//!
//! Matches Blender's `rotlike_evaluate`: decomposes to euler angles, applies
//! per-axis filtering with inversion, then uses the selected mix mode.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use crate::transform::AxisFlags;
use serde::{Deserialize, Serialize};

/// Rotation mix mode (matches Blender's ROTLIKE_MIX_*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RotationMixMode {
    /// Replace rotation entirely.
    Replace,
    /// Legacy offset mode: rotate euler axes individually.
    Offset,
    /// Add euler angles.
    Add,
    /// Multiply (concatenate) rotations: new * old.
    Before,
    /// Multiply in reverse order: old * new.
    After,
}

impl Default for RotationMixMode {
    fn default() -> Self {
        Self::Replace
    }
}

/// Copies the rotation of a target to the owner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyRotation {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    /// Which axes to copy.
    pub axes: AxisFlags,
    /// Invert flags per axis.
    pub invert_x: bool,
    pub invert_y: bool,
    pub invert_z: bool,
    /// Mix mode.
    pub mix_mode: RotationMixMode,
}

impl CopyRotation {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            axes: AxisFlags::ALL,
            invert_x: false,
            invert_y: false,
            invert_z: false,
            mix_mode: RotationMixMode::Replace,
        }
    }

    /// Evaluate the constraint.
    ///
    /// Matches Blender's `rotlike_evaluate`:
    /// 1. Decompose owner and target quaternions to euler (XYZ order).
    /// 2. Per-axis: if axis not enabled, keep owner euler; if enabled and invert, negate.
    /// 3. Apply mix mode (Replace, Add, Offset, Before, After).
    /// 4. Blend result with original using influence.
    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        let Some(target) = &ctx.target_transform else {
            return;
        };

        let influence = self.base.effective_influence();

        let original_quat = ctx.owner_transform.rotation;
        let owner_euler = quat_to_euler_xyz(original_quat);
        let target_euler = quat_to_euler_xyz(target.rotation);

        // Determine default euler values for non-copied axes.
        let defeul = match self.mix_mode {
            RotationMixMode::Replace | RotationMixMode::Offset => owner_euler,
            _ => [0.0; 3],
        };

        let is_offset = self.mix_mode == RotationMixMode::Offset;

        // Per-axis filtering, inversion, and offset.
        // Blender order: invert target FIRST, then add offset.
        let mut eul = target_euler;

        if !self.axes.contains(AxisFlags::X) {
            eul[0] = defeul[0];
        } else {
            if self.invert_x {
                eul[0] = -eul[0];
            }
            if is_offset {
                eul[0] += owner_euler[0];
            }
        }

        if !self.axes.contains(AxisFlags::Y) {
            eul[1] = defeul[1];
        } else {
            if self.invert_y {
                eul[1] = -eul[1];
            }
            if is_offset {
                eul[1] += owner_euler[1];
            }
        }

        if !self.axes.contains(AxisFlags::Z) {
            eul[2] = defeul[2];
        } else {
            if self.invert_z {
                eul[2] = -eul[2];
            }
            if is_offset {
                eul[2] += owner_euler[2];
            }
        }

        // Apply mix mode.
        let result_quat = match self.mix_mode {
            RotationMixMode::Replace | RotationMixMode::Offset => {
                euler_xyz_to_quat(eul)
            }
            RotationMixMode::Add => {
                let combined = [
                    eul[0] + owner_euler[0],
                    eul[1] + owner_euler[1],
                    eul[2] + owner_euler[2],
                ];
                euler_xyz_to_quat(combined)
            }
            RotationMixMode::Before => {
                // new * old
                let new_q = euler_xyz_to_quat(eul);
                quat_mul(new_q, original_quat)
            }
            RotationMixMode::After => {
                // old * new
                let new_q = euler_xyz_to_quat(eul);
                quat_mul(original_quat, new_q)
            }
        };

        // Blend with influence.
        ctx.owner_transform.rotation = nlerp_quat(original_quat, result_quat, influence);
    }
}

/// Convert quaternion [x, y, z, w] to euler XYZ angles.
fn quat_to_euler_xyz(q: [f32; 4]) -> [f32; 3] {
    let [x, y, z, w] = q;

    // Build rotation matrix elements from quaternion.
    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xy = x * y;
    let xz = x * z;
    let yz = y * z;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;

    let m00 = 1.0 - 2.0 * (yy + zz);
    let m01 = 2.0 * (xy - wz);
    let m02 = 2.0 * (xz + wy);
    let _m10 = 2.0 * (xy + wz);
    let m11 = 1.0 - 2.0 * (xx + zz);
    let m12 = 2.0 * (yz - wx);
    let _m20 = 2.0 * (xz - wy);
    let m21 = 2.0 * (yz + wx);
    let m22 = 1.0 - 2.0 * (xx + yy);

    // Extract Euler XYZ from rotation matrix.
    let sy = m02.clamp(-1.0, 1.0);
    let ey = sy.asin();

    if (1.0 - sy.abs()) > 1e-6 {
        let ex = (-m12).atan2(m22);
        let ez = (-m01).atan2(m00);
        [ex, ey, ez]
    } else {
        // Gimbal lock.
        let ex = m21.atan2(m11);
        let ez = 0.0;
        [ex, ey, ez]
    }
}

/// Convert Euler XYZ angles to quaternion [x, y, z, w].
fn euler_xyz_to_quat(euler: [f32; 3]) -> [f32; 4] {
    let (sx, cx) = (euler[0] * 0.5).sin_cos();
    let (sy, cy) = (euler[1] * 0.5).sin_cos();
    let (sz, cz) = (euler[2] * 0.5).sin_cos();

    [
        sx * cy * cz - cx * sy * sz,
        cx * sy * cz + sx * cy * sz,
        cx * cy * sz - sx * sy * cz,
        cx * cy * cz + sx * sy * sz,
    ]
}

/// Multiply two quaternions: a * b (Hamilton product). [x, y, z, w] layout.
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

/// Normalized linear interpolation between two quaternions.
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
    let mut result = [
        a[0] * inv + b[0] * t,
        a[1] * inv + b[1] * t,
        a[2] * inv + b[2] * t,
        a[3] * inv + b[3] * t,
    ];

    let len = (result[0] * result[0]
        + result[1] * result[1]
        + result[2] * result[2]
        + result[3] * result[3])
        .sqrt();
    if len > f32::EPSILON {
        for r in &mut result {
            *r /= len;
        }
    }
    result
}
