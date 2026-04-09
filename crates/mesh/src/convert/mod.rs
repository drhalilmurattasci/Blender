//! Conversion utilities (normals, triangulation).

pub mod normals;
pub mod to_triangles;

pub use normals::{compute_face_normal, compute_face_normals, compute_vertex_normals};
pub use to_triangles::{triangulate_all_fan, triangulate_face_fan, Triangle};
