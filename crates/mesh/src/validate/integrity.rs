//! Referential integrity validation — ensures all handles point to live elements.

use forge3d_alloc::Arena;

use crate::elements::{Edge, Face, LoopElem, Vert};

/// Validates that every handle referenced by mesh elements points to a live
/// entry in the corresponding arena.
pub fn validate_referential_integrity(
    verts: &Arena<Vert>,
    edges: &Arena<Edge>,
    loops: &Arena<LoopElem>,
    faces: &Arena<Face>,
) -> Vec<String> {
    let mut errors = Vec::new();

    // Check vertices.
    for (vh, vert) in verts.iter() {
        if let Some(edge_h) = vert.edge {
            if !edges.contains(edge_h) {
                errors.push(format!(
                    "Vert {:?} references dead edge {:?}",
                    vh, edge_h
                ));
            }
        }
    }

    // Check edges.
    for (eh, edge) in edges.iter() {
        // Degenerate edge check.
        if edge.v1 == edge.v2 {
            errors.push(format!("Edge {:?} has v1 == v2 (degenerate)", eh));
        }
        if !verts.contains(edge.v1) {
            errors.push(format!("Edge {:?} references dead v1 {:?}", eh, edge.v1));
        }
        if !verts.contains(edge.v2) {
            errors.push(format!("Edge {:?} references dead v2 {:?}", eh, edge.v2));
        }
        if let Some(lh) = edge.loop_first {
            if !loops.contains(lh) {
                errors.push(format!(
                    "Edge {:?} references dead loop_first {:?}",
                    eh, lh
                ));
            }
        }
        // Check disk links: if vertex has an edge, disk links should not be dangling.
        for (label, link, v_h) in [("v1", &edge.disk_v1, edge.v1), ("v2", &edge.disk_v2, edge.v2)] {
            // If the vertex is alive, disk links must be set (not dangling).
            if verts.contains(v_h) {
                if link.next.is_dangling() || link.prev.is_dangling() {
                    errors.push(format!(
                        "Edge {:?} disk_{} has dangling links but vertex {:?} is alive",
                        eh, label, v_h
                    ));
                }
            }
            if !link.next.is_dangling() && !edges.contains(link.next) {
                errors.push(format!(
                    "Edge {:?} disk_{}.next references dead edge {:?}",
                    eh, label, link.next
                ));
            }
            if !link.prev.is_dangling() && !edges.contains(link.prev) {
                errors.push(format!(
                    "Edge {:?} disk_{}.prev references dead edge {:?}",
                    eh, label, link.prev
                ));
            }
        }
    }

    // Check loops.
    for (lh, l) in loops.iter() {
        if !verts.contains(l.vert) {
            errors.push(format!("Loop {:?} references dead vert {:?}", lh, l.vert));
        }
        if !edges.contains(l.edge) {
            errors.push(format!("Loop {:?} references dead edge {:?}", lh, l.edge));
        }
        if !faces.contains(l.face) {
            errors.push(format!("Loop {:?} references dead face {:?}", lh, l.face));
        }
        if !l.radial_next.is_dangling() && !loops.contains(l.radial_next) {
            errors.push(format!(
                "Loop {:?} radial_next references dead loop {:?}",
                lh, l.radial_next
            ));
        }
        if !l.radial_prev.is_dangling() && !loops.contains(l.radial_prev) {
            errors.push(format!(
                "Loop {:?} radial_prev references dead loop {:?}",
                lh, l.radial_prev
            ));
        }
        if !l.next.is_dangling() && !loops.contains(l.next) {
            errors.push(format!(
                "Loop {:?} next references dead loop {:?}",
                lh, l.next
            ));
        }
        if !l.prev.is_dangling() && !loops.contains(l.prev) {
            errors.push(format!(
                "Loop {:?} prev references dead loop {:?}",
                lh, l.prev
            ));
        }
    }

    // Check loop-edge consistency: each loop's edge must connect loop.vert
    // to the next loop's vert (in either direction).
    for (lh, l) in loops.iter() {
        if !l.next.is_dangling() {
            if let Some(next_l) = loops.get(l.next) {
                if let Some(edge) = edges.get(l.edge) {
                    let endpoints = (edge.v1, edge.v2);
                    let v_cur = l.vert;
                    let v_next = next_l.vert;
                    if !((endpoints.0 == v_cur && endpoints.1 == v_next)
                        || (endpoints.0 == v_next && endpoints.1 == v_cur))
                    {
                        errors.push(format!(
                            "Loop {:?} edge {:?} does not connect loop.vert {:?} to next loop.vert {:?}",
                            lh, l.edge, v_cur, v_next
                        ));
                    }
                }
            }
        }
    }

    // Check faces.
    for (fh, face) in faces.iter() {
        if !loops.contains(face.loop_first) {
            errors.push(format!(
                "Face {:?} references dead loop_first {:?}",
                fh, face.loop_first
            ));
        }
    }

    errors
}
