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
