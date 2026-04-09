//! Fan triangulation — converts n-gon faces into triangle fans.

use forge3d_alloc::{Arena, Handle};

use crate::elements::{Face, LoopElem, Vert};
use crate::cycles::face_loop::FaceLoopIter;

/// A single output triangle, represented as three vertex handles.
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub v0: Handle<Vert>,
    pub v1: Handle<Vert>,
    pub v2: Handle<Vert>,
}

/// Triangulates a single face using a fan from the first vertex.
///
/// For a face with vertices `[a, b, c, d, ...]` this produces triangles:
/// `(a, b, c)`, `(a, c, d)`, ...
pub fn triangulate_face_fan(
    faces: &Arena<Face>,
    loops: &Arena<LoopElem>,
    face_h: Handle<Face>,
) -> Vec<Triangle> {
    let mut vert_handles = Vec::new();
    let iter = FaceLoopIter::new(faces, loops, face_h);
    for loop_h in iter {
        if let Some(l) = loops.get(loop_h) {
            vert_handles.push(l.vert);
        }
    }

    if vert_handles.len() < 3 {
        return Vec::new();
    }

    let mut triangles = Vec::with_capacity(vert_handles.len() - 2);
    let pivot = vert_handles[0];
    for i in 1..vert_handles.len() - 1 {
        triangles.push(Triangle {
            v0: pivot,
            v1: vert_handles[i],
            v2: vert_handles[i + 1],
        });
    }
    triangles
}

/// Triangulates all faces in the mesh using fan triangulation.
/// Returns a flat list of triangles.
pub fn triangulate_all_fan(
    faces: &Arena<Face>,
    loops: &Arena<LoopElem>,
) -> Vec<Triangle> {
    let mut all_tris = Vec::new();
    for (fh, _) in faces.iter() {
        all_tris.extend(triangulate_face_fan(faces, loops, fh));
    }
    all_tris
}
