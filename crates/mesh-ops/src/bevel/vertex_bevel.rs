//! Vertex bevel — replaces each selected vertex with a face, pushing edges outward.

use forge3d_alloc::Handle;
use forge3d_math::Vec3;
use forge3d_mesh::{Mesh, Vert};
use smallvec::SmallVec;

use crate::common::{OpError, OpResult, SelectionHelper};
use super::BevelParams;

/// Bevels each selected vertex by replacing it with a new face.
///
/// For each selected vertex V with N edges, the operation:
/// 1. Creates N new vertices along each edge at `offset` distance from V.
/// 2. Reconnects the edges to the new vertices.
/// 3. Creates a new N-gon face connecting the new vertices.
pub fn vertex_bevel(mesh: &mut Mesh, params: &BevelParams) -> OpResult<()> {
    let selected = SelectionHelper::selected_verts(mesh);
    if selected.is_empty() {
        return Err(OpError::InvalidSelection("no vertices selected".into()));
    }

    let offset = params.offset.max(0.0);
    if offset < f32::EPSILON {
        return Ok(());
    }

    for vert_h in &selected {
        bevel_single_vert(mesh, *vert_h, offset)?;
    }

    Ok(())
}

/// Bevels a single vertex.
fn bevel_single_vert(
    mesh: &mut Mesh,
    vert_h: Handle<Vert>,
    offset: f32,
) -> OpResult<()> {
    let vert = mesh
        .verts
        .get(vert_h)
        .ok_or_else(|| OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", vert_h))))?;
    let center = vert.co;

    // Collect connected edge information: the other vertex and the direction.
    let mut neighbors: SmallVec<[(Handle<Vert>, Vec3); 8]> = SmallVec::new();

    let first_edge = match vert.edge {
        Some(h) => h,
        None => return Ok(()), // Isolated vertex, nothing to bevel.
    };

    let mut cur_edge_h = first_edge;
    loop {
        let edge = mesh.edges.get(cur_edge_h).ok_or_else(|| {
            OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", cur_edge_h)))
        })?;
        let other_h = edge.other_vert(vert_h);
        let other = mesh.verts.get(other_h).ok_or_else(|| {
            OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", other_h)))
        })?;

        let dir = (other.co - center).normalize_or_zero();
        neighbors.push((other_h, dir));

        cur_edge_h = edge.disk_link(vert_h).next;
        if cur_edge_h == first_edge {
            break;
        }
    }

    if neighbors.len() < 2 {
        return Ok(()); // Wire vertex or single-edge, skip.
    }

    // Create new vertices along each edge direction.
    let mut new_verts: SmallVec<[Handle<Vert>; 8]> = SmallVec::new();
    for (_, dir) in &neighbors {
        let new_co = center + *dir * offset;
        let nv = mesh.create_vert(new_co);
        new_verts.push(nv);
    }

    // Remove the original vertex first (which also removes connected edges/faces).
    // We do this BEFORE creating the bevel face so that kill_vert doesn't
    // accidentally destroy our new geometry.
    mesh.kill_vert(vert_h)?;

    // Create the bevel face from the new vertices.
    let face_verts: Vec<Handle<Vert>> = new_verts.iter().copied().collect();
    if face_verts.len() >= 3 {
        mesh.create_face(&face_verts)?;
    }

    // Reconnect: for each original neighbor, create an edge to the corresponding
    // new vertex. This is a simplified reconnection; a full implementation would
    // also rebuild the adjacent faces.
    for (i, (neighbor_h, _)) in neighbors.iter().enumerate() {
        if mesh.verts.contains(*neighbor_h) && mesh.find_edge(new_verts[i], *neighbor_h).is_none() {
            let _ = mesh.create_edge(new_verts[i], *neighbor_h);
        }
    }

    Ok(())
}
