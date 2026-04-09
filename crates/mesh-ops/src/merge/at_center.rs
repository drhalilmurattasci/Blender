//! Merge at center — collapses selected vertices to their centroid.

use forge3d_math::Vec3;
use forge3d_mesh::Mesh;

use crate::common::{OpError, OpResult, SelectionHelper};

/// Merges all selected vertices at their centroid.
///
/// All selected vertices are moved to the average position, then welded
/// into a single vertex.
pub fn merge_at_center(mesh: &mut Mesh) -> OpResult<()> {
    let selected = SelectionHelper::selected_verts(mesh);
    if selected.len() < 2 {
        return Err(OpError::InvalidSelection(
            "need at least 2 selected vertices".into(),
        ));
    }

    // Compute centroid.
    let mut sum = Vec3::ZERO;
    let mut count = 0u32;
    for &vh in &selected {
        if let Some(v) = mesh.verts.get(vh) {
            sum += v.co;
            count += 1;
        }
    }

    if count == 0 {
        return Ok(());
    }

    let centroid = sum / count as f32;

    // Keep the first selected vertex as the survivor.
    let survivor = selected[0];
    if let Some(v) = mesh.verts.get_mut(survivor) {
        v.co = centroid;
    }

    // Kill all other selected vertices.
    for &vh in &selected[1..] {
        if mesh.verts.contains(vh) {
            let _ = mesh.kill_vert(vh);
        }
    }

    Ok(())
}
