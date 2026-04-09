//! Limit Rotation constraint.
//!
//! Matches Blender's `rotlimit_evaluate`: decomposes rotation to euler,
//! clamps per-axis, converts back, and blends with influence.

use crate::common::{ConstraintBase, ConstraintContext};
use serde::{Deserialize, Serialize};

/// Limit Rotation constraint: clamps the owner's euler rotation angles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitRotation {
    pub base: ConstraintBase,
    pub use_limit_x: bool,
    pub use_limit_y: bool,
    pub use_limit_z: bool,
    /// Min rotation per axis (radians).
    pub min: [f32; 3],
    /// Max rotation per axis (radians).
    pub max: [f32; 3],
}

impl LimitRotation {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: ConstraintBase::new(name),
            use_limit_x: false,
            use_limit_y: false,
            use_limit_z: false,
            min: [-std::f32::consts::PI; 3],
            max: [std::f32::consts::PI; 3],
        }
    }

    /// Evaluate the constraint.
    ///
    /// Matches Blender's `rotlimit_evaluate`:
    /// 1. Decompose rotation quaternion to euler angles (XYZ).
    /// 2. Clamp each enabled axis to [min, max].
    /// 3. Convert back to quaternion.
    /// 4. Blend with original using influence.
    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        if !self.use_limit_x && !self.use_limit_y && !self.use_limit_z {
            return;
        }

        let influence = self.base.effective_influence();
        let original_quat = ctx.owner_transform.rotation;

        // Decompose to euler XYZ.
        let mut eul = quat_to_euler_xyz(original_quat);

        // Clamp each enabled axis.
        if self.use_limit_x {
            eul[0] = eul[0].clamp(self.min[0], self.max[0]);
        }
        if self.use_limit_y {
            eul[1] = eul[1].clamp(self.min[1], self.max[1]);
        }
        if self.use_limit_z {
            eul[2] = eul[2].clamp(self.min[2], self.max[2]);
        }

        // Convert back to quaternion.
        let clamped_quat = euler_xyz_to_quat(eul);

        // Blend with influence.
        ctx.owner_transform.rotation = nlerp_quat(original_quat, clamped_quat, influence);
    }
}

/// Convert quaternion [x, y, z, w] to euler XYZ angles.
fn quat_to_euler_xyz(q: [f32; 4]) -> [f32; 3] {
    let [x, y, z, w] = q;

    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xy = x * y;
    let xz = x * z;
    let yz = y * z;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;

    let m02 = 2.0 * (xz + wy);
    let m12 = 2.0 * (yz - wx);
    let m22 = 1.0 - 2.0 * (xx + yy);
    let m00 = 1.0 - 2.0 * (yy + zz);
    let m01 = 2.0 * (xy - wz);
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
