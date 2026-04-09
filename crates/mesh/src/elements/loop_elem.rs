//! Loop element (corner of a face).

use forge3d_alloc::Handle;
use serde::{Deserialize, Serialize};

use super::edge::Edge;
use super::face::Face;
use super::vert::Vert;

/// A loop (corner) element that ties a vertex, edge, and face together.
///
/// Loops form two cycles:
/// - **Radial cycle**: all loops sharing the same edge (doubly linked via
///   `radial_next` / `radial_prev`).
/// - **Face loop cycle**: the ordered boundary of a face (doubly linked via
///   `next` / `prev`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopElem {
    /// The vertex at this corner.
    pub vert: Handle<Vert>,
    /// The edge departing from this corner toward the next corner.
    pub edge: Handle<Edge>,
    /// The face this loop belongs to.
    pub face: Handle<Face>,

    // --- Radial cycle (all loops around the same edge) ---
    /// Next loop in the radial cycle.
    pub radial_next: Handle<LoopElem>,
    /// Previous loop in the radial cycle.
    pub radial_prev: Handle<LoopElem>,

    // --- Face loop cycle (boundary of the owning face) ---
    /// Next loop around the face boundary.
    pub next: Handle<LoopElem>,
    /// Previous loop around the face boundary.
    pub prev: Handle<LoopElem>,
}

impl LoopElem {
    /// Creates a new loop element with all link handles pointing to `dangling`.
    pub fn new(vert: Handle<Vert>, edge: Handle<Edge>, face: Handle<Face>) -> Self {
        Self {
            vert,
            edge,
            face,
            radial_next: Handle::dangling(),
            radial_prev: Handle::dangling(),
            next: Handle::dangling(),
            prev: Handle::dangling(),
        }
    }
}
