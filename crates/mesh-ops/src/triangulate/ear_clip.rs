//! Ear clipping triangulation for simple polygons.

use forge3d_alloc::Handle;
use forge3d_math::Vec3;
use forge3d_mesh::{Face, Mesh, Vert};

use crate::common::{OpError, OpResult};

/// A triangle produced by ear clipping, as three vertex handles.
#[derive(Debug, Clone, Copy)]
pub struct EarClipTriangle {
    pub v0: Handle<Vert>,
    pub v1: Handle<Vert>,
    pub v2: Handle<Vert>,
}

/// Triangulates a face using the ear clipping algorithm.
///
/// Works for simple (non-self-intersecting) polygons. Falls back to fan
/// triangulation for degenerate cases.
pub fn ear_clip_face(mesh: &Mesh, face_h: Handle<Face>) -> OpResult<Vec<EarClipTriangle>> {
    let face = mesh.faces.get(face_h).ok_or_else(|| {
        OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", face_h)))
    })?;

    let first_lh = face.loop_first;
    if first_lh.is_dangling() {
        return Ok(Vec::new());
    }

    // Collect vertices and positions.
    let mut vert_handles = Vec::new();
    let mut positions = Vec::new();

    let mut cur_lh = first_lh;
    loop {
        let l = mesh.loops.get(cur_lh).unwrap();
        vert_handles.push(l.vert);
        positions.push(mesh.verts.get(l.vert).unwrap().co);
        cur_lh = l.next;
        if cur_lh == first_lh {
            break;
        }
    }

    let n = vert_handles.len();
    if n < 3 {
        return Ok(Vec::new());
    }
    if n == 3 {
        return Ok(vec![EarClipTriangle {
            v0: vert_handles[0],
            v1: vert_handles[1],
            v2: vert_handles[2],
        }]);
    }

    // Use the stored face normal, but if it is zero (not yet computed),
    // compute it on the fly using Newell's method.
    let mut normal = face.normal;
    if normal.length_squared() < f32::EPSILON {
        let mut n_calc = Vec3::ZERO;
        for i in 0..n {
            let cur = positions[i];
            let next = positions[(i + 1) % n];
            n_calc.x += (cur.y - next.y) * (cur.z + next.z);
            n_calc.y += (cur.z - next.z) * (cur.x + next.x);
            n_calc.z += (cur.x - next.x) * (cur.y + next.y);
        }
        let len = n_calc.length();
        if len > f32::EPSILON {
            normal = n_calc / len;
        } else {
            // Degenerate polygon -- all vertices are collinear.
            return Err(OpError::GeometryError("degenerate polygon, cannot compute normal for ear clipping".into()));
        }
    }

    // Ear clipping on a mutable index list.
    let mut indices: Vec<usize> = (0..n).collect();
    let mut triangles = Vec::with_capacity(n - 2);

    let mut safety = n * n; // Prevent infinite loops.

    while indices.len() > 3 && safety > 0 {
        safety -= 1;
        let len = indices.len();
        let mut found_ear = false;

        for i in 0..len {
            let prev = indices[(i + len - 1) % len];
            let curr = indices[i];
            let next = indices[(i + 1) % len];

            let a = positions[prev];
            let b = positions[curr];
            let c = positions[next];

            // Check if this is a convex vertex.
            let cross = (b - a).cross(c - b);
            if cross.dot(normal) < 0.0 {
                continue; // Reflex vertex.
            }

            // Check no other vertex is inside triangle abc.
            let mut is_ear = true;
            for j in 0..len {
                let idx = indices[j];
                if idx == prev || idx == curr || idx == next {
                    continue;
                }
                if point_in_triangle(positions[idx], a, b, c, normal) {
                    is_ear = false;
                    break;
                }
            }

            if is_ear {
                triangles.push(EarClipTriangle {
                    v0: vert_handles[prev],
                    v1: vert_handles[curr],
                    v2: vert_handles[next],
                });
                indices.remove(i);
                found_ear = true;
                break;
            }
        }

        if !found_ear {
            break; // Degenerate polygon.
        }
    }

    // Final triangle.
    if indices.len() == 3 {
        triangles.push(EarClipTriangle {
            v0: vert_handles[indices[0]],
            v1: vert_handles[indices[1]],
            v2: vert_handles[indices[2]],
        });
    }

    Ok(triangles)
}

/// Tests whether point P lies inside triangle ABC (projected onto the plane
/// defined by `normal`).
fn point_in_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3, normal: Vec3) -> bool {
    let cross0 = (b - a).cross(p - a).dot(normal);
    let cross1 = (c - b).cross(p - b).dot(normal);
    let cross2 = (a - c).cross(p - c).dot(normal);
    (cross0 >= 0.0 && cross1 >= 0.0 && cross2 >= 0.0)
        || (cross0 <= 0.0 && cross1 <= 0.0 && cross2 <= 0.0)
}
