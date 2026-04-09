//! Mesh element types: vertices, edges, loops, and faces.

pub mod edge;
pub mod face;
pub mod flags;
pub mod loop_elem;
pub mod vert;

pub use edge::{DiskLink, Edge};
pub use face::Face;
pub use flags::ElemFlags;
pub use loop_elem::LoopElem;
pub use vert::Vert;
