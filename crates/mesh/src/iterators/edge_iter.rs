//! Iterator from an edge to its adjacent faces via the radial cycle.

use forge3d_alloc::{Arena, Handle};

use crate::elements::{Edge, Face, LoopElem};
use crate::cycles::radial::RadialIter;

/// Iterator over all faces sharing the given edge.
pub struct EdgeFaces<'a> {
    loops: &'a Arena<LoopElem>,
    radial: RadialIter<'a>,
}

impl<'a> EdgeFaces<'a> {
    /// Creates an iterator over faces adjacent to `edge_h`.
    pub fn new(
        edges: &Arena<Edge>,
        loops: &'a Arena<LoopElem>,
        edge_h: Handle<Edge>,
    ) -> Self {
        Self {
            loops,
            radial: RadialIter::new(edges, loops, edge_h),
        }
    }
}

impl<'a> Iterator for EdgeFaces<'a> {
    type Item = Handle<Face>;

    fn next(&mut self) -> Option<Self::Item> {
        let loop_h = self.radial.next()?;
        let l = self.loops.get(loop_h)?;
        Some(l.face)
    }
}
