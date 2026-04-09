//! Vertex extrusion — duplicates selected vertices and connects with edges.

use forge3d_math::Vec3;
use forge3d_mesh::Mesh;

use crate::common::{OpError, OpResult, SelectionHelper};
use super::ExtrudeParams;

/// Extrudes selected vertices by duplicating them at an offset and creating
/// connecting edges.
pub fn extrude_verts(mesh: &mut Mesh, params: &ExtrudeParams) -> OpResult<()> {
    let selected = SelectionHelper::selected_verts(mesh);
    if selected.is_empty() {
        return Err(OpError::InvalidSelection("no vertices selected".into()));
    }

    let direction = params
        .direction
        .unwrap_or(Vec3::Y)
        .normalize_or_zero();
    let displacement = direction * params.offset;

    for &vert_h in &selected {
        if mesh.verts.contains(vert_h) {
            let co = mesh.verts.get(vert_h).unwrap().co;
            let new_h = mesh.create_vert(co + displacement);
            mesh.create_edge(vert_h, new_h)?;
        }
    }

    Ok(())
}
