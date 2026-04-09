//! Dissolve vertices — merges edges and faces surrounding a vertex.

use forge3d_alloc::Handle;
use forge3d_mesh::{Mesh, Vert};

use crate::common::{OpError, OpResult, SelectionHelper};

/// Dissolves selected vertices, merging their adjacent faces.
///
/// For each selected vertex with exactly two edges, the vertex is removed and
/// the edges are merged into one. For vertices with more edges, adjacent faces
/// are merged into a single face.
pub fn dissolve_verts(mesh: &mut Mesh) -> OpResult<()> {
    let selected = SelectionHelper::selected_verts(mesh);
    if selected.is_empty() {
        return Err(OpError::InvalidSelection("no vertices selected".into()));
    }

    for &vert_h in &selected {
        if mesh.verts.contains(vert_h) {
            dissolve_single_vert(mesh, vert_h)?;
        }
    }

    Ok(())
}

fn dissolve_single_vert(mesh: &mut Mesh, vert_h: Handle<Vert>) -> OpResult<()> {
    let vert = mesh.verts.get(vert_h).ok_or_else(|| {
        OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", vert_h)))
    })?;

    let first_eh = match vert.edge {
        Some(h) => h,
        None => {
            // Isolated vertex — just remove it.
            mesh.verts.remove(vert_h);
            return Ok(());
        }
    };

    // Count edges.
    let mut edge_count = 0u32;
    let mut cur_eh = first_eh;
    loop {
        edge_count += 1;
        let edge = mesh.edges.get(cur_eh).ok_or_else(|| {
            OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", cur_eh)))
        })?;
        cur_eh = edge.disk_link(vert_h).next;
        if cur_eh == first_eh {
            break;
        }
    }

    if edge_count == 2 {
        // Simple case: merge two edges into one.
        let e1_h = first_eh;
        let e1 = mesh.edges.get(e1_h).unwrap();
        let e2_h = e1.disk_link(vert_h).next;
        let e2 = mesh.edges.get(e2_h).unwrap();

        let other1 = e1.other_vert(vert_h);
        let other2 = e2.other_vert(vert_h);

        // Collect faces that reference the dissolving vertex.
        // Kill the vertex (removes edges and faces).
        mesh.kill_vert(vert_h)?;

        // Re-create the merged edge.
        if mesh.verts.contains(other1) && mesh.verts.contains(other2) {
            if mesh.find_edge(other1, other2).is_none() {
                let _ = mesh.create_edge(other1, other2);
            }
        }
    } else {
        // Complex case: collect all adjacent face verts, merge into one face.
        // Collect neighbor vertices in order.
        let mut neighbor_verts = Vec::new();
        let mut cur_eh = first_eh;
        loop {
            let edge = mesh.edges.get(cur_eh).unwrap();
            neighbor_verts.push(edge.other_vert(vert_h));
            cur_eh = edge.disk_link(vert_h).next;
            if cur_eh == first_eh {
                break;
            }
        }

        // Kill the vertex.
        mesh.kill_vert(vert_h)?;

        // Create a new face from neighbor vertices if enough remain.
        let valid: Vec<_> = neighbor_verts
            .iter()
            .copied()
            .filter(|h| mesh.verts.contains(*h))
            .collect();

        if valid.len() >= 3 {
            let _ = mesh.create_face(&valid);
        }
    }

    Ok(())
}
