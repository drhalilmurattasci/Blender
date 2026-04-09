//! Common constraint infrastructure: targets, influence, evaluation context.

pub mod context;
pub mod target;

pub use context::{ConstraintContext, Transform};
pub use target::ConstraintTarget;

use crate::{ConstraintSpace, InfluenceMode};
use serde::{Deserialize, Serialize};

/// Base properties shared by all constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintBase {
    /// Display name of the constraint.
    pub name: String,
    /// Influence factor in `[0, 1]`.
    pub influence: f32,
    /// Influence blending mode.
    pub influence_mode: InfluenceMode,
    /// Whether the constraint is enabled.
    pub enabled: bool,
    /// Whether the constraint is muted (temporarily disabled).
    pub muted: bool,
    /// Owner space.
    pub owner_space: ConstraintSpace,
    /// Target space.
    pub target_space: ConstraintSpace,
    /// Error state message (empty if no error).
    #[serde(skip)]
    pub error: String,
}

impl Default for ConstraintBase {
    fn default() -> Self {
        Self {
            name: String::new(),
            influence: 1.0,
            influence_mode: InfluenceMode::Linear,
            enabled: true,
            muted: false,
            owner_space: ConstraintSpace::World,
            target_space: ConstraintSpace::World,
            error: String::new(),
        }
    }
}

impl ConstraintBase {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Whether this constraint should be evaluated.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.enabled && !self.muted && self.influence > 0.0
    }

    /// Effective influence clamped to [0, 1].
    #[inline]
    pub fn effective_influence(&self) -> f32 {
        self.influence.clamp(0.0, 1.0)
    }
}
