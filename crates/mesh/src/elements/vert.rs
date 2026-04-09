//! Vertex element.

use forge3d_alloc::Handle;
use forge3d_math::Vec3;
use serde::{Deserialize, Serialize};

use super::flags::ElemFlags;
use super::edge::Edge;

/// A mesh vertex storing position, normal, and a link to its first disk edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vert {
    /// World-space (or object-space) position.
    pub co: Vec3,
    /// Vertex normal (may be computed from adjacent faces).
    pub normal: Vec3,
    /// Handle to the first edge in this vertex's disk cycle.
    /// `None` for isolated vertices.
    pub edge: Option<Handle<Edge>>,
    /// Element flags.
    pub flags: ElemFlags,
}

impl Vert {
    /// Creates a new vertex at the given position with default normal and no edge.
    pub fn new(co: Vec3) -> Self {
        Self {
            co,
            normal: Vec3::ZERO,
            edge: None,
            flags: ElemFlags::empty(),
        }
    }
}
