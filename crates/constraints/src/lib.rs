//! # forge3d-constraints
//!
//! Object and bone constraint system for Forge3D.
//!
//! Constraints modify transforms at evaluation time. Categories:
//! - **Transform**: copy/maintain location, rotation, scale
//! - **Tracking**: aim-at, track-to, locked-track
//! - **Relationship**: parent, pivot, child-of
//! - **Limit**: clamp location, rotation, scale, distance
//! - **Common**: shared constraint infrastructure

pub mod common;
pub mod limit;
pub mod relationship;
pub mod tracking;
pub mod transform;

// Re-export key types for convenience.
pub use common::{ConstraintBase, ConstraintContext, ConstraintTarget, Transform};
pub use limit::{DistanceClampMode, Floor, LimitDistance, LimitLocation, LimitRotation, LimitScale};
pub use relationship::{ActionConstraint, ChildOf, Pivot};
pub use tracking::{DampedTrack, LockedTrack, StretchTo, TrackTo, VolumeMode};
pub use transform::{
    AxisFlags, CopyLocation, CopyRotation, CopyScale, CopyTransforms, MaintainVolume,
    Transformation,
};

use thiserror::Error;

/// Errors from constraint evaluation.
#[derive(Debug, Error)]
pub enum ConstraintError {
    #[error("constraint target not found: {0}")]
    TargetNotFound(String),

    #[error("constraint evaluation failed: {0}")]
    EvalFailed(String),

    #[error("invalid constraint configuration: {0}")]
    InvalidConfig(String),
}

pub type ConstraintResult<T> = Result<T, ConstraintError>;

/// Space in which a constraint operates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ConstraintSpace {
    /// World (scene) space.
    World,
    /// Owner's local (parent-relative) space.
    Local,
    /// Owner's pose space (for bones).
    Pose,
    /// Owner's local space with parent orientation.
    LocalWithParent,
    /// Custom space defined by a reference object/bone.
    Custom,
}

impl Default for ConstraintSpace {
    fn default() -> Self {
        Self::World
    }
}

/// Influence blending mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InfluenceMode {
    /// Linear blend between original and constrained.
    Linear,
    /// Multiply.
    Multiply,
}

impl Default for InfluenceMode {
    fn default() -> Self {
        Self::Linear
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::context::{ConstraintContext, Transform};
    use crate::common::target::ConstraintTarget;

    const EPSILON: f32 = 1e-4;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn approx_eq3(a: [f32; 3], b: [f32; 3]) -> bool {
        approx_eq(a[0], b[0]) && approx_eq(a[1], b[1]) && approx_eq(a[2], b[2])
    }

    // ---- CopyLocation ----

    #[test]
    fn copy_location_basic() {
        let target = ConstraintTarget::object("target");
        let c = CopyLocation::new("test", target);
        let owner = Transform { location: [0.0, 0.0, 0.0], ..Transform::identity() };
        let tgt = Transform { location: [3.0, 4.0, 5.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq3(ctx.owner_transform.location, [3.0, 4.0, 5.0]));
    }

    #[test]
    fn copy_location_half_influence() {
        let target = ConstraintTarget::object("target");
        let mut c = CopyLocation::new("test", target);
        c.base.influence = 0.5;
        let owner = Transform { location: [0.0, 0.0, 0.0], ..Transform::identity() };
        let tgt = Transform { location: [4.0, 0.0, 0.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq(ctx.owner_transform.location[0], 2.0));
    }

    #[test]
    fn copy_location_invert_x() {
        let target = ConstraintTarget::object("t");
        let mut c = CopyLocation::new("test", target);
        c.invert_x = true;
        let tgt = Transform { location: [5.0, 3.0, 1.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(Transform::identity(), tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq(ctx.owner_transform.location[0], -5.0));
        assert!(approx_eq(ctx.owner_transform.location[1], 3.0));
    }

    #[test]
    fn copy_location_offset_mode() {
        let target = ConstraintTarget::object("t");
        let mut c = CopyLocation::new("test", target);
        c.use_offset = true;
        let owner = Transform { location: [1.0, 2.0, 3.0], ..Transform::identity() };
        let tgt = Transform { location: [10.0, 20.0, 30.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq3(ctx.owner_transform.location, [11.0, 22.0, 33.0]));
    }

    #[test]
    fn copy_location_single_axis() {
        let target = ConstraintTarget::object("t");
        let mut c = CopyLocation::new("test", target);
        c.axes = AxisFlags::X;
        let owner = Transform { location: [1.0, 2.0, 3.0], ..Transform::identity() };
        let tgt = Transform { location: [10.0, 20.0, 30.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq(ctx.owner_transform.location[0], 10.0));
        assert!(approx_eq(ctx.owner_transform.location[1], 2.0));
        assert!(approx_eq(ctx.owner_transform.location[2], 3.0));
    }

    #[test]
    fn copy_location_disabled() {
        let target = ConstraintTarget::object("t");
        let mut c = CopyLocation::new("test", target);
        c.base.enabled = false;
        let owner = Transform { location: [1.0, 2.0, 3.0], ..Transform::identity() };
        let tgt = Transform { location: [10.0, 20.0, 30.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq3(ctx.owner_transform.location, [1.0, 2.0, 3.0]));
    }

    // ---- CopyRotation ----

    #[test]
    fn copy_rotation_replace_identity() {
        let target = ConstraintTarget::object("t");
        let c = CopyRotation::new("test", target);
        let owner = Transform::identity();
        let tgt = Transform {
            rotation: [0.0, 0.0, 0.3827, 0.9239], // ~45 deg around Z
            ..Transform::identity()
        };
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        // Result should match the target rotation.
        let r = ctx.owner_transform.rotation;
        assert!(approx_eq(r[2], 0.3827) && approx_eq(r[3], 0.9239));
    }

    #[test]
    fn copy_rotation_half_influence() {
        let target = ConstraintTarget::object("t");
        let mut c = CopyRotation::new("test", target);
        c.base.influence = 0.5;
        let tgt = Transform {
            rotation: [0.0, 0.0, 0.3827, 0.9239], // ~45 deg around Z
            ..Transform::identity()
        };
        let mut ctx = ConstraintContext::with_target(Transform::identity(), tgt);
        c.evaluate(&mut ctx);
        // Half influence should give roughly half the rotation angle.
        let r = ctx.owner_transform.rotation;
        // z component should be about half of 0.3827 (~0.195) but nlerp is an approximation.
        assert!(r[2] > 0.0 && r[2] < 0.38);
    }

    // ---- CopyScale ----

    #[test]
    fn copy_scale_basic() {
        let target = ConstraintTarget::object("t");
        let c = CopyScale::new("test", target);
        let tgt = Transform { scale: [2.0, 3.0, 4.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(Transform::identity(), tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq3(ctx.owner_transform.scale, [2.0, 3.0, 4.0]));
    }

    #[test]
    fn copy_scale_power() {
        let target = ConstraintTarget::object("t");
        let mut c = CopyScale::new("test", target);
        c.power = 2.0;
        let tgt = Transform { scale: [3.0, 1.0, 1.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(Transform::identity(), tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq(ctx.owner_transform.scale[0], 9.0)); // 3^2
    }

    // ---- CopyTransforms ----

    #[test]
    fn copy_transforms_replace() {
        let target = ConstraintTarget::object("t");
        let c = CopyTransforms::new("test", target);
        let tgt = Transform {
            location: [1.0, 2.0, 3.0],
            scale: [2.0, 2.0, 2.0],
            ..Transform::identity()
        };
        let mut ctx = ConstraintContext::with_target(Transform::identity(), tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq3(ctx.owner_transform.location, [1.0, 2.0, 3.0]));
        assert!(approx_eq3(ctx.owner_transform.scale, [2.0, 2.0, 2.0]));
    }

    // ---- MaintainVolume ----

    #[test]
    fn maintain_volume_y_free() {
        let mut c = MaintainVolume::new("test");
        c.volume = 1.0;
        c.free_axis = transform::maintain_volume::FreeAxis::Y;
        let owner = Transform { scale: [1.0, 2.0, 1.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::owner_only(owner);
        c.evaluate(&mut ctx);
        // With free_axis Y and scale Y=2, correction = sqrt(1.0/2.0) ~ 0.7071
        let correction = (0.5_f32).sqrt();
        assert!(approx_eq(ctx.owner_transform.scale[0], correction));
        assert!(approx_eq(ctx.owner_transform.scale[2], correction));
        assert!(approx_eq(ctx.owner_transform.scale[1], 2.0)); // unchanged
    }

    #[test]
    fn maintain_volume_zero_free_scale() {
        let mut c = MaintainVolume::new("test");
        c.volume = 1.0;
        let owner = Transform { scale: [1.0, 0.0, 1.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::owner_only(owner);
        c.evaluate(&mut ctx);
        // Should not panic, scale unchanged.
        assert!(approx_eq(ctx.owner_transform.scale[0], 1.0));
    }

    // ---- Transformation ----

    #[test]
    fn transformation_map_location_to_location() {
        let target = ConstraintTarget::object("t");
        let mut c = Transformation::new("test", target);
        c.source_channel = transform::transformation::MapChannel::LocationX;
        c.dest_channel = transform::transformation::MapChannel::LocationY;
        c.source_min = 0.0;
        c.source_max = 10.0;
        c.dest_min = 0.0;
        c.dest_max = 5.0;
        let tgt = Transform { location: [5.0, 0.0, 0.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(Transform::identity(), tgt);
        c.evaluate(&mut ctx);
        // t = (5-0)/10 = 0.5; mapped = 0 + 0.5*5 = 2.5
        assert!(approx_eq(ctx.owner_transform.location[1], 2.5));
    }

    #[test]
    fn transformation_zero_range() {
        let target = ConstraintTarget::object("t");
        let mut c = Transformation::new("test", target);
        c.source_min = 5.0;
        c.source_max = 5.0; // zero range
        let tgt = Transform::identity();
        let mut ctx = ConstraintContext::with_target(Transform::identity(), tgt);
        c.evaluate(&mut ctx);
        // Should not panic, t = 0.0 so mapped = dest_min.
    }

    // ---- LimitLocation ----

    #[test]
    fn limit_location_clamp_min() {
        let mut c = LimitLocation::new("test");
        c.use_min_x = true;
        c.min[0] = -1.0;
        let owner = Transform { location: [-5.0, 0.0, 0.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::owner_only(owner);
        c.evaluate(&mut ctx);
        assert!(approx_eq(ctx.owner_transform.location[0], -1.0));
    }

    #[test]
    fn limit_location_no_clamp_within_bounds() {
        let mut c = LimitLocation::new("test");
        c.use_min_x = true;
        c.use_max_x = true;
        c.min[0] = -1.0;
        c.max[0] = 1.0;
        let owner = Transform { location: [0.5, 0.0, 0.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::owner_only(owner);
        c.evaluate(&mut ctx);
        assert!(approx_eq(ctx.owner_transform.location[0], 0.5));
    }

    // ---- LimitRotation ----

    #[test]
    fn limit_rotation_clamp() {
        let mut c = LimitRotation::new("test");
        c.use_limit_x = true;
        c.min[0] = -0.5;
        c.max[0] = 0.5;
        // Rotation with large X euler: ~90 deg around X.
        let owner = Transform {
            rotation: [0.7071, 0.0, 0.0, 0.7071],
            ..Transform::identity()
        };
        let mut ctx = ConstraintContext::owner_only(owner);
        c.evaluate(&mut ctx);
        // The X euler should be clamped to 0.5 radians (was ~1.57).
        // We just check the rotation changed.
        assert!(ctx.owner_transform.rotation != owner.rotation);
    }

    // ---- LimitScale ----

    #[test]
    fn limit_scale_clamp_min() {
        let mut c = LimitScale::new("test");
        c.use_min_x = true;
        c.min[0] = 0.5;
        let owner = Transform { scale: [0.1, 1.0, 1.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::owner_only(owner);
        c.evaluate(&mut ctx);
        assert!(approx_eq(ctx.owner_transform.scale[0], 0.5));
    }

    // ---- LimitDistance ----

    #[test]
    fn limit_distance_inside() {
        let target = ConstraintTarget::object("t");
        let mut c = LimitDistance::new("test", target);
        c.distance = 2.0;
        c.clamp_mode = limit::DistanceClampMode::Inside;
        let owner = Transform { location: [5.0, 0.0, 0.0], ..Transform::identity() };
        let tgt = Transform::identity();
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        let d = ctx.owner_transform.location[0];
        assert!(approx_eq(d, 2.0));
    }

    #[test]
    fn limit_distance_outside() {
        let target = ConstraintTarget::object("t");
        let mut c = LimitDistance::new("test", target);
        c.distance = 5.0;
        c.clamp_mode = limit::DistanceClampMode::Outside;
        let owner = Transform { location: [1.0, 0.0, 0.0], ..Transform::identity() };
        let tgt = Transform::identity();
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq(ctx.owner_transform.location[0], 5.0));
    }

    #[test]
    fn limit_distance_zero_distance_safe() {
        let target = ConstraintTarget::object("t");
        let mut c = LimitDistance::new("test", target);
        c.distance = 1.0;
        c.clamp_mode = limit::DistanceClampMode::Outside;
        // Owner at same pos as target - should not panic.
        let owner = Transform::identity();
        let tgt = Transform::identity();
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        // Owner should remain at origin (can't determine direction).
        assert!(approx_eq3(ctx.owner_transform.location, [0.0, 0.0, 0.0]));
    }

    // ---- Floor ----

    #[test]
    fn floor_clamps_below() {
        let target = ConstraintTarget::object("t");
        let c = Floor::new("test", target);
        let owner = Transform { location: [0.0, -5.0, 0.0], ..Transform::identity() };
        let tgt = Transform::identity();
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq(ctx.owner_transform.location[1], 0.0));
    }

    #[test]
    fn floor_no_clamp_above() {
        let target = ConstraintTarget::object("t");
        let c = Floor::new("test", target);
        let owner = Transform { location: [0.0, 5.0, 0.0], ..Transform::identity() };
        let tgt = Transform::identity();
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        assert!(approx_eq(ctx.owner_transform.location[1], 5.0));
    }

    // ---- TrackTo ----

    #[test]
    fn track_to_coincident_safe() {
        let target = ConstraintTarget::object("t");
        let c = TrackTo::new("test", target);
        let owner = Transform::identity();
        let tgt = Transform::identity();
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        // Should not panic, rotation unchanged.
        assert_eq!(ctx.owner_transform.rotation, owner.rotation);
    }

    #[test]
    fn track_to_modifies_rotation() {
        let target = ConstraintTarget::object("t");
        let c = TrackTo::new("test", target);
        let owner = Transform { location: [0.0, 0.0, 0.0], ..Transform::identity() };
        let tgt = Transform { location: [5.0, 0.0, 0.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        // Rotation should have changed (tracking toward +X with -Z track axis).
        assert!(ctx.owner_transform.rotation != [0.0, 0.0, 0.0, 1.0]);
    }

    // ---- DampedTrack ----

    #[test]
    fn damped_track_coincident_safe() {
        let target = ConstraintTarget::object("t");
        let c = DampedTrack::new("test", target);
        let owner = Transform::identity();
        let tgt = Transform::identity();
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        // No panic, identity preserved.
        assert_eq!(ctx.owner_transform.rotation, [0.0, 0.0, 0.0, 1.0]);
    }

    // ---- LockedTrack ----

    #[test]
    fn locked_track_coincident_safe() {
        let target = ConstraintTarget::object("t");
        let c = LockedTrack::new("test", target);
        let owner = Transform::identity();
        let tgt = Transform::identity();
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        assert_eq!(ctx.owner_transform.rotation, [0.0, 0.0, 0.0, 1.0]);
    }

    // ---- StretchTo ----

    #[test]
    fn stretch_to_scales_along_track() {
        let target = ConstraintTarget::object("t");
        let mut c = StretchTo::new("test", target);
        c.rest_length = 1.0;
        c.volume = tracking::VolumeMode::None;
        let owner = Transform::identity();
        let tgt = Transform { location: [0.0, 3.0, 0.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        // Tracking PosY, rest_length=1, distance=3, so Y scale should be 3.
        assert!(approx_eq(ctx.owner_transform.scale[1], 3.0));
    }

    // ---- ChildOf ----

    #[test]
    fn child_of_follows_parent() {
        let target = ConstraintTarget::object("t");
        let c = ChildOf::new("test", target);
        let owner = Transform::identity();
        let tgt = Transform { location: [5.0, 0.0, 0.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        // Owner should be at target's location (parent * child = target * identity).
        assert!(approx_eq(ctx.owner_transform.location[0], 5.0));
    }

    // ---- Pivot ----

    #[test]
    fn pivot_no_rotation_no_change() {
        let mut c = Pivot::new("test");
        c.offset = [5.0, 0.0, 0.0];
        let owner = Transform { location: [3.0, 0.0, 0.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::owner_only(owner);
        // With identity rotation, rotating offset around pivot should give the
        // same location (identity rotation does nothing).
        c.evaluate(&mut ctx);
        assert!(approx_eq3(ctx.owner_transform.location, [3.0, 0.0, 0.0]));
    }

    // ---- ActionConstraint ----

    #[test]
    fn action_constraint_map_to_frame() {
        let target = ConstraintTarget::object("t");
        let mut c = ActionConstraint::new("test", target, "walk");
        c.source_min = 0.0;
        c.source_max = 1.0;
        c.action_start = 10.0;
        c.action_end = 50.0;
        assert!(approx_eq(c.map_to_frame(0.5), 30.0));
        assert!(approx_eq(c.map_to_frame(0.0), 10.0));
        assert!(approx_eq(c.map_to_frame(1.0), 50.0));
    }

    #[test]
    fn action_constraint_zero_range() {
        let target = ConstraintTarget::object("t");
        let mut c = ActionConstraint::new("test", target, "walk");
        c.source_min = 5.0;
        c.source_max = 5.0;
        // Should not panic.
        assert!(approx_eq(c.map_to_frame(5.0), c.action_start));
    }

    // ---- Transform blend ----

    #[test]
    fn transform_blend_identity() {
        let a = Transform::identity();
        let b = Transform {
            location: [10.0, 0.0, 0.0],
            scale: [2.0, 2.0, 2.0],
            ..Transform::identity()
        };
        let c = a.blend(&b, 0.5);
        assert!(approx_eq(c.location[0], 5.0));
        assert!(approx_eq(c.scale[0], 1.5));
    }

    // ---- Influence zero means no change ----

    #[test]
    fn zero_influence_no_change() {
        let target = ConstraintTarget::object("t");
        let mut c = CopyLocation::new("test", target);
        c.base.influence = 0.0;
        let owner = Transform { location: [1.0, 2.0, 3.0], ..Transform::identity() };
        let tgt = Transform { location: [10.0, 20.0, 30.0], ..Transform::identity() };
        let mut ctx = ConstraintContext::with_target(owner, tgt);
        c.evaluate(&mut ctx);
        // is_active() returns false when influence == 0.
        assert!(approx_eq3(ctx.owner_transform.location, [1.0, 2.0, 3.0]));
    }
}
