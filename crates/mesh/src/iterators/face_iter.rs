//! Iterators from a face to its vertices and edges via the face loop cycle.

use forge3d_alloc::{Arena, Handle};

use crate::elements::{Edge, Face, LoopElem, Vert};
use crate::cycles::face_loop::FaceLoopIter;

/// Iterator over all vertices of a face (in winding order).
pub struct FaceVerts<'a> {
    loops: &'a Arena<LoopElem>,
    inner: FaceLoopIter<'a>,
}

impl<'a> FaceVerts<'a> {
    /// Creates an iterator over the vertices of `face_h`.
    pub fn new(
        faces: &Arena<Face>,
        loops: &'a Arena<LoopElem>,
        face_h: Handle<Face>,
    ) -> Self {
        Self {
            loops,
            inner: FaceLoopIter::new(faces, loops, face_h),
        }
    }
}

impl<'a> Iterator for FaceVerts<'a> {
    type Item = Handle<Vert>;

    fn next(&mut self) -> Option<Self::Item> {
        let loop_h = self.inner.next()?;
        let l = self.loops.get(loop_h)?;
        Some(l.vert)
    }
}

/// Iterator over all edges of a face (in winding order).
pub struct FaceEdges<'a> {
    loops: &'a Arena<LoopElem>,
    inner: FaceLoopIter<'a>,
}

impl<'a> FaceEdges<'a> {
    /// Creates an iterator over the edges of `face_h`.
    pub fn new(
        faces: &Arena<Face>,
        loops: &'a Arena<LoopElem>,
        face_h: Handle<Face>,
    ) -> Self {
        Self {
            loops,
            inner: FaceLoopIter::new(faces, loops, face_h),
        }
    }
}

impl<'a> Iterator for FaceEdges<'a> {
    type Item = Handle<Edge>;

    fn next(&mut self) -> Option<Self::Item> {
        let loop_h = self.inner.next()?;
        let l = self.loops.get(loop_h)?;
        Some(l.edge)
    }
}
