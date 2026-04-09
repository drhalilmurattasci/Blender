//! Limit Location constraint.

use crate::common::{ConstraintBase, ConstraintContext};
use serde::{Deserialize, Serialize};

/// Limit Location constraint: clamps the owner's location to within bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitLocation {
    pub base: ConstraintBase,
    pub use_min_x: bool,
    pub use_min_y: bool,
    pub use_min_z: bool,
    pub use_max_x: bool,
    pub use_max_y: bool,
    pub use_max_z: bool,
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl LimitLocation {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: ConstraintBase::new(name),
            use_min_x: false,
            use_min_y: false,
            use_min_z: false,
            use_max_x: false,
            use_max_y: false,
            use_max_z: false,
            min: [-1.0; 3],
            max: [1.0; 3],
        }
    }

    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        let influence = self.base.effective_influence();
        let loc = &mut ctx.owner_transform.location;

        if self.use_min_x && loc[0] < self.min[0] {
            loc[0] = loc[0] + (self.min[0] - loc[0]) * influence;
        }
        if self.use_max_x && loc[0] > self.max[0] {
            loc[0] = loc[0] + (self.max[0] - loc[0]) * influence;
        }
        if self.use_min_y && loc[1] < self.min[1] {
            loc[1] = loc[1] + (self.min[1] - loc[1]) * influence;
        }
        if self.use_max_y && loc[1] > self.max[1] {
            loc[1] = loc[1] + (self.max[1] - loc[1]) * influence;
        }
        if self.use_min_z && loc[2] < self.min[2] {
            loc[2] = loc[2] + (self.min[2] - loc[2]) * influence;
        }
        if self.use_max_z && loc[2] > self.max[2] {
            loc[2] = loc[2] + (self.max[2] - loc[2]) * influence;
        }
    }
}
