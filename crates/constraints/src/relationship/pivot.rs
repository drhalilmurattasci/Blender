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

        let _influence = self.base.effective_influence();

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

        // Apply rotation around the pivot point (simplified).
        // Full implementation would decompose the owner rotation and apply it around `pivot`.
        let _ = pivot;
    }
}
