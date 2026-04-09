//! # forge3d-depsgraph
//!
//! Dependency graph for Forge3D.
//!
//! The depsgraph tracks relationships between scene data (objects, bones,
//! constraints, modifiers, etc.) and evaluates them in topologically-sorted
//! order with parallel execution via rayon.

pub mod builder;
pub mod evaluate;
pub mod node;
pub mod parallel;
pub mod tag;

use thiserror::Error;

/// Errors from the dependency graph.
#[derive(Debug, Error)]
pub enum DepsgraphError {
    #[error("node `{0}` not found in the dependency graph")]
    NodeNotFound(String),

    #[error("cyclic dependency detected involving node `{0}`")]
    CyclicDependency(String),

    #[error("evaluation failed for node `{0}`: {1}")]
    EvalFailed(String, String),

    #[error("graph build error: {0}")]
    BuildError(String),
}

pub type DepsgraphResult<T> = Result<T, DepsgraphError>;

/// Unique identifier for a node in the dependency graph.
pub type NodeId = u32;
