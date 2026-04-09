//! Disk cycle operations — the circular list of edges around a vertex.

use forge3d_alloc::{Arena, Handle};

use crate::elements::{Edge, Vert};

/// Appends `edge_h` into the disk cycle of vertex `vert_h`.
///
/// If the vertex has no edge yet, the edge becomes the sole entry (self-loop).
/// Otherwise it is inserted after the vertex's current first edge.
pub fn disk_append(
    verts: &mut Arena<Vert>,
    edges: &mut Arena<Edge>,
    vert_h: Handle<Vert>,
    edge_h: Handle<Edge>,
) {
    let vert = verts.get_mut(vert_h).expect("disk_append: invalid vert handle");

    if let Some(first_h) = vert.edge {
        // Insert after `first` in the cycle.
        let first = edges.get(first_h).expect("disk_append: invalid first edge");
        let prev_h = first.disk_link(vert_h).prev;

        // edge -> next = first, edge -> prev = prev
        {
            let e = edges.get_mut(edge_h).expect("disk_append: invalid edge handle");
            let link = e.disk_link_mut(vert_h);
            link.next = first_h;
            link.prev = prev_h;
        }

        // prev -> next = edge
        {
            let prev = edges.get_mut(prev_h).expect("disk_append: invalid prev edge");
            prev.disk_link_mut(vert_h).next = edge_h;
        }

        // first -> prev = edge
        {
            let first = edges.get_mut(first_h).expect("disk_append: invalid first edge");
            first.disk_link_mut(vert_h).prev = edge_h;
        }
    } else {
        // First edge for this vertex — self-loop.
        vert.edge = Some(edge_h);
        let e = edges.get_mut(edge_h).expect("disk_append: invalid edge handle");
        let link = e.disk_link_mut(vert_h);
        link.next = edge_h;
        link.prev = edge_h;
    }
}

/// Removes `edge_h` from the disk cycle of vertex `vert_h`.
pub fn disk_remove(
    verts: &mut Arena<Vert>,
    edges: &mut Arena<Edge>,
    vert_h: Handle<Vert>,
    edge_h: Handle<Edge>,
) {
    let edge = edges.get(edge_h).expect("disk_remove: invalid edge handle");
    let next_h = edge.disk_link(vert_h).next;
    let prev_h = edge.disk_link(vert_h).prev;

    if next_h == edge_h {
        // Sole edge — clear the vertex's edge pointer.
        let vert = verts.get_mut(vert_h).expect("disk_remove: invalid vert handle");
        vert.edge = None;
    } else {
        // Unlink from cycle.
        {
            let next = edges.get_mut(next_h).expect("disk_remove: invalid next edge");
            next.disk_link_mut(vert_h).prev = prev_h;
        }
        {
            let prev = edges.get_mut(prev_h).expect("disk_remove: invalid prev edge");
            prev.disk_link_mut(vert_h).next = next_h;
        }

        // If this was the vertex's first edge, advance the pointer.
        let vert = verts.get_mut(vert_h).expect("disk_remove: invalid vert handle");
        if vert.edge == Some(edge_h) {
            vert.edge = Some(next_h);
        }
    }

    // Clear the removed edge's disk links for this vertex.
    let edge = edges.get_mut(edge_h).expect("disk_remove: invalid edge handle");
    let link = edge.disk_link_mut(vert_h);
    link.next = Handle::dangling();
    link.prev = Handle::dangling();
}

/// Counts the number of edges in the disk cycle of `vert_h`.
pub fn disk_count(
    verts: &Arena<Vert>,
    edges: &Arena<Edge>,
    vert_h: Handle<Vert>,
) -> usize {
    let vert = match verts.get(vert_h) {
        Some(v) => v,
        None => return 0,
    };
    let first_h = match vert.edge {
        Some(h) => h,
        None => return 0,
    };

    let mut count = 0;
    let mut cur_h = first_h;
    loop {
        count += 1;
        let cur = edges.get(cur_h).expect("disk_count: broken disk cycle");
        cur_h = cur.disk_link(vert_h).next;
        if cur_h == first_h {
            break;
        }
    }
    count
}

/// Maximum number of iterations before bailing out (infinite loop guard).
const MAX_DISK_ITER: usize = 1_000_000;

/// Iterator over all edges in the disk cycle of a vertex.
pub struct DiskIter<'a> {
    edges: &'a Arena<Edge>,
    vert_h: Handle<Vert>,
    first_h: Handle<Edge>,
    current_h: Handle<Edge>,
    done: bool,
    /// Safety guard to prevent infinite loops on corrupt cycles.
    remaining: usize,
}

impl<'a> DiskIter<'a> {
    /// Creates a new disk iterator for the given vertex.
    /// Returns an empty iterator if the vertex has no edges.
    pub fn new(
        verts: &Arena<Vert>,
        edges: &'a Arena<Edge>,
        vert_h: Handle<Vert>,
    ) -> Self {
        let first_h = verts
            .get(vert_h)
            .and_then(|v| v.edge)
            .unwrap_or_else(Handle::dangling);

        Self {
            edges,
            vert_h,
            first_h,
            current_h: first_h,
            done: first_h.is_dangling(),
            remaining: MAX_DISK_ITER,
        }
    }
}

impl<'a> Iterator for DiskIter<'a> {
    type Item = Handle<Edge>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done || self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let result = self.current_h;
        let cur = self.edges.get(self.current_h)?;
        self.current_h = cur.disk_link(self.vert_h).next;
        if self.current_h == self.first_h {
            self.done = true;
        }
        Some(result)
    }
}
