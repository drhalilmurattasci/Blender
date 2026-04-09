//! Face loop cycle iteration — the ordered boundary of a face.

use forge3d_alloc::{Arena, Handle};

use crate::elements::{Face, LoopElem};

/// Returns an iterator over the loop handles forming the boundary of `face_h`.
pub fn face_loop_iter<'a>(
    faces: &Arena<Face>,
    loops: &'a Arena<LoopElem>,
    face_h: Handle<Face>,
) -> FaceLoopIter<'a> {
    FaceLoopIter::new(faces, loops, face_h)
}

/// Iterator over all loops in a face's boundary cycle.
pub struct FaceLoopIter<'a> {
    loops: &'a Arena<LoopElem>,
    first_h: Handle<LoopElem>,
    current_h: Handle<LoopElem>,
    done: bool,
    /// Safety guard to prevent infinite loops on corrupt cycles.
    remaining: usize,
}

/// Maximum number of iterations before bailing out (infinite loop guard).
const MAX_FACE_LOOP_ITER: usize = 1_000_000;

impl<'a> FaceLoopIter<'a> {
    /// Creates a new face loop iterator for the given face.
    pub fn new(
        faces: &Arena<Face>,
        loops: &'a Arena<LoopElem>,
        face_h: Handle<Face>,
    ) -> Self {
        let (first_h, max_len) = faces
            .get(face_h)
            .map(|f| (f.loop_first, f.len as usize))
            .unwrap_or((Handle::dangling(), 0));

        Self {
            loops,
            first_h,
            current_h: first_h,
            done: first_h.is_dangling(),
            // Use face.len as the expected count, but cap at a safety maximum.
            remaining: if max_len > 0 { max_len } else { MAX_FACE_LOOP_ITER },
        }
    }
}

impl<'a> Iterator for FaceLoopIter<'a> {
    type Item = Handle<LoopElem>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done || self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let result = self.current_h;
        let cur = self.loops.get(self.current_h)?;
        self.current_h = cur.next;
        if self.current_h == self.first_h {
            self.done = true;
        }
        Some(result)
    }
}
