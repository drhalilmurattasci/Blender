//! Iterators walking from a vertex to its neighboring edges and faces.

use forge3d_alloc::{Arena, Handle};

use crate::elements::{Edge, Face, LoopElem, Vert};
use crate::cycles::disk::DiskIter;

/// Iterator over all edges connected to a vertex (disk cycle).
pub struct VertEdges<'a> {
    inner: DiskIter<'a>,
}

impl<'a> VertEdges<'a> {
    /// Creates an iterator over edges of `vert_h`.
    pub fn new(
        verts: &Arena<Vert>,
        edges: &'a Arena<Edge>,
        vert_h: Handle<Vert>,
    ) -> Self {
        Self {
            inner: DiskIter::new(verts, edges, vert_h),
        }
    }
}

impl<'a> Iterator for VertEdges<'a> {
    type Item = Handle<Edge>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Iterator over all faces connected to a vertex.
///
/// Walks the disk cycle, and for each edge walks the radial cycle to collect
/// faces that reference `vert_h`.
pub struct VertFaces<'a> {
    edges: &'a Arena<Edge>,
    loops: &'a Arena<LoopElem>,
    vert_h: Handle<Vert>,
    disk_iter: DiskIter<'a>,
    /// Current edge's radial iteration state.
    radial_first: Handle<LoopElem>,
    radial_current: Handle<LoopElem>,
    radial_done: bool,
}

impl<'a> VertFaces<'a> {
    /// Creates an iterator over faces adjacent to `vert_h`.
    pub fn new(
        verts: &Arena<Vert>,
        edges: &'a Arena<Edge>,
        loops: &'a Arena<LoopElem>,
        vert_h: Handle<Vert>,
    ) -> Self {
        let mut disk_iter = DiskIter::new(verts, edges, vert_h);
        let (radial_first, radial_current, radial_done) =
            Self::advance_disk(&mut disk_iter, edges);

        Self {
            edges,
            loops,
            vert_h,
            disk_iter,
            radial_first,
            radial_current,
            radial_done,
        }
    }

    fn advance_disk(
        disk_iter: &mut DiskIter<'a>,
        edges: &Arena<Edge>,
    ) -> (Handle<LoopElem>, Handle<LoopElem>, bool) {
        while let Some(edge_h) = disk_iter.next() {
            if let Some(edge) = edges.get(edge_h) {
                if let Some(first_h) = edge.loop_first {
                    return (first_h, first_h, false);
                }
            }
        }
        (Handle::dangling(), Handle::dangling(), true)
    }
}

impl<'a> Iterator for VertFaces<'a> {
    type Item = Handle<Face>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.radial_done {
                // Move to next edge in the disk cycle.
                let (first, current, done) =
                    Self::advance_disk(&mut self.disk_iter, self.edges);
                self.radial_first = first;
                self.radial_current = current;
                self.radial_done = done;
                if done {
                    return None;
                }
            }

            let l = self.loops.get(self.radial_current)?;
            let face_h = l.face;

            // Advance radial.
            self.radial_current = l.radial_next;
            if self.radial_current == self.radial_first {
                self.radial_done = true;
            }

            // Only yield if the loop's vertex matches ours.
            if l.vert == self.vert_h {
                return Some(face_h);
            }
        }
    }
}
