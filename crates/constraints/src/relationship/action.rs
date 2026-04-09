//! Action constraint: uses an action's FCurve evaluation to drive a transform.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use crate::transform::transformation::MapChannel;
use serde::{Deserialize, Serialize};

/// Action constraint: plays a portion of an animation action based on a target property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConstraint {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    /// Name of the action to evaluate.
    pub action_name: String,
    /// Source channel that drives the action evaluation.
    pub source_channel: MapChannel,
    /// Minimum value of the source channel (maps to action start frame).
    pub source_min: f32,
    /// Maximum value of the source channel (maps to action end frame).
    pub source_max: f32,
    /// Start frame of the action range.
    pub action_start: f32,
    /// End frame of the action range.
    pub action_end: f32,
}

impl ActionConstraint {
    pub fn new(name: impl Into<String>, target: ConstraintTarget, action_name: impl Into<String>) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            action_name: action_name.into(),
            source_channel: MapChannel::LocationX,
            source_min: 0.0,
            source_max: 1.0,
            action_start: 0.0,
            action_end: 100.0,
        }
    }

    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        // The full implementation would look up the action, evaluate its F-Curves
        // at the mapped frame, and apply the result. Here we store the proper
        // data structure for the pipeline to use.
        let _ = ctx;
    }

    /// Compute the action frame for a given source value.
    pub fn map_to_frame(&self, source_value: f32) -> f32 {
        let range = self.source_max - self.source_min;
        if range.abs() < f32::EPSILON {
            return self.action_start;
        }
        let t = ((source_value - self.source_min) / range).clamp(0.0, 1.0);
        self.action_start + t * (self.action_end - self.action_start)
    }
}
