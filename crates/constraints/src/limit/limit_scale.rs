//! Limit Scale constraint.

use crate::common::{ConstraintBase, ConstraintContext};
use serde::{Deserialize, Serialize};

/// Limit Scale constraint: clamps the owner's scale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitScale {
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

impl LimitScale {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: ConstraintBase::new(name),
            use_min_x: false,
            use_min_y: false,
            use_min_z: false,
            use_max_x: false,
            use_max_y: false,
            use_max_z: false,
            min: [0.0; 3],
            max: [10.0; 3],
        }
    }

    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        let influence = self.base.effective_influence();
        let scale = &mut ctx.owner_transform.scale;

        let limits = [
            (self.use_min_x, self.use_max_x, 0),
            (self.use_min_y, self.use_max_y, 1),
            (self.use_min_z, self.use_max_z, 2),
        ];

        for (use_min, use_max, i) in limits {
            let mut target = scale[i];
            if use_min && target < self.min[i] {
                target = self.min[i];
            }
            if use_max && target > self.max[i] {
                target = self.max[i];
            }
            scale[i] = scale[i] + (target - scale[i]) * influence;
        }
    }
}
