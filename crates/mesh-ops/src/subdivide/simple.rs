//! Simple subdivision — splits every edge at its midpoint.

use std::collections::HashMap;

use forge3d_alloc::Handle;
use forge3d_mesh::{Edge, Face, Mesh, Vert};

use crate::common::{OpError, OpResult};

/// Performs a single pass of simple (midpoint) subdivision.
///
/// Each edge is split at its midpoint, and each face is subdivided by connecting
/// the new midpoint vertices.
pub fn subdivide_simple(mesh: &mut Mesh) -> OpResult<()> {
    // Collect all current edges.
    let edge_handles: Vec<(Handle<Edge>, Handle<Vert>, Handle<Vert>)> = mesh
        .edges
        .iter()
        .map(|(h, e)| (h, e.v1, e.v2))
        .collect();

    // Split each edge at its midpoint, recording old_edge -> new_midpoint_vert.
    let mut edge_midpoints: HashMap<Handle<Edge>, Handle<Vert>> = HashMap::new();

    for (edge_h, v1_h, v2_h) in &edge_handles {
        let co1 = mesh
            .verts
            .get(*v1_h)
            .ok_or_else(|| OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", v1_h))))?
            .co;
        let co2 = mesh
            .verts
            .get(*v2_h)
            .ok_or_else(|| OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", v2_h))))?
            .co;

        let mid = (co1 + co2) * 0.5;
        let mid_h = mesh.create_vert(mid);
        edge_midpoints.insert(*edge_h, mid_h);
    }

    // Collect original faces before modifying topology.
    let face_data: Vec<(Handle<Face>, Vec<Handle<Vert>>, Vec<Handle<Edge>>)> = {
        let mut data = Vec::new();
        let face_handles: Vec<Handle<Face>> = mesh.faces.iter().map(|(h, _)| h).collect();
        for fh in face_handles {
            let face = mesh.faces.get(fh).unwrap();
            let first_lh = face.loop_first;
            if first_lh.is_dangling() {
                continue;
            }
            let mut face_verts = Vec::new();
            let mut face_edges = Vec::new();
            let mut cur_lh = first_lh;
            loop {
                let l = mesh.loops.get(cur_lh).unwrap();
                face_verts.push(l.vert);
                face_edges.push(l.edge);
                cur_lh = l.next;
                if cur_lh == first_lh {
                    break;
                }
            }
            data.push((fh, face_verts, face_edges));
        }
        data
    };

    // Kill old faces first (this removes loops and radial links but keeps edges/verts).
    for (face_h, _, _) in &face_data {
        if mesh.faces.contains(*face_h) {
            mesh.kill_face(*face_h)?;
        }
    }

    // Kill old edges (faces are gone so no radial links remain).
    for (edge_h, _, _) in &edge_handles {
        if mesh.edges.contains(*edge_h) {
            let _ = mesh.kill_edge(*edge_h);
        }
    }

    // Now create the subdivided faces using only the original vert handles and midpoint handles.
    for (_face_h, face_verts, face_edges) in &face_data {
        let n = face_verts.len();

        let mut mid_verts: Vec<Handle<Vert>> = Vec::new();
        for edge_h in face_edges {
            if let Some(&mid) = edge_midpoints.get(edge_h) {
                mid_verts.push(mid);
            }
        }

        if mid_verts.len() == n {
            // For each corner, create a triangle: (corner_vert, edge_mid_of_outgoing, edge_mid_of_incoming).
            for i in 0..n {
                let prev_mid = mid_verts[(i + n - 1) % n];
                let cur_vert = face_verts[i];
                let next_mid = mid_verts[i];
                let tri = [cur_vert, next_mid, prev_mid];
                mesh.create_face(&tri)?;
            }

            // Create central polygon from midpoints.
            if mid_verts.len() >= 3 {
                mesh.create_face(&mid_verts)?;
            }
        }
    }

    Ok(())
}
