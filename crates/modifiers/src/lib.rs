//! # forge3d-modifiers
//!
//! Modifier stack system for Forge3D. Modifiers are non-destructive
//! operations applied to mesh data in a defined order. Categories:
//!
//! - **Generate**: create geometry (array, mirror, boolean, screw, solidify)
//! - **Deform**: reshape vertices (lattice, armature, shrinkwrap, smooth)
//! - **Modify**: alter attributes without changing topology (vertex weight, UV warp)
//! - **Physics**: cloth, fluid, softbody stubs

pub mod common;
pub mod deform;
pub mod generate;
pub mod modify;
pub mod physics;

pub use common::{Modifier, ModifierFlags, ModifierStack, ModifierType};
pub use deform::{
    DeformAxis, ShrinkwrapMode, ShrinkwrapModifier, SimpleDeformMode, SimpleDeformModifier,
    SmoothModifier,
};
pub use generate::{
    ArrayFitType, ArrayModifier, BevelLimitMethod, BevelModifier, MirrorModifier,
    SolidifyModifier, SubdivBoundarySmooth, SubdivUvSmooth, SubdivisionModifier,
};
pub use modify::{
    DecimateMode, DecimateModifier, TriangulateModifier, TriangulateNgonMethod,
    TriangulateQuadMethod, WeightedNormalMode, WeightedNormalModifier,
};
pub use physics::{ClothModifier, CollisionModifier};

use thiserror::Error;

/// Errors from modifier evaluation.
#[derive(Debug, Error)]
pub enum ModifierError {
    #[error("modifier `{name}` failed: {reason}")]
    EvaluationFailed { name: String, reason: String },

    #[error("invalid parameter `{param}` for modifier `{modifier}`: {detail}")]
    InvalidParameter {
        modifier: String,
        param: String,
        detail: String,
    },

    #[error("mesh data required but missing")]
    MissingMesh,

    #[error("dependency cycle detected in modifier stack")]
    DependencyCycle,
}

/// Result alias.
pub type ModifierResult<T> = Result<T, ModifierError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Default values are sensible
    // ---------------------------------------------------------------

    #[test]
    fn array_modifier_defaults() {
        let m = generate::ArrayModifier::default();
        assert_eq!(m.count, 2);
        assert!(m.use_relative_offset);
        assert!(!m.use_constant_offset);
        assert!(m.flags.contains(ModifierFlags::VIEWPORT));
        assert!(m.flags.contains(ModifierFlags::RENDER));
    }

    #[test]
    fn mirror_modifier_defaults() {
        let m = generate::MirrorModifier::default();
        assert!(m.axis_x);
        assert!(!m.axis_y);
        assert!(!m.axis_z);
        assert!(m.merge);
    }

    #[test]
    fn subdivision_modifier_defaults() {
        let m = generate::SubdivisionModifier::default();
        assert_eq!(m.levels_viewport, 1);
        assert_eq!(m.levels_render, 2);
        assert!(!m.use_simple);
    }

    #[test]
    fn bevel_modifier_defaults() {
        let m = generate::BevelModifier::default();
        assert_eq!(m.segments, 1);
        assert!((m.profile - 0.5).abs() < f32::EPSILON);
        assert!(!m.vertex_only);
    }

    #[test]
    fn cloth_modifier_defaults() {
        let m = physics::ClothModifier::default();
        assert!(m.structural_stiffness > 0.0);
        assert!(m.mass > 0.0);
        assert_eq!(m.quality, 5);
    }

    #[test]
    fn simple_deform_modifier_defaults() {
        let m = deform::SimpleDeformModifier::default();
        assert_eq!(m.mode, deform::SimpleDeformMode::Twist);
        assert!((m.factor - 0.0).abs() < f32::EPSILON);
        assert!((m.limit_max - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn triangulate_modifier_defaults() {
        let m = modify::TriangulateModifier::default();
        assert_eq!(m.min_vertices, 4);
        assert!(m.keep_normals);
    }

    #[test]
    fn decimate_modifier_defaults() {
        let m = modify::DecimateModifier::default();
        assert!((m.ratio - 1.0).abs() < f32::EPSILON);
        assert_eq!(m.mode, modify::DecimateMode::Collapse);
    }

    // ---------------------------------------------------------------
    // Modifier stack ordering
    // ---------------------------------------------------------------

    #[test]
    fn stack_push_and_order() {
        let mut stack = ModifierStack::new();
        stack.push(Box::new(generate::ArrayModifier::default()));
        stack.push(Box::new(generate::MirrorModifier::default()));
        stack.push(Box::new(generate::SubdivisionModifier::default()));

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.get(0).unwrap().modifier_type(), ModifierType::Array);
        assert_eq!(stack.get(1).unwrap().modifier_type(), ModifierType::Mirror);
        assert_eq!(stack.get(2).unwrap().modifier_type(), ModifierType::Subdivision);
    }

    #[test]
    fn stack_insert_at_index() {
        let mut stack = ModifierStack::new();
        stack.push(Box::new(generate::ArrayModifier::default()));
        stack.push(Box::new(generate::SubdivisionModifier::default()));
        // Insert mirror between array and subdiv.
        stack.insert(1, Box::new(generate::MirrorModifier::default()));

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.get(0).unwrap().modifier_type(), ModifierType::Array);
        assert_eq!(stack.get(1).unwrap().modifier_type(), ModifierType::Mirror);
        assert_eq!(stack.get(2).unwrap().modifier_type(), ModifierType::Subdivision);
    }

    #[test]
    fn stack_insert_beyond_end_clamps() {
        let mut stack = ModifierStack::new();
        stack.push(Box::new(generate::ArrayModifier::default()));
        stack.insert(100, Box::new(generate::MirrorModifier::default()));
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.get(1).unwrap().modifier_type(), ModifierType::Mirror);
    }

    #[test]
    fn stack_remove() {
        let mut stack = ModifierStack::new();
        stack.push(Box::new(generate::ArrayModifier::default()));
        stack.push(Box::new(generate::MirrorModifier::default()));

        let removed = stack.remove(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().modifier_type(), ModifierType::Array);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.get(0).unwrap().modifier_type(), ModifierType::Mirror);
    }

    #[test]
    fn stack_remove_out_of_bounds() {
        let mut stack = ModifierStack::new();
        assert!(stack.remove(0).is_none());
    }

    #[test]
    fn stack_reorder() {
        let mut stack = ModifierStack::new();
        stack.push(Box::new(generate::ArrayModifier::default()));
        stack.push(Box::new(generate::MirrorModifier::default()));
        stack.push(Box::new(generate::SubdivisionModifier::default()));

        // Move Array (index 0) to index 2.
        stack.reorder(0, 2);
        assert_eq!(stack.get(0).unwrap().modifier_type(), ModifierType::Mirror);
        assert_eq!(stack.get(1).unwrap().modifier_type(), ModifierType::Subdivision);
        assert_eq!(stack.get(2).unwrap().modifier_type(), ModifierType::Array);
    }

    #[test]
    fn stack_is_empty() {
        let stack = ModifierStack::new();
        assert!(stack.is_empty());
    }

    #[test]
    fn stack_apply_all_with_disabled() {
        let mut stack = ModifierStack::new();

        let mut array = generate::ArrayModifier::default();
        // Disable viewport flag.
        array.flags = ModifierFlags::RENDER;
        stack.push(Box::new(array));
        stack.push(Box::new(generate::MirrorModifier::default()));

        // apply_all should skip the disabled array modifier.
        let mut dummy: u32 = 0;
        stack.apply_all(&mut dummy).unwrap();
    }

    #[test]
    fn stack_debug_format() {
        let mut stack = ModifierStack::new();
        stack.push(Box::new(generate::ArrayModifier::default()));
        let debug_str = format!("{:?}", stack);
        assert!(debug_str.contains("ModifierStack"));
        assert!(debug_str.contains("Array"));
    }

    #[test]
    fn stack_iter_and_iter_mut() {
        let mut stack = ModifierStack::new();
        stack.push(Box::new(generate::ArrayModifier::default()));
        stack.push(Box::new(generate::MirrorModifier::default()));

        assert_eq!(stack.iter().count(), 2);

        // Use iter_mut to disable all modifiers.
        for m in stack.iter_mut() {
            m.set_flags(ModifierFlags::empty());
        }

        // Verify flags were changed.
        assert_eq!(stack.get(0).unwrap().flags(), ModifierFlags::empty());
        assert_eq!(stack.get(1).unwrap().flags(), ModifierFlags::empty());
    }
}
