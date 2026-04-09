//! Catmull-Clark subdivision surface.

use std::collections::HashMap;

use forge3d_alloc::Handle;
use forge3d_math::Vec3;
use forge3d_mesh::{Edge, Face, Mesh, Vert};

use crate::common::OpResult;
use super::CatmullClarkParams;

/// Applies one iteration of Catmull-Clark subdivision.
///
/// Steps:
/// 1. Compute a face point for each face (centroid).
/// 2. Compute an edge point for each edge (avg of endpoints + adjacent face points).
/// 3. Move original vertices toward the average of adjacent face and edge points.
/// 4. Rebuild topology by connecting face points, edge points, and moved vertices.
pub fn catmull_clark(mesh: &mut Mesh, params: &CatmullClarkParams) -> OpResult<()> {
    for _ in 0..params.iterations {
        catmull_clark_single(mesh)?;
    }
    Ok(())
}

/// Per-face topology snapshot taken before we destroy the original mesh.
struct FaceRecord {
    /// Ordered vertex handles around the face boundary.
    verts: Vec<Handle<Vert>>,
    /// Ordered edge handles around the face boundary (edge[i] connects vert[i]->vert[i+1]).
    edges: Vec<Handle<Edge>>,
}

fn catmull_clark_single(mesh: &mut Mesh) -> OpResult<()> {
    // --- Step 0: Snapshot old topology before any mutations ---
    let face_handles: Vec<Handle<Face>> = mesh.faces.iter().map(|(h, _)| h).collect();
    let edge_handles: Vec<(Handle<Edge>, Handle<Vert>, Handle<Vert>)> = mesh
        .edges
        .iter()
        .map(|(h, e)| (h, e.v1, e.v2))
        .collect();

    // Record per-face vertex and edge ordering (needed for Step 4).
    let mut face_records: HashMap<Handle<Face>, FaceRecord> = HashMap::new();
    for &fh in &face_handles {
        let face = mesh.faces.get(fh).unwrap();
        let first_lh = face.loop_first;
        if first_lh.is_dangling() {
            continue;
        }
        let mut verts = Vec::new();
        let mut edges = Vec::new();
        let mut cur_lh = first_lh;
        loop {
            let l = mesh.loops.get(cur_lh).unwrap();
            verts.push(l.vert);
            edges.push(l.edge);
            cur_lh = l.next;
            if cur_lh == first_lh {
                break;
            }
        }
        face_records.insert(fh, FaceRecord { verts, edges });
    }

    // --- Step 1: Face points (centroids) ---
    let mut face_point_map: HashMap<Handle<Face>, Handle<Vert>> = HashMap::new();

    for &fh in &face_handles {
        let record = match face_records.get(&fh) {
            Some(r) => r,
            None => continue,
        };
        let mut sum = Vec3::ZERO;
        let mut count = 0u32;
        for &vh in &record.verts {
            if let Some(v) = mesh.verts.get(vh) {
                sum += v.co;
                count += 1;
            }
        }
        if count > 0 {
            let centroid = sum / count as f32;
            let fp = mesh.create_vert(centroid);
            face_point_map.insert(fh, fp);
        }
    }

    // --- Step 2: Edge points ---
    let mut edge_point_map: HashMap<Handle<Edge>, Handle<Vert>> = HashMap::new();

    for &(eh, v1_h, v2_h) in &edge_handles {
        let co1 = mesh.verts.get(v1_h).map(|v| v.co).unwrap_or(Vec3::ZERO);
        let co2 = mesh.verts.get(v2_h).map(|v| v.co).unwrap_or(Vec3::ZERO);

        // Average of adjacent face points.
        let mut fp_sum = Vec3::ZERO;
        let mut fp_count = 0u32;

        let edge = mesh.edges.get(eh).unwrap();
        if let Some(first_lh) = edge.loop_first {
            let mut cur_lh = first_lh;
            loop {
                let l = mesh.loops.get(cur_lh).unwrap();
                if let Some(&fp_h) = face_point_map.get(&l.face) {
                    if let Some(fp_v) = mesh.verts.get(fp_h) {
                        fp_sum += fp_v.co;
                        fp_count += 1;
                    }
                }
                cur_lh = l.radial_next;
                if cur_lh == first_lh {
                    break;
                }
            }
        }

        let edge_pt = if fp_count > 0 {
            (co1 + co2 + fp_sum) / (2.0 + fp_count as f32)
        } else {
            (co1 + co2) * 0.5
        };

        let ep = mesh.create_vert(edge_pt);
        edge_point_map.insert(eh, ep);
    }

    // --- Step 3: Move original vertices ---
    let new_point_set: std::collections::HashSet<Handle<Vert>> = face_point_map
        .values()
        .copied()
        .chain(edge_point_map.values().copied())
        .collect();

    let orig_vert_handles: Vec<Handle<Vert>> = mesh
        .verts
        .iter()
        .map(|(h, _)| h)
        .filter(|h| !new_point_set.contains(h))
        .collect();

    for &vh in &orig_vert_handles {
        let vert = match mesh.verts.get(vh) {
            Some(v) => v,
            None => continue,
        };
        let orig_co = vert.co;

        let mut f_avg = Vec3::ZERO;
        let mut f_count = 0u32;
        let mut r_avg = Vec3::ZERO;
        let mut r_count = 0u32;

        let first_eh = match vert.edge {
            Some(h) => h,
            None => continue,
        };

        let mut cur_eh = first_eh;
        loop {
            let edge = match mesh.edges.get(cur_eh) {
                Some(e) => e,
                None => break,
            };

            // Edge midpoint (original edge midpoint, not the edge point).
            let other_h = edge.other_vert(vh);
            if let Some(other) = mesh.verts.get(other_h) {
                r_avg += (orig_co + other.co) * 0.5;
                r_count += 1;
            }

            // Adjacent face points via radial cycle.
            if let Some(first_lh) = edge.loop_first {
                let mut cur_lh = first_lh;
                loop {
                    let l = mesh.loops.get(cur_lh).unwrap();
                    if l.vert == vh {
                        if let Some(&fp_h) = face_point_map.get(&l.face) {
                            if let Some(fp_v) = mesh.verts.get(fp_h) {
                                f_avg += fp_v.co;
                                f_count += 1;
                            }
                        }
                    }
                    cur_lh = l.radial_next;
                    if cur_lh == first_lh {
                        break;
                    }
                }
            }

            cur_eh = edge.disk_link(vh).next;
            if cur_eh == first_eh {
                break;
            }
        }

        if f_count > 0 && r_count > 0 {
            let n = f_count as f32;
            let f = f_avg / n;
            let r = r_avg / r_count as f32;
            let new_co = (f + 2.0 * r + (n - 3.0) * orig_co) / n;

            if let Some(v) = mesh.verts.get_mut(vh) {
                v.co = new_co;
            }
        }
    }

    // --- Step 4: Rebuild topology ---
    // Kill old faces first (removes loops and radial links but keeps edges/verts).
    for &fh in &face_handles {
        if mesh.faces.contains(fh) {
            mesh.kill_face(fh)?;
        }
    }

    // Kill old edges (faces are already gone so no radial links remain).
    for &(eh, _, _) in &edge_handles {
        if mesh.edges.contains(eh) {
            let _ = mesh.kill_edge(eh);
        }
    }

    // For each original face, create one quad per corner:
    //   (orig_vert[i], edge_point[i], face_point, edge_point[i-1])
    // where edge[i] connects vert[i] -> vert[i+1].
    for (&fh, record) in &face_records {
        let fp_h = match face_point_map.get(&fh) {
            Some(&h) => h,
            None => continue,
        };

        let n = record.verts.len();
        for i in 0..n {
            let prev_edge_idx = (i + n - 1) % n;
            let vert_h = record.verts[i];
            let ep_next = match edge_point_map.get(&record.edges[i]) {
                Some(&h) => h,
                None => continue,
            };
            let ep_prev = match edge_point_map.get(&record.edges[prev_edge_idx]) {
                Some(&h) => h,
                None => continue,
            };

            // Quad: orig_vert -> edge_point_of_outgoing_edge -> face_point -> edge_point_of_incoming_edge
            let quad = [vert_h, ep_next, fp_h, ep_prev];
            mesh.create_face(&quad)?;
        }
    }

    Ok(())
}
