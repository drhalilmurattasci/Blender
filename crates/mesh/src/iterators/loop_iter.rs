//! Iterator over the loops forming a face boundary.

use forge3d_alloc::{Arena, Handle};

use crate::elements::{Face, LoopElem};
use crate::cycles::face_loop::FaceLoopIter;

/// Iterator over all loop handles belonging to a face.
pub struct FaceLoops<'a> {
    inner: FaceLoopIter<'a>,
}

impl<'a> FaceLoops<'a> {
    /// Creates an iterator over the loops of `face_h`.
    pub fn new(
        faces: &Arena<Face>,
        loops: &'a Arena<LoopElem>,
        face_h: Handle<Face>,
    ) -> Self {
        Self {
            inner: FaceLoopIter::new(faces, loops, face_h),
        }
    }
}

impl<'a> Iterator for FaceLoops<'a> {
    type Item = Handle<LoopElem>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
