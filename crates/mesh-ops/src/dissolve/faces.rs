//! Dissolve faces — removes faces while keeping edges and vertices.

use forge3d_mesh::Mesh;

use crate::common::{OpError, OpResult, SelectionHelper};

/// Dissolves selected faces, removing them from the mesh.
///
/// Interior edges (shared only by dissolving faces) are also removed.
/// Boundary edges (shared with non-selected faces) are kept.
pub fn dissolve_faces(mesh: &mut Mesh) -> OpResult<()> {
    let selected = SelectionHelper::selected_faces(mesh);
    if selected.is_empty() {
        return Err(OpError::InvalidSelection("no faces selected".into()));
    }

    // Simply kill each selected face. Edges remain unless they become wire edges.
    for &face_h in &selected {
        if mesh.faces.contains(face_h) {
            mesh.kill_face(face_h)?;
        }
    }

    Ok(())
}
