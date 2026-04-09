//! Dissolve operations — remove elements while preserving surrounding topology.

pub mod edges;
pub mod faces;
pub mod verts;

pub use edges::dissolve_edges;
pub use faces::dissolve_faces;
pub use verts::dissolve_verts;
