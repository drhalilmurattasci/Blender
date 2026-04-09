//! Extrude operations for vertices, edges, and faces.

pub mod edges;
pub mod faces;
pub mod verts;

pub use edges::extrude_edges;
pub use faces::extrude_faces;
pub use verts::extrude_verts;

use forge3d_math::Vec3;

/// Parameters controlling extrude operations.
#[derive(Debug, Clone)]
pub struct ExtrudeParams {
    /// Distance to extrude.
    pub offset: f32,
    /// Optional extrusion direction. If `None`, faces use their normal and
    /// edges/verts use +Y.
    pub direction: Option<Vec3>,
}

impl Default for ExtrudeParams {
    fn default() -> Self {
        Self {
            offset: 1.0,
            direction: None,
        }
    }
}
