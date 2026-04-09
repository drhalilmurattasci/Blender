//! # forge3d-mesh-ops
//!
//! High-level mesh editing operations for the Forge3D engine.
//!
//! This crate builds on top of `forge3d-mesh` and provides operations such as
//! bevel, subdivide, extrude, dissolve, boolean, triangulate, merge, bisect,
//! and remesh.

pub mod bevel;
pub mod bisect;
pub mod boolean;
pub mod common;
pub mod dissolve;
pub mod extrude;
pub mod merge;
pub mod remesh;
pub mod subdivide;
pub mod triangulate;

// Re-export commonly used types.
pub use bevel::BevelParams;
pub use bisect::BisectParams;
pub use boolean::BooleanOp;
pub use common::{OpError, OpResult};
pub use extrude::ExtrudeParams;
pub use merge::MergeParams;
pub use remesh::RemeshParams;
pub use subdivide::{CatmullClarkParams, SubdivideParams};
pub use triangulate::TriangulateMethod;
