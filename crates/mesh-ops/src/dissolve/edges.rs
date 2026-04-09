//! Dissolve edges — merges adjacent faces sharing the edge.

use forge3d_alloc::Handle;
use forge3d_mesh::{Edge, Face, Mesh, Vert};

use crate::common::{OpError, OpResult, SelectionHelper};

/// Dissolves selected edges, merging the faces on either side.
pub fn dissolve_edges(mesh: &mut Mesh) -> OpResult<()> {
    let selected = SelectionHelper::selected_edges(mesh);
    if selected.is_empty() {
        return Err(OpError::InvalidSelection("no edges selected".into()));
    }

    for &edge_h in &selected {
        if mesh.edges.contains(edge_h) {
            dissolve_single_edge(mesh, edge_h)?;
        }
    }

    Ok(())
}

fn dissolve_single_edge(mesh: &mut Mesh, edge_h: Handle<Edge>) -> OpResult<()> {
    let edge = mesh.edges.get(edge_h).ok_or_else(|| {
        OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", edge_h)))
    })?;

    // Collect adjacent faces via radial cycle.
    let mut adj_faces: Vec<Handle<Face>> = Vec::new();
    if let Some(first_lh) = edge.loop_first {
        let mut cur_lh = first_lh;
        loop {
            let l = mesh.loops.get(cur_lh).unwrap();
            if !adj_faces.contains(&l.face) {
                adj_faces.push(l.face);
            }
            cur_lh = l.radial_next;
            if cur_lh == first_lh {
                break;
            }
        }
    }

    if adj_faces.len() != 2 {
        if adj_faces.is_empty() {
            // Wire edge -- just remove it.
            mesh.kill_edge(edge_h)?;
            return Ok(());
        }
        return Ok(());
    }

    let v1 = edge.v1;
    let v2 = edge.v2;

    // Collect ordered vertex loops for both faces.
    let verts_a = collect_face_verts(mesh, adj_faces[0])?;
    let verts_b = collect_face_verts(mesh, adj_faces[1])?;

    // Build the merged vertex loop by walking face A and, at the shared edge,
    // splicing in the non-shared portion of face B in correct winding order.
    //
    // The shared edge in face A goes from some vertex to the next. We find
    // the two shared vertices in face A's winding, and at that splice point
    // we insert face B's verts that are NOT on the shared edge (in face B's
    // reverse winding across the shared edge so the orientations match).
    let merged = merge_face_verts(&verts_a, &verts_b, v1, v2);

    // Kill both faces and the shared edge.
    mesh.kill_face(adj_faces[0])?;
    mesh.kill_face(adj_faces[1])?;
    mesh.kill_edge(edge_h)?;

    // Create the merged face.
    if merged.len() >= 3 {
        mesh.create_face(&merged)?;
    }

    Ok(())
}

/// Merges two face vertex loops that share an edge (v1-v2) into a single
/// combined loop with correct winding order.
fn merge_face_verts(
    verts_a: &[Handle<Vert>],
    verts_b: &[Handle<Vert>],
    v1: Handle<Vert>,
    v2: Handle<Vert>,
) -> Vec<Handle<Vert>> {
    let na = verts_a.len();
    let nb = verts_b.len();

    // Find the index in face A where the shared edge starts.
    // The shared edge goes from verts_a[i] to verts_a[i+1] for some i,
    // where {verts_a[i], verts_a[i+1]} == {v1, v2}.
    let mut shared_start_a = None;
    for i in 0..na {
        let cur = verts_a[i];
        let next = verts_a[(i + 1) % na];
        if (cur == v1 && next == v2) || (cur == v2 && next == v1) {
            shared_start_a = Some(i);
            break;
        }
    }
    let shared_start_a = match shared_start_a {
        Some(i) => i,
        None => {
            // Fallback: just concatenate unique verts (shouldn't happen).
            let mut result = verts_a.to_vec();
            for &vh in verts_b {
                if !result.contains(&vh) {
                    result.push(vh);
                }
            }
            return result;
        }
    };

    // In face A, the shared edge goes from verts_a[shared_start_a] to
    // verts_a[shared_start_a + 1]. We keep all of face A's verts EXCEPT
    // the transition across the shared edge, where we splice in face B's verts.
    let a_shared_v0 = verts_a[shared_start_a];
    let a_shared_v1 = verts_a[(shared_start_a + 1) % na];

    // Find the corresponding edge in face B (it will be in reverse direction).
    // face B has the same edge but traversed as a_shared_v1 -> a_shared_v0.
    let mut shared_start_b = None;
    for i in 0..nb {
        let cur = verts_b[i];
        let next = verts_b[(i + 1) % nb];
        if cur == a_shared_v1 && next == a_shared_v0 {
            shared_start_b = Some(i);
            break;
        }
    }
    let shared_start_b = match shared_start_b {
        Some(i) => i,
        None => {
            // The faces might have same winding across the edge (non-manifold).
            // Fallback: concatenate unique verts.
            let mut result = verts_a.to_vec();
            for &vh in verts_b {
                if !result.contains(&vh) {
                    result.push(vh);
                }
            }
            return result;
        }
    };

    // Build merged loop:
    // - Walk face A from after the shared edge end, around to the shared edge start.
    // - At the splice point, walk face B from after its shared edge end, around to
    //   its shared edge start (skipping the two shared verts).
    let mut merged = Vec::with_capacity(na + nb - 2);

    // Add face A verts: from (shared_start_a+1) all the way to shared_start_a (inclusive).
    // This is the portion of face A that is NOT the shared edge transition.
    // We go: a_shared_v1, ..., a_shared_v0.
    for k in 0..na {
        let idx = (shared_start_a + 1 + k) % na;
        merged.push(verts_a[idx]);
        // Stop before we re-reach a_shared_v1 (we've added a_shared_v0 as the last one).
        if idx == shared_start_a {
            break;
        }
    }
    // merged is now [a_shared_v1, ..., a_shared_v0].
    // We need to splice face B's non-shared verts between a_shared_v0 and a_shared_v1.
    // Face B goes: ..., a_shared_v1, [non-shared verts...], a_shared_v0, ...
    // Its shared edge starts at shared_start_b: verts_b[shared_start_b] = a_shared_v1,
    // verts_b[shared_start_b+1] = a_shared_v0.
    // The non-shared portion of B runs from (shared_start_b+2) to (shared_start_b-1).

    // Remove a_shared_v0 from end of merged (it will be part of the splice).
    // Actually, we keep a_shared_v0 and just insert B's non-shared verts after it,
    // then a_shared_v1 is already at the start. So we DON'T add a_shared_v1 again.

    // Insert face B's non-shared vertices: from (shared_start_b+2) to (shared_start_b-1) inclusive.
    let b_non_shared_count = nb - 2;
    for k in 0..b_non_shared_count {
        let idx = (shared_start_b + 2 + k) % nb;
        merged.push(verts_b[idx]);
    }

    // Remove the duplicate: merged starts with a_shared_v1 and we don't want it
    // repeated. Actually we need to check: the last B vert leads back to a_shared_v1
    // which is already at position 0 of merged. That's fine for a closed polygon.

    merged
}

/// Collects the ordered vertex handles of a face.
fn collect_face_verts(mesh: &Mesh, face_h: Handle<Face>) -> OpResult<Vec<Handle<Vert>>> {
    let face = mesh.faces.get(face_h).ok_or_else(|| {
        OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", face_h)))
    })?;
    let first_lh = face.loop_first;
    if first_lh.is_dangling() {
        return Ok(Vec::new());
    }

    let mut verts = Vec::new();
    let mut cur_lh = first_lh;
    loop {
        let l = mesh.loops.get(cur_lh).ok_or_else(|| {
            OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", cur_lh)))
        })?;
        verts.push(l.vert);
        cur_lh = l.next;
        if cur_lh == first_lh {
            break;
        }
    }
    Ok(verts)
}
