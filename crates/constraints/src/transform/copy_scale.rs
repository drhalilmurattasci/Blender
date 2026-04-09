//! Copy Scale constraint.
//!
//! Matches Blender's `sizelike_evaluate`: apply power, then offset mode
//! (multiply or additive), then per-axis replacement via influence.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use crate::transform::AxisFlags;
use serde::{Deserialize, Serialize};

/// Scale mix mode (matches Blender's SIZELIKE_OFFSET + SIZELIKE_MULTIPLY flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScaleMixMode {
    /// Replace owner's scale with target's scale.
    Replace,
    /// Multiply owner's scale by target's scale (offset + multiply).
    Multiply,
    /// Add target's scale to owner's scale (legacy offset, additive: size + obsize - 1).
    Add,
}

impl Default for ScaleMixMode {
    fn default() -> Self {
        Self::Replace
    }
}

/// Copies the scale of a target to the owner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyScale {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    pub axes: AxisFlags,
    /// Offset mode: combine with owner's scale instead of replacing.
    pub use_offset: bool,
    /// Power exponent applied to each target scale axis.
    pub power: f32,
    pub mix_mode: ScaleMixMode,
}

impl CopyScale {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            axes: AxisFlags::ALL,
            use_offset: false,
            power: 1.0,
            mix_mode: ScaleMixMode::Replace,
        }
    }

    /// Evaluate the constraint.
    ///
    /// Matches Blender's `sizelike_evaluate`:
    /// 1. Extract target scale, apply power exponent.
    /// 2. If offset mode: multiply or add with owner scale.
    /// 3. Per enabled axis: replace owner scale column (ratio-based).
    /// 4. Blend with influence.
    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        let Some(target) = &ctx.target_transform else {
            return;
        };

        let influence = self.base.effective_influence();
        let owner_scale = ctx.owner_transform.scale;

        // Step 1: Get target scale and apply power.
        let mut size = [
            target.scale[0].powf(self.power),
            target.scale[1].powf(self.power),
            target.scale[2].powf(self.power),
        ];

        // Step 2: Apply offset mode (Blender: SIZELIKE_OFFSET).
        if self.use_offset {
            match self.mix_mode {
                ScaleMixMode::Multiply => {
                    // Blender: mul_v3_v3(size, obsize) -- multiply target scale by owner scale.
                    size[0] *= owner_scale[0];
                    size[1] *= owner_scale[1];
                    size[2] *= owner_scale[2];
                }
                ScaleMixMode::Add | ScaleMixMode::Replace => {
                    // Blender legacy: add_v3_v3(size, obsize); add_v3_fl(size, -1.0f);
                    // Effectively: size = target_scale + owner_scale - 1.0
                    size[0] = size[0] + owner_scale[0] - 1.0;
                    size[1] = size[1] + owner_scale[1] - 1.0;
                    size[2] = size[2] + owner_scale[2] - 1.0;
                }
            }
        }

        // Step 3: Per-axis replacement with influence blending.
        // Blender does: mul_v3_fl(cob->matrix[i], size[i] / obsize[i])
        // which replaces the scale on that axis. We work with decomposed scale,
        // so we directly blend between owner_scale and the computed size.
        let flags = [(0, AxisFlags::X), (1, AxisFlags::Y), (2, AxisFlags::Z)];

        for (i, flag) in flags {
            if !self.axes.contains(flag) {
                continue;
            }
            // Blend: owner_scale + (target_size - owner_scale) * influence
            ctx.owner_transform.scale[i] =
                owner_scale[i] + (size[i] - owner_scale[i]) * influence;
        }
    }
}
