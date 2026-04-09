//! Voxel remesh — rebuilds mesh topology using a voxel grid.

use forge3d_mesh::Mesh;

use crate::common::{OpError, OpResult};
use super::RemeshParams;

/// Performs a voxel-based remesh of the mesh.
///
/// # Algorithm overview (stub)
///
/// 1. Voxelize the mesh at the given resolution.
/// 2. Flood-fill to determine interior/exterior voxels.
/// 3. Extract an isosurface using Marching Cubes or Dual Contouring.
/// 4. Replace the mesh topology with the extracted surface.
///
/// This is currently a stub — the full implementation requires a voxelization
/// pipeline and isosurface extraction.
pub fn voxel_remesh(mesh: &mut Mesh, params: &RemeshParams) -> OpResult<()> {
    if mesh.face_count() == 0 {
        return Err(OpError::InvalidParam("mesh has no faces to remesh".into()));
    }

    if params.voxel_size <= 0.0 {
        return Err(OpError::InvalidParam(
            "voxel_size must be positive".into(),
        ));
    }

    // Compute bounding box.
    let mut min = forge3d_math::Vec3::splat(f32::MAX);
    let mut max = forge3d_math::Vec3::splat(f32::MIN);

    for (_, v) in mesh.verts.iter() {
        min = min.min(v.co);
        max = max.max(v.co);
    }

    let _extent = max - min;
    let _grid_size = (
        ((_extent.x / params.voxel_size).ceil() as u32).max(1),
        ((_extent.y / params.voxel_size).ceil() as u32).max(1),
        ((_extent.z / params.voxel_size).ceil() as u32).max(1),
    );

    // TODO: Implement voxelization, flood fill, and marching cubes.
    Err(OpError::NotImplemented(
        "voxel remesh requires voxelization and isosurface extraction — pending".into(),
    ))
}
