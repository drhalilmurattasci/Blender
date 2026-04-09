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
