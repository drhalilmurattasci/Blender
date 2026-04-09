//! Radial cycle operations — the circular list of loops around an edge.

use forge3d_alloc::{Arena, Handle};

use crate::elements::{Edge, LoopElem};

/// Appends `loop_h` into the radial cycle of `edge_h`.
///
/// If the edge has no loop yet, the loop becomes the sole entry (self-loop).
/// Otherwise it is inserted after the edge's current first loop.
pub fn radial_append(
    edges: &mut Arena<Edge>,
    loops: &mut Arena<LoopElem>,
    edge_h: Handle<Edge>,
    loop_h: Handle<LoopElem>,
) {
    let edge = edges.get_mut(edge_h).expect("radial_append: invalid edge handle");

    if let Some(first_h) = edge.loop_first {
        let first = loops.get(first_h).expect("radial_append: invalid first loop");
        let prev_h = first.radial_prev;

        // loop -> next = first, loop -> prev = prev
        {
            let l = loops.get_mut(loop_h).expect("radial_append: invalid loop handle");
            l.radial_next = first_h;
            l.radial_prev = prev_h;
        }

        // prev -> next = loop
        {
            let prev = loops.get_mut(prev_h).expect("radial_append: invalid prev loop");
            prev.radial_next = loop_h;
        }

        // first -> prev = loop
        {
            let first = loops.get_mut(first_h).expect("radial_append: invalid first loop");
            first.radial_prev = loop_h;
        }
    } else {
        // First loop for this edge.
        edge.loop_first = Some(loop_h);
        let l = loops.get_mut(loop_h).expect("radial_append: invalid loop handle");
        l.radial_next = loop_h;
        l.radial_prev = loop_h;
    }
}

/// Removes `loop_h` from the radial cycle of `edge_h`.
pub fn radial_remove(
    edges: &mut Arena<Edge>,
    loops: &mut Arena<LoopElem>,
    edge_h: Handle<Edge>,
    loop_h: Handle<LoopElem>,
) {
    let l = loops.get(loop_h).expect("radial_remove: invalid loop handle");
    let next_h = l.radial_next;
    let prev_h = l.radial_prev;

    if next_h == loop_h {
        // Sole loop — clear the edge's loop pointer.
        let edge = edges.get_mut(edge_h).expect("radial_remove: invalid edge handle");
        edge.loop_first = None;
    } else {
        // Unlink from cycle.
        {
            let next = loops.get_mut(next_h).expect("radial_remove: invalid next loop");
            next.radial_prev = prev_h;
        }
        {
            let prev = loops.get_mut(prev_h).expect("radial_remove: invalid prev loop");
            prev.radial_next = next_h;
        }

        let edge = edges.get_mut(edge_h).expect("radial_remove: invalid edge handle");
        if edge.loop_first == Some(loop_h) {
            edge.loop_first = Some(next_h);
        }
    }

    // Clear removed loop's radial links.
    let l = loops.get_mut(loop_h).expect("radial_remove: invalid loop handle");
    l.radial_next = Handle::dangling();
    l.radial_prev = Handle::dangling();
}

/// Counts the number of loops in the radial cycle of `edge_h`.
pub fn radial_count(
    edges: &Arena<Edge>,
    loops: &Arena<LoopElem>,
    edge_h: Handle<Edge>,
) -> usize {
    let edge = match edges.get(edge_h) {
        Some(e) => e,
        None => return 0,
    };
    let first_h = match edge.loop_first {
        Some(h) => h,
        None => return 0,
    };

    let mut count = 0;
    let mut cur_h = first_h;
    loop {
        count += 1;
        let cur = loops.get(cur_h).expect("radial_count: broken radial cycle");
        cur_h = cur.radial_next;
        if cur_h == first_h {
            break;
        }
    }
    count
}

/// Maximum number of iterations before bailing out (infinite loop guard).
const MAX_RADIAL_ITER: usize = 1_000_000;

/// Iterator over all loops in the radial cycle of an edge.
pub struct RadialIter<'a> {
    loops: &'a Arena<LoopElem>,
    first_h: Handle<LoopElem>,
    current_h: Handle<LoopElem>,
    done: bool,
    /// Safety guard to prevent infinite loops on corrupt cycles.
    remaining: usize,
}

impl<'a> RadialIter<'a> {
    /// Creates a new radial iterator for the given edge.
    pub fn new(
        edges: &Arena<Edge>,
        loops: &'a Arena<LoopElem>,
        edge_h: Handle<Edge>,
    ) -> Self {
        let first_h = edges
            .get(edge_h)
            .and_then(|e| e.loop_first)
            .unwrap_or_else(Handle::dangling);

        Self {
            loops,
            first_h,
            current_h: first_h,
            done: first_h.is_dangling(),
            remaining: MAX_RADIAL_ITER,
        }
    }
}

impl<'a> Iterator for RadialIter<'a> {
    type Item = Handle<LoopElem>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done || self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let result = self.current_h;
        let cur = self.loops.get(self.current_h)?;
        self.current_h = cur.radial_next;
        if self.current_h == self.first_h {
            self.done = true;
        }
        Some(result)
    }
}
