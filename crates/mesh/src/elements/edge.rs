//! Edge element.

use forge3d_alloc::Handle;
use serde::{Deserialize, Serialize};

use super::flags::ElemFlags;
use super::loop_elem::LoopElem;
use super::vert::Vert;

/// Disk link for one endpoint of an edge, forming a circular doubly-linked list
/// of all edges sharing that vertex.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DiskLink {
    /// Next edge in the disk cycle around this vertex.
    pub next: Handle<Edge>,
    /// Previous edge in the disk cycle around this vertex.
    pub prev: Handle<Edge>,
}

impl Default for DiskLink {
    fn default() -> Self {
        Self {
            next: Handle::dangling(),
            prev: Handle::dangling(),
        }
    }
}

/// A mesh edge connecting two vertices, with disk links and an optional loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// First endpoint vertex.
    pub v1: Handle<Vert>,
    /// Second endpoint vertex.
    pub v2: Handle<Vert>,
    /// First loop in the radial cycle around this edge.
    /// `None` if no face uses this edge (wire edge).
    pub loop_first: Option<Handle<LoopElem>>,
    /// Disk cycle link for vertex v1.
    pub disk_v1: DiskLink,
    /// Disk cycle link for vertex v2.
    pub disk_v2: DiskLink,
    /// Element flags.
    pub flags: ElemFlags,
}

impl Edge {
    /// Creates a new edge between two vertices with no loops and default disk links.
    pub fn new(v1: Handle<Vert>, v2: Handle<Vert>) -> Self {
        Self {
            v1,
            v2,
            loop_first: None,
            disk_v1: DiskLink::default(),
            disk_v2: DiskLink::default(),
            flags: ElemFlags::empty(),
        }
    }

    /// Returns the other vertex handle given one endpoint.
    /// Panics if `v` is neither `v1` nor `v2`.
    pub fn other_vert(&self, v: Handle<Vert>) -> Handle<Vert> {
        if v == self.v1 {
            self.v2
        } else if v == self.v2 {
            self.v1
        } else {
            panic!("vertex handle does not belong to this edge");
        }
    }

    /// Returns the disk link for the given vertex endpoint.
    pub fn disk_link(&self, v: Handle<Vert>) -> &DiskLink {
        if v == self.v1 {
            &self.disk_v1
        } else if v == self.v2 {
            &self.disk_v2
        } else {
            panic!("vertex handle does not belong to this edge");
        }
    }

    /// Returns a mutable disk link for the given vertex endpoint.
    pub fn disk_link_mut(&mut self, v: Handle<Vert>) -> &mut DiskLink {
        if v == self.v1 {
            &mut self.disk_v1
        } else if v == self.v2 {
            &mut self.disk_v2
        } else {
            panic!("vertex handle does not belong to this edge");
        }
    }
}
