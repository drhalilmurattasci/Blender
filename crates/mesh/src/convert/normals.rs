//! Normal computation utilities.

use forge3d_alloc::{Arena, Handle};
use forge3d_math::Vec3;

use crate::elements::{Edge, Face, LoopElem, Vert};
use crate::cycles::face_loop::FaceLoopIter;

/// Computes the normal vector for a single face using Newell's method.
///
/// The face normal is stored back into `face.normal`.
pub fn compute_face_normal(
    faces: &mut Arena<Face>,
    loops: &Arena<LoopElem>,
    verts: &Arena<Vert>,
    face_h: Handle<Face>,
) {
    let face = match faces.get(face_h) {
        Some(f) => f,
        None => return,
    };
    let first_h = face.loop_first;
    if first_h.is_dangling() {
        return;
    }

    // Collect positions.
    let mut positions = Vec::new();
    let iter = FaceLoopIter::new(faces, loops, face_h);
    for loop_h in iter {
        if let Some(l) = loops.get(loop_h) {
            if let Some(v) = verts.get(l.vert) {
                positions.push(v.co);
            }
        }
    }

    if positions.len() < 3 {
        return;
    }

    // Newell's method for polygon normal.
    let mut normal = Vec3::ZERO;
    let n = positions.len();
    for i in 0..n {
        let cur = positions[i];
        let next = positions[(i + 1) % n];
        normal.x += (cur.y - next.y) * (cur.z + next.z);
        normal.y += (cur.z - next.z) * (cur.x + next.x);
        normal.z += (cur.x - next.x) * (cur.y + next.y);
    }

    let len = normal.length();
    if len > f32::EPSILON {
        normal /= len;
    }

    if let Some(face) = faces.get_mut(face_h) {
        face.normal = normal;
    }
}

/// Computes normals for all faces in the mesh.
pub fn compute_face_normals(
    faces: &mut Arena<Face>,
    loops: &Arena<LoopElem>,
    verts: &Arena<Vert>,
) {
    let face_handles: Vec<Handle<Face>> = faces.iter().map(|(h, _)| h).collect();
    for face_h in face_handles {
        compute_face_normal(faces, loops, verts, face_h);
    }
}

/// Computes vertex normals by averaging adjacent face normals.
///
/// Face normals should be computed first via [`compute_face_normals`].
pub fn compute_vertex_normals(
    verts: &mut Arena<Vert>,
    edges: &Arena<Edge>,
    loops: &Arena<LoopElem>,
    faces: &Arena<Face>,
) {
    // Collect vertex handles first.
    let vert_handles: Vec<Handle<Vert>> = verts.iter().map(|(h, _)| h).collect();

    for vert_h in vert_handles {
        let mut normal = Vec3::ZERO;
        let mut face_count = 0u32;

        // Walk the disk cycle to find adjacent faces.
        let vert = match verts.get(vert_h) {
            Some(v) => v,
            None => continue,
        };

        let first_edge_h = match vert.edge {
            Some(h) => h,
            None => continue,
        };

        let mut cur_edge_h = first_edge_h;
        loop {
            let edge = match edges.get(cur_edge_h) {
                Some(e) => e,
                None => break,
            };

            // Walk radial cycle.
            if let Some(first_loop_h) = edge.loop_first {
                let mut cur_loop_h = first_loop_h;
                loop {
                    let l = match loops.get(cur_loop_h) {
                        Some(l) => l,
                        None => break,
                    };

                    if l.vert == vert_h {
                        if let Some(face) = faces.get(l.face) {
                            normal += face.normal;
                            face_count += 1;
                        }
                    }

                    cur_loop_h = l.radial_next;
                    if cur_loop_h == first_loop_h {
                        break;
                    }
                }
            }

            cur_edge_h = edge.disk_link(vert_h).next;
            if cur_edge_h == first_edge_h {
                break;
            }
        }

        if face_count > 0 {
            let len = normal.length();
            if len > f32::EPSILON {
                normal /= len;
            }
            if let Some(v) = verts.get_mut(vert_h) {
                v.normal = normal;
            }
        }
    }
}
