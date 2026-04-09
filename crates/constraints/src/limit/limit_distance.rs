//! Limit Distance constraint.
//!
//! Matches Blender's `distlimit_evaluate`:
//! - Inside: keep owner within the distance (clamp when too far).
//! - Outside: keep owner outside the distance (clamp when too close).
//! - OnSurface: keep owner exactly at the distance.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use serde::{Deserialize, Serialize};

/// Clamping mode for the distance limit (matches Blender's eDistLimit_Modes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DistanceClampMode {
    /// Keep the owner inside the sphere: clamp when distance > limit.
    Inside,
    /// Keep the owner outside the sphere: clamp when distance < limit.
    Outside,
    /// Keep the owner on the sphere surface: always clamp to exact distance.
    OnSurface,
}

impl Default for DistanceClampMode {
    fn default() -> Self {
        Self::Inside
    }
}

/// Limit Distance constraint: restricts the owner's distance from a target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitDistance {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    /// Distance limit value.
    pub distance: f32,
    /// Clamping mode.
    pub clamp_mode: DistanceClampMode,
}

impl LimitDistance {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            distance: 1.0,
            clamp_mode: DistanceClampMode::Inside,
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

        let dx = ctx.owner_transform.location[0] - target.location[0];
        let dy = ctx.owner_transform.location[1] - target.location[1];
        let dz = ctx.owner_transform.location[2] - target.location[2];
        let current_dist = (dx * dx + dy * dy + dz * dz).sqrt();

        // Determine whether we need to clamp.
        // Blender semantics:
        //   Inside:    clamp when current_dist > self.distance (push inward)
        //   Outside:   clamp when current_dist < self.distance (push outward)
        //   OnSurface: always clamp to exactly the distance
        let should_clamp = match self.clamp_mode {
            DistanceClampMode::Inside => current_dist > self.distance,
            DistanceClampMode::Outside => current_dist < self.distance,
            DistanceClampMode::OnSurface => {
                (current_dist - self.distance).abs() > f32::EPSILON
            }
        };

        if !should_clamp {
            return;
        }

        if current_dist < f32::EPSILON {
            // Owner is at the same position as target; can't determine direction.
            // For Outside/OnSurface, we'd need to push in some direction but have none.
            return;
        }

        let factor = self.distance / current_dist;
        let new_loc = [
            target.location[0] + dx * factor,
            target.location[1] + dy * factor,
            target.location[2] + dz * factor,
        ];

        // Blend with influence.
        for i in 0..3 {
            ctx.owner_transform.location[i] = ctx.owner_transform.location[i]
                + (new_loc[i] - ctx.owner_transform.location[i]) * influence;
        }
    }
}
