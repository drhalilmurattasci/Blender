//! Remesh operations.

pub mod voxel;

pub use voxel::voxel_remesh;

/// Parameters for voxel remeshing.
#[derive(Debug, Clone)]
pub struct RemeshParams {
    /// Size of each voxel cell.
    pub voxel_size: f32,
    /// If `true`, smooth the result after remeshing.
    pub smooth: bool,
}

impl Default for RemeshParams {
    fn default() -> Self {
        Self {
            voxel_size: 0.1,
            smooth: true,
        }
    }
}
