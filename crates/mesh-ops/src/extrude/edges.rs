//! Edge extrusion — extends selected edges outward creating new faces.

use forge3d_alloc::Handle;
use forge3d_math::Vec3;
use forge3d_mesh::{Edge, Mesh};

use crate::common::{OpError, OpResult, SelectionHelper};
use super::ExtrudeParams;

/// Extrudes selected edges along the given direction.
///
/// For each selected edge, two new vertices are created at offset distance
/// and a quad face connects the old and new edges.
pub fn extrude_edges(mesh: &mut Mesh, params: &ExtrudeParams) -> OpResult<()> {
    let selected = SelectionHelper::selected_edges(mesh);
    if selected.is_empty() {
        return Err(OpError::InvalidSelection("no edges selected".into()));
    }

    let direction = params
        .direction
        .unwrap_or(Vec3::Y)
        .normalize_or_zero();
    let offset = params.offset;

    for &edge_h in &selected {
        if mesh.edges.contains(edge_h) {
            extrude_single_edge(mesh, edge_h, direction * offset)?;
        }
    }

    Ok(())
}

fn extrude_single_edge(
    mesh: &mut Mesh,
    edge_h: Handle<Edge>,
    displacement: Vec3,
) -> OpResult<()> {
    let edge = mesh.edges.get(edge_h).ok_or_else(|| {
        OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", edge_h)))
    })?;

    let v1_h = edge.v1;
    let v2_h = edge.v2;

    let co1 = mesh.verts.get(v1_h).unwrap().co;
    let co2 = mesh.verts.get(v2_h).unwrap().co;

    let nv1 = mesh.create_vert(co1 + displacement);
    let nv2 = mesh.create_vert(co2 + displacement);

    // Create quad: v1 -> v2 -> nv2 -> nv1
    mesh.create_face(&[v1_h, v2_h, nv2, nv1])?;

    Ok(())
}
