//! Triangulation operations.

pub mod ear_clip;
pub mod fan;

pub use ear_clip::{ear_clip_face, EarClipTriangle};
pub use fan::{fan_triangulate_all, fan_triangulate_face};

/// Method to use for triangulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriangulateMethod {
    /// Simple fan from first vertex — fast but only correct for convex polygons.
    Fan,
    /// Ear clipping — works for any simple polygon.
    EarClip,
}

impl Default for TriangulateMethod {
    fn default() -> Self {
        TriangulateMethod::EarClip
    }
}
