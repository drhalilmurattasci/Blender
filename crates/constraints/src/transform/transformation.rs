//! Transformation constraint: maps input ranges on one channel to output ranges on another.

use crate::common::{ConstraintBase, ConstraintContext, ConstraintTarget};
use serde::{Deserialize, Serialize};

/// Which transform channel to read/write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapChannel {
    LocationX,
    LocationY,
    LocationZ,
    RotationX,
    RotationY,
    RotationZ,
    ScaleX,
    ScaleY,
    ScaleZ,
}

/// Transformation constraint: maps source ranges to destination ranges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    pub base: ConstraintBase,
    pub target: ConstraintTarget,
    /// Source channel to read from the target.
    pub source_channel: MapChannel,
    /// Destination channel to write on the owner.
    pub dest_channel: MapChannel,
    /// Source range (min, max).
    pub source_min: f32,
    pub source_max: f32,
    /// Destination range (min, max).
    pub dest_min: f32,
    pub dest_max: f32,
    /// Whether to extrapolate outside the source range.
    pub extrapolate: bool,
}

impl Transformation {
    pub fn new(name: impl Into<String>, target: ConstraintTarget) -> Self {
        Self {
            base: ConstraintBase::new(name),
            target,
            source_channel: MapChannel::LocationX,
            dest_channel: MapChannel::LocationX,
            source_min: 0.0,
            source_max: 1.0,
            dest_min: 0.0,
            dest_max: 1.0,
            extrapolate: false,
        }
    }

    pub fn evaluate(&self, ctx: &mut ConstraintContext) {
        if !self.base.is_active() {
            return;
        }

        let Some(target) = &ctx.target_transform else {
            return;
        };

        let source_val = read_channel(target, self.source_channel);
        let influence = self.base.effective_influence();

        // Map source value through range.
        let range = self.source_max - self.source_min;
        let t = if range.abs() < f32::EPSILON {
            0.0
        } else {
            let raw = (source_val - self.source_min) / range;
            if self.extrapolate { raw } else { raw.clamp(0.0, 1.0) }
        };

        let mapped = self.dest_min + t * (self.dest_max - self.dest_min);
        let current = read_channel_owner(&ctx.owner_transform, self.dest_channel);
        let blended = current * (1.0 - influence) + mapped * influence;

        write_channel_owner(&mut ctx.owner_transform, self.dest_channel, blended);
    }
}

fn read_channel(t: &crate::common::context::Transform, ch: MapChannel) -> f32 {
    match ch {
        MapChannel::LocationX => t.location[0],
        MapChannel::LocationY => t.location[1],
        MapChannel::LocationZ => t.location[2],
        MapChannel::RotationX => t.rotation[0],
        MapChannel::RotationY => t.rotation[1],
        MapChannel::RotationZ => t.rotation[2],
        MapChannel::ScaleX => t.scale[0],
        MapChannel::ScaleY => t.scale[1],
        MapChannel::ScaleZ => t.scale[2],
    }
}

fn read_channel_owner(t: &crate::common::context::Transform, ch: MapChannel) -> f32 {
    read_channel(t, ch)
}

fn write_channel_owner(t: &mut crate::common::context::Transform, ch: MapChannel, val: f32) {
    match ch {
        MapChannel::LocationX => t.location[0] = val,
        MapChannel::LocationY => t.location[1] = val,
        MapChannel::LocationZ => t.location[2] = val,
        MapChannel::RotationX => t.rotation[0] = val,
        MapChannel::RotationY => t.rotation[1] = val,
        MapChannel::RotationZ => t.rotation[2] = val,
        MapChannel::ScaleX => t.scale[0] = val,
        MapChannel::ScaleY => t.scale[1] = val,
        MapChannel::ScaleZ => t.scale[2] = val,
    }
}
