//! Bevel operations for vertices and edges.

pub mod edge_bevel;
pub mod vertex_bevel;

pub use edge_bevel::edge_bevel;
pub use vertex_bevel::vertex_bevel;

/// Parameters controlling bevel operations.
#[derive(Debug, Clone)]
pub struct BevelParams {
    /// Distance to offset from the original element.
    pub offset: f32,
    /// Number of segments in the bevel profile.
    pub segments: u32,
    /// Clamping factor (0.0 = no clamp, 1.0 = full clamp).
    pub clamp_overlap: f32,
}

impl Default for BevelParams {
    fn default() -> Self {
        Self {
            offset: 0.1,
            segments: 1,
            clamp_overlap: 0.0,
        }
    }
}
