//! High-level topology iterators.

pub mod edge_iter;
pub mod face_iter;
pub mod loop_iter;
pub mod vert_iter;

pub use edge_iter::EdgeFaces;
pub use face_iter::{FaceEdges, FaceVerts};
pub use loop_iter::FaceLoops;
pub use vert_iter::{VertEdges, VertFaces};
