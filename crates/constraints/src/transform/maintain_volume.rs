//! Maintain Volume constraint: preserves volume when one axis is scaled.

use crate::common::{ConstraintBase, ConstraintContext};
use serde::{Deserialize, Serialize};

/// Which axis is the "free" axis that drives the volume correction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FreeAxis {
    X,
    Y,
    Z,
}

impl Default for FreeAxis {
    fn default() -> Self {
        Self::Y
    }
}

/// Maintain Volume constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainVolume {
    pub base: ConstraintBase,
    /// The axis whose scale changes drive the volume correction.
    pub free_axis: FreeAxis,
    /// Reference volume (product of all three scales at rest).
    pub volume: f32,
}

impl MaintainVolume {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: ConstraintBase::new(name),
            free_axis: FreeAxis::Y,
            volume: 1.0,
        }
    }

    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        let influence = self.base.effective_influence();
        let scale = &ctx.owner_transform.scale;
        let free_idx = match self.free_axis {
            FreeAxis::X => 0,
            FreeAxis::Y => 1,
            FreeAxis::Z => 2,
        };

        let free_scale = scale[free_idx];
        if free_scale.abs() < f32::EPSILON {
            return;
        }

        // Blender's samevolume_evaluate: correction = sqrt(volume / free_scale).
        // The other two axes are each set to this correction factor.
        // This maintains: free_scale * correction * correction = volume.
        let correction = (self.volume / free_scale).abs().sqrt();

        let other_axes: [usize; 2] = match self.free_axis {
            FreeAxis::X => [1, 2],
            FreeAxis::Y => [0, 2],
            FreeAxis::Z => [0, 1],
        };

        for &axis in &other_axes {
            let original = ctx.owner_transform.scale[axis];
            // Blender scales the matrix columns by (correction / original_scale),
            // effectively replacing the scale. With influence, blend between
            // the original and the corrected value.
            ctx.owner_transform.scale[axis] =
                original + (correction - original) * influence;
        }
    }
}
