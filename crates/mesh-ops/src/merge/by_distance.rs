//! Merge by distance — welds vertices closer than a threshold.

use forge3d_alloc::Handle;
use forge3d_mesh::{Face, Mesh, Vert};

use crate::common::{OpError, OpResult};
use super::MergeParams;

/// Merges vertices that are within `threshold` distance of each other.
///
/// For each cluster of close vertices, all are replaced by a single vertex
/// at their centroid. Faces referencing merged vertices are rebuilt with the
/// representative vertex. Degenerate faces (where merging collapses edges)
/// are removed.
pub fn merge_by_distance(mesh: &mut Mesh, params: &MergeParams) -> OpResult<()> {
    let threshold = params.threshold;
    let threshold_sq = threshold * threshold;

    if threshold <= 0.0 {
        return Err(OpError::InvalidParam(
            "threshold must be positive".into(),
        ));
    }

    // Collect all vertices.
    let vert_data: Vec<(Handle<Vert>, forge3d_math::Vec3)> = mesh
        .verts
        .iter()
        .map(|(h, v)| (h, v.co))
        .collect();

    // Naive O(n^2) clustering. A real implementation would use a spatial hash or k-d tree.
    let n = vert_data.len();
    let mut merged_to: Vec<Option<usize>> = vec![None; n]; // index -> cluster representative index

    for i in 0..n {
        if merged_to[i].is_some() {
            continue;
        }
        merged_to[i] = Some(i);

        for j in (i + 1)..n {
            if merged_to[j].is_some() {
                continue;
            }
            let dist_sq = (vert_data[i].1 - vert_data[j].1).length_squared();
            if dist_sq <= threshold_sq {
                merged_to[j] = Some(i);
            }
        }
    }

    // Build a map from vertex handle -> representative vertex handle.
    let mut vert_remap: std::collections::HashMap<Handle<Vert>, Handle<Vert>> =
        std::collections::HashMap::new();
    let mut clusters: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();

    for (idx, rep) in merged_to.iter().enumerate() {
        if let Some(r) = rep {
            clusters.entry(*r).or_default().push(idx);
            if idx != *r {
                vert_remap.insert(vert_data[idx].0, vert_data[*r].0);
            }
        }
    }

    // If nothing to merge, bail early.
    if vert_remap.is_empty() {
        return Ok(());
    }

    // Move representative vertices to cluster centroids.
    for (rep_idx, members) in &clusters {
        if members.len() <= 1 {
            continue;
        }

        let mut sum = forge3d_math::Vec3::ZERO;
        for &idx in members {
            sum += vert_data[idx].1;
        }
        let centroid = sum / members.len() as f32;

        let rep_h = vert_data[*rep_idx].0;
        if let Some(v) = mesh.verts.get_mut(rep_h) {
            v.co = centroid;
        }
    }

    // Collect all faces and their vertex lists before modifying topology.
    let face_data: Vec<(Handle<Face>, Vec<Handle<Vert>>)> = {
        let face_handles: Vec<Handle<Face>> = mesh.faces.iter().map(|(h, _)| h).collect();
        let mut data = Vec::new();
        for fh in face_handles {
            let face = mesh.faces.get(fh).unwrap();
            let first_lh = face.loop_first;
            if first_lh.is_dangling() {
                continue;
            }
            let mut verts = Vec::new();
            let mut cur_lh = first_lh;
            loop {
                let l = mesh.loops.get(cur_lh).unwrap();
                verts.push(l.vert);
                cur_lh = l.next;
                if cur_lh == first_lh {
                    break;
                }
            }
            data.push((fh, verts));
        }
        data
    };

    // Kill all existing faces (we will rebuild them with remapped vertices).
    for (fh, _) in &face_data {
        if mesh.faces.contains(*fh) {
            mesh.kill_face(*fh)?;
        }
    }

    // Kill all edges (they may reference merged-away vertices).
    let edge_handles: Vec<Handle<forge3d_mesh::Edge>> =
        mesh.edges.iter().map(|(h, _)| h).collect();
    for eh in edge_handles {
        if mesh.edges.contains(eh) {
            let _ = mesh.kill_edge(eh);
        }
    }

    // Kill non-representative vertices.
    for (&old_h, _) in &vert_remap {
        if mesh.verts.contains(old_h) {
            mesh.verts.remove(old_h);
        }
    }

    // Rebuild faces with remapped vertices, skipping degenerate ones.
    for (_fh, orig_verts) in &face_data {
        let remapped: Vec<Handle<Vert>> = orig_verts
            .iter()
            .map(|vh| vert_remap.get(vh).copied().unwrap_or(*vh))
            .collect();

        // Remove consecutive duplicates (collapsed edges).
        let mut deduped: Vec<Handle<Vert>> = Vec::with_capacity(remapped.len());
        for &vh in &remapped {
            if deduped.last() != Some(&vh) {
                deduped.push(vh);
            }
        }
        // Also check wrap-around: first == last.
        if deduped.len() > 1 && deduped.first() == deduped.last() {
            deduped.pop();
        }

        // Check for any remaining non-consecutive duplicates.
        let mut seen = std::collections::HashSet::new();
        let mut unique = Vec::with_capacity(deduped.len());
        for &vh in &deduped {
            if seen.insert(vh) {
                unique.push(vh);
            }
        }

        if unique.len() >= 3 {
            let _ = mesh.create_face(&unique);
        }
    }

    Ok(())
}
