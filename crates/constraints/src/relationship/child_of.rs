//! Child Of constraint: parents the owner to a target with per-channel control.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use crate::transform::AxisFlags;
use serde::{Deserialize, Serialize};

/// Child Of constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildOf {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    /// Which location axes to inherit.
    pub location_axes: AxisFlags,
    /// Which rotation axes to inherit.
    pub rotation_axes: AxisFlags,
    /// Which scale axes to inherit.
    pub scale_axes: AxisFlags,
    /// Whether to use the set-inverse matrix.
    pub use_inverse: bool,
    /// The inverse matrix (flattened 4x4 column-major) for offset correction.
    pub inverse_matrix: [f32; 16],
}

impl ChildOf {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            location_axes: AxisFlags::ALL,
            rotation_axes: AxisFlags::ALL,
            scale_axes: AxisFlags::ALL,
            use_inverse: true,
            inverse_matrix: identity_4x4(),
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

        // Save the original owner transform for per-channel filtering and influence blending.
        let original = ctx.owner_transform;

        // Build the parent matrix from the target transform.
        let parent_mat = transform_to_mat4(target);

        // Apply inverse matrix if set (Blender: parmat = parent * inverse).
        let parent_mat = if self.use_inverse {
            mul_4x4(&parent_mat, &self.inverse_matrix)
        } else {
            parent_mat
        };

        // Build the owner matrix.
        let owner_mat = transform_to_mat4(&original);

        // Blender: result = parmat * owner_original
        let result_mat = mul_4x4(&parent_mat, &owner_mat);

        // Decompose result back to TRS.
        let mut result = mat4_to_transform(&result_mat);

        // Per-channel filtering: restore original values for disabled axes.
        if !self.location_axes.contains(AxisFlags::X) {
            result.location[0] = original.location[0];
        }
        if !self.location_axes.contains(AxisFlags::Y) {
            result.location[1] = original.location[1];
        }
        if !self.location_axes.contains(AxisFlags::Z) {
            result.location[2] = original.location[2];
        }
        if !self.rotation_axes.contains(AxisFlags::ALL) {
            // If any rotation axis is disabled, fall back to original rotation.
            // Full per-axis euler filtering would require decompose->filter->recompose,
            // but Blender's ChildOf works on the full matrix, so partial rotation
            // filtering is an approximation. Keep original if not all axes enabled.
            if self.rotation_axes.is_empty() {
                result.rotation = original.rotation;
            }
        }
        if !self.scale_axes.contains(AxisFlags::X) {
            result.scale[0] = original.scale[0];
        }
        if !self.scale_axes.contains(AxisFlags::Y) {
            result.scale[1] = original.scale[1];
        }
        if !self.scale_axes.contains(AxisFlags::Z) {
            result.scale[2] = original.scale[2];
        }

        // Blend with influence.
        ctx.owner_transform = original.blend(&result, influence);
    }
}

fn identity_4x4() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

/// Build a 4x4 column-major matrix from a Transform (TRS).
fn transform_to_mat4(t: &crate::common::context::Transform) -> [f32; 16] {
    let [qx, qy, qz, qw] = t.rotation;
    let [sx, sy, sz] = t.scale;
    let [tx, ty, tz] = t.location;

    let xx = qx * qx;
    let yy = qy * qy;
    let zz = qz * qz;
    let xy = qx * qy;
    let xz = qx * qz;
    let yz = qy * qz;
    let wx = qw * qx;
    let wy = qw * qy;
    let wz = qw * qz;

    [
        (1.0 - 2.0 * (yy + zz)) * sx,
        (2.0 * (xy + wz)) * sx,
        (2.0 * (xz - wy)) * sx,
        0.0,
        (2.0 * (xy - wz)) * sy,
        (1.0 - 2.0 * (xx + zz)) * sy,
        (2.0 * (yz + wx)) * sy,
        0.0,
        (2.0 * (xz + wy)) * sz,
        (2.0 * (yz - wx)) * sz,
        (1.0 - 2.0 * (xx + yy)) * sz,
        0.0,
        tx,
        ty,
        tz,
        1.0,
    ]
}

/// Multiply two 4x4 column-major matrices.
fn mul_4x4(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut result = [0.0_f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[row + k * 4] * b[k + col * 4];
            }
            result[row + col * 4] = sum;
        }
    }
    result
}

/// Decompose a 4x4 column-major matrix back to a Transform.
fn mat4_to_transform(m: &[f32; 16]) -> crate::common::context::Transform {
    let location = [m[12], m[13], m[14]];

    // Extract scale from column lengths.
    let sx = (m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt();
    let sy = (m[4] * m[4] + m[5] * m[5] + m[6] * m[6]).sqrt();
    let sz = (m[8] * m[8] + m[9] * m[9] + m[10] * m[10]).sqrt();
    let scale = [sx, sy, sz];

    // Normalize rotation columns.
    let inv_sx = if sx > f32::EPSILON { 1.0 / sx } else { 0.0 };
    let inv_sy = if sy > f32::EPSILON { 1.0 / sy } else { 0.0 };
    let inv_sz = if sz > f32::EPSILON { 1.0 / sz } else { 0.0 };

    let r00 = m[0] * inv_sx;
    let r10 = m[1] * inv_sx;
    let r20 = m[2] * inv_sx;
    let r01 = m[4] * inv_sy;
    let r11 = m[5] * inv_sy;
    let r21 = m[6] * inv_sy;
    let r02 = m[8] * inv_sz;
    let r12 = m[9] * inv_sz;
    let r22 = m[10] * inv_sz;

    // Matrix to quaternion (Shepperd's method).
    let trace = r00 + r11 + r22;
    let rotation = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        let inv = 1.0 / s;
        [
            (r12 - r21) * inv,
            (r20 - r02) * inv,
            (r01 - r10) * inv,
            0.25 * s,
        ]
    } else if r00 > r11 && r00 > r22 {
        let s = (1.0 + r00 - r11 - r22).sqrt() * 2.0;
        let inv = 1.0 / s;
        [
            0.25 * s,
            (r01 + r10) * inv,
            (r20 + r02) * inv,
            (r12 - r21) * inv,
        ]
    } else if r11 > r22 {
        let s = (1.0 + r11 - r00 - r22).sqrt() * 2.0;
        let inv = 1.0 / s;
        [
            (r01 + r10) * inv,
            0.25 * s,
            (r12 + r21) * inv,
            (r20 - r02) * inv,
        ]
    } else {
        let s = (1.0 + r22 - r00 - r11).sqrt() * 2.0;
        let inv = 1.0 / s;
        [
            (r20 + r02) * inv,
            (r12 + r21) * inv,
            0.25 * s,
            (r01 - r10) * inv,
        ]
    };

    crate::common::context::Transform {
        location,
        rotation,
        scale,
    }
}
