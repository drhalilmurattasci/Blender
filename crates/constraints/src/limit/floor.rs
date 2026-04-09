//! Floor constraint: prevents the owner from going below a reference plane.
//!
//! Matches Blender's `minmax_evaluate`: clamps the owner's position along
//! the selected axis so it stays above (or below) the target's position.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use serde::{Deserialize, Serialize};

/// Which axis and direction defines the floor plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FloorAxis {
    /// Owner must stay above the target on -X.
    NegX,
    /// Owner must stay above the target on -Y.
    NegY,
    /// Owner must stay above the target on -Z.
    NegZ,
    /// Owner must stay above the target on +X.
    PosX,
    /// Owner must stay above the target on +Y.
    PosY,
    /// Owner must stay above the target on +Z.
    PosZ,
}

impl Default for FloorAxis {
    fn default() -> Self {
        Self::NegY
    }
}

/// Floor constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Floor {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    /// The floor axis determines which direction is "up".
    pub floor_axis: FloorAxis,
    /// Offset from the target's position along the floor axis.
    pub offset: f32,
    /// Whether to use rotation from the target to define the floor plane.
    pub use_rotation: bool,
}

impl Floor {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            floor_axis: FloorAxis::NegY,
            offset: 0.0,
            use_rotation: false,
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

        // Determine axis index and direction.
        let (axis, is_neg) = match self.floor_axis {
            FloorAxis::NegX => (0, true),
            FloorAxis::NegY => (1, true),
            FloorAxis::NegZ => (2, true),
            FloorAxis::PosX => (0, false),
            FloorAxis::PosY => (1, false),
            FloorAxis::PosZ => (2, false),
        };

        let floor_level = target.location[axis] + self.offset;
        let owner_val = ctx.owner_transform.location[axis];

        // Check if the owner violates the floor constraint.
        let needs_clamp = if is_neg {
            // Floor below: owner must stay >= floor_level.
            owner_val < floor_level
        } else {
            // Floor above: owner must stay <= floor_level.
            owner_val > floor_level
        };

        if !needs_clamp {
            return;
        }

        // Blend the clamped value with the original using influence.
        ctx.owner_transform.location[axis] =
            owner_val + (floor_level - owner_val) * influence;
    }
}
