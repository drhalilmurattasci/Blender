//! Bisect with a plane — cuts the mesh along a plane, splitting faces.

use forge3d_alloc::Handle;
use forge3d_mesh::{Face, Mesh, Vert};

use crate::common::{OpError, OpResult};
use super::BisectParams;

/// Bisects the mesh with a plane defined by a point and normal.
///
/// Faces that straddle the plane are split. Optionally, geometry on one side
/// of the plane can be removed.
pub fn plane_cut(mesh: &mut Mesh, params: &BisectParams) -> OpResult<()> {
    let plane_co = params.plane_co;
    let plane_no = params.plane_no.normalize_or_zero();

    if plane_no.length_squared() < f32::EPSILON {
        return Err(OpError::InvalidParam("plane normal is zero".into()));
    }

    // Classify each vertex as above (+), below (-), or on the plane.
    let vert_data: Vec<(Handle<Vert>, f32)> = mesh
        .verts
        .iter()
        .map(|(h, v)| {
            let dist = (v.co - plane_co).dot(plane_no);
            (h, dist)
        })
        .collect();

    let classify = |vh: Handle<Vert>| -> f32 {
        vert_data
            .iter()
            .find(|(h, _)| *h == vh)
            .map(|(_, d)| *d)
            .unwrap_or(0.0)
    };

    // Collect faces that straddle the plane.
    let face_handles: Vec<Handle<Face>> = mesh.faces.iter().map(|(h, _)| h).collect();

    for face_h in face_handles {
        if !mesh.faces.contains(face_h) {
            continue;
        }

        let face = mesh.faces.get(face_h).unwrap();
        let first_lh = face.loop_first;
        if first_lh.is_dangling() {
            continue;
        }

        // Collect face vertices and their signed distances.
        let mut face_verts: Vec<(Handle<Vert>, f32)> = Vec::new();
        let mut cur_lh = first_lh;
        loop {
            let l = mesh.loops.get(cur_lh).unwrap();
            face_verts.push((l.vert, classify(l.vert)));
            cur_lh = l.next;
            if cur_lh == first_lh {
                break;
            }
        }

        // Check if this face straddles the plane.
        let has_above = face_verts.iter().any(|(_, d)| *d > params.epsilon);
        let has_below = face_verts.iter().any(|(_, d)| *d < -params.epsilon);

        if !has_above || !has_below {
            // Face is entirely on one side — optionally remove.
            if params.clear_outer && !has_below {
                mesh.kill_face(face_h)?;
            } else if params.clear_inner && !has_above {
                mesh.kill_face(face_h)?;
            }
            continue;
        }

        // Split the face along the plane.
        let n = face_verts.len();
        let mut above_verts: Vec<Handle<Vert>> = Vec::new();
        let mut below_verts: Vec<Handle<Vert>> = Vec::new();

        for i in 0..n {
            let (v_cur, d_cur) = face_verts[i];
            let (v_next, d_next) = face_verts[(i + 1) % n];

            if d_cur >= -params.epsilon {
                above_verts.push(v_cur);
            }
            if d_cur <= params.epsilon {
                below_verts.push(v_cur);
            }

            // Check if the edge crosses the plane.
            if (d_cur > params.epsilon && d_next < -params.epsilon)
                || (d_cur < -params.epsilon && d_next > params.epsilon)
            {
                // Interpolate to find the intersection point.
                let co_cur = mesh.verts.get(v_cur).unwrap().co;
                let co_next = mesh.verts.get(v_next).unwrap().co;
                let t = d_cur / (d_cur - d_next);
                let intersection = co_cur + (co_next - co_cur) * t;
                let split_h = mesh.create_vert(intersection);

                above_verts.push(split_h);
                below_verts.push(split_h);
            }
        }

        // Kill original face.
        mesh.kill_face(face_h)?;

        // Create new faces for each side.
        if above_verts.len() >= 3 && !params.clear_outer {
            let _ = mesh.create_face(&above_verts);
        }
        if below_verts.len() >= 3 && !params.clear_inner {
            let _ = mesh.create_face(&below_verts);
        }
    }

    Ok(())
}
