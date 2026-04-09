//! Copy Location constraint.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use crate::transform::AxisFlags;
use serde::{Deserialize, Serialize};

/// Copies the location of a target to the owner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyLocation {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    /// Which axes to copy.
    pub axes: AxisFlags,
    /// Whether to invert each axis.
    pub invert_x: bool,
    pub invert_y: bool,
    pub invert_z: bool,
    /// Whether to add to the owner's location (offset mode).
    pub use_offset: bool,
}

impl CopyLocation {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            axes: AxisFlags::ALL,
            invert_x: false,
            invert_y: false,
            invert_z: false,
            use_offset: false,
        }
    }

    /// Evaluate the constraint, modifying the owner's transform in the context.
    ///
    /// Matches Blender's `loclike_evaluate` order of operations:
    /// 1. Save owner location as offset (if offset mode).
    /// 2. Per enabled axis: copy target location, apply inversion, add offset.
    /// 3. Blend result with original using influence.
    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        let Some(target) = &ctx.target_transform else {
            return;
        };

        let influence = self.base.effective_influence();
        let original_loc = ctx.owner_transform.location;

        // Capture offset from original owner location (Blender: copy_v3_v3(offset, cob->matrix[3])).
        let offset = if self.use_offset { original_loc } else { [0.0; 3] };

        let mut new_loc = original_loc;

        // Per-axis: copy target, apply inversion, then add offset.
        // Blender: cob->matrix[3][i] = ct->matrix[3][i]; if INVERT: *= -1; += offset[i];
        if self.axes.contains(AxisFlags::X) {
            let mut val = target.location[0];
            if self.invert_x {
                val = -val;
            }
            new_loc[0] = val + offset[0];
        }
        if self.axes.contains(AxisFlags::Y) {
            let mut val = target.location[1];
            if self.invert_y {
                val = -val;
            }
            new_loc[1] = val + offset[1];
        }
        if self.axes.contains(AxisFlags::Z) {
            let mut val = target.location[2];
            if self.invert_z {
                val = -val;
            }
            new_loc[2] = val + offset[2];
        }

        // Blend with influence (Blender applies this in the constraint framework).
        for i in 0..3 {
            ctx.owner_transform.location[i] =
                original_loc[i] + (new_loc[i] - original_loc[i]) * influence;
        }
    }
}
