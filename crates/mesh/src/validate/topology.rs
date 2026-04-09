//! Topology cycle validation — checks that disk, radial, and face loop cycles
//! are well-formed circular doubly-linked lists.

use forge3d_alloc::{Arena, Handle};

use crate::elements::{Edge, Face, LoopElem, Vert};

/// Maximum iterations before declaring a cycle broken (infinite loop guard).
const MAX_CYCLE_LEN: usize = 1_000_000;

/// Validates the disk cycle of a single vertex.
/// Returns `Ok(edge_count)` or an error description.
pub fn validate_disk_cycle(
    verts: &Arena<Vert>,
    edges: &Arena<Edge>,
    vert_h: Handle<Vert>,
) -> Result<usize, String> {
    let vert = verts
        .get(vert_h)
        .ok_or_else(|| format!("Vert {:?} not found", vert_h))?;

    let first_h = match vert.edge {
        Some(h) => h,
        None => return Ok(0),
    };

    if edges.get(first_h).is_none() {
        return Err(format!(
            "Vert {:?} references dangling edge {:?}",
            vert_h, first_h
        ));
    }

    let mut count = 0usize;
    let mut cur_h = first_h;

    loop {
        let cur = edges
            .get(cur_h)
            .ok_or_else(|| format!("Disk cycle broken at edge {:?}", cur_h))?;

        // Verify the edge actually references this vertex.
        if cur.v1 != vert_h && cur.v2 != vert_h {
            return Err(format!(
                "Edge {:?} in disk of vert {:?} does not reference that vertex",
                cur_h, vert_h
            ));
        }

        let link = cur.disk_link(vert_h);
        let next_h = link.next;

        // Check back-link: next.prev == cur
        if let Some(next) = edges.get(next_h) {
            if next.disk_link(vert_h).prev != cur_h {
                return Err(format!(
                    "Disk back-link mismatch: edge {:?} -> next {:?}, but next.prev != cur",
                    cur_h, next_h
                ));
            }
        }

        count += 1;
        if count > MAX_CYCLE_LEN {
            return Err(format!(
                "Disk cycle of vert {:?} exceeds {} edges (likely infinite loop)",
                vert_h, MAX_CYCLE_LEN
            ));
        }

        cur_h = next_h;
        if cur_h == first_h {
            break;
        }
    }

    Ok(count)
}

/// Validates the radial cycle of a single edge.
pub fn validate_radial_cycle(
    edges: &Arena<Edge>,
    loops: &Arena<LoopElem>,
    edge_h: Handle<Edge>,
) -> Result<usize, String> {
    let edge = edges
        .get(edge_h)
        .ok_or_else(|| format!("Edge {:?} not found", edge_h))?;

    let first_h = match edge.loop_first {
        Some(h) => h,
        None => return Ok(0),
    };

    let mut count = 0usize;
    let mut cur_h = first_h;

    loop {
        let cur = loops
            .get(cur_h)
            .ok_or_else(|| format!("Radial cycle broken at loop {:?}", cur_h))?;

        if cur.edge != edge_h {
            return Err(format!(
                "Loop {:?} in radial of edge {:?} references edge {:?} instead",
                cur_h, edge_h, cur.edge
            ));
        }

        let next_h = cur.radial_next;

        if let Some(next) = loops.get(next_h) {
            if next.radial_prev != cur_h {
                return Err(format!(
                    "Radial back-link mismatch at loop {:?}",
                    cur_h
                ));
            }
        }

        count += 1;
        if count > MAX_CYCLE_LEN {
            return Err(format!(
                "Radial cycle of edge {:?} exceeds max length",
                edge_h
            ));
        }

        cur_h = next_h;
        if cur_h == first_h {
            break;
        }
    }

    Ok(count)
}

/// Validates the face loop cycle of a single face.
pub fn validate_face_loop_cycle(
    faces: &Arena<Face>,
    loops: &Arena<LoopElem>,
    face_h: Handle<Face>,
) -> Result<usize, String> {
    let face = faces
        .get(face_h)
        .ok_or_else(|| format!("Face {:?} not found", face_h))?;

    let first_h = face.loop_first;
    if first_h.is_dangling() {
        return Err(format!("Face {:?} has dangling loop_first", face_h));
    }

    if face.len < 3 {
        return Err(format!(
            "Face {:?} has len={} which is less than 3",
            face_h, face.len
        ));
    }

    let mut count = 0usize;
    let mut cur_h = first_h;

    loop {
        let cur = loops
            .get(cur_h)
            .ok_or_else(|| format!("Face loop cycle broken at loop {:?}", cur_h))?;

        if cur.face != face_h {
            return Err(format!(
                "Loop {:?} in face {:?} references face {:?} instead",
                cur_h, face_h, cur.face
            ));
        }

        // Verify that the loop's radial links are valid (not dangling).
        if cur.radial_next.is_dangling() || cur.radial_prev.is_dangling() {
            return Err(format!(
                "Loop {:?} in face {:?} has dangling radial links",
                cur_h, face_h
            ));
        }

        let next_h = cur.next;
        if next_h.is_dangling() {
            return Err(format!(
                "Loop {:?} in face {:?} has dangling next link",
                cur_h, face_h
            ));
        }
        if cur.prev.is_dangling() {
            return Err(format!(
                "Loop {:?} in face {:?} has dangling prev link",
                cur_h, face_h
            ));
        }

        if let Some(next) = loops.get(next_h) {
            if next.prev != cur_h {
                return Err(format!(
                    "Face loop back-link mismatch at loop {:?}",
                    cur_h
                ));
            }
        }

        count += 1;
        if count > MAX_CYCLE_LEN {
            return Err(format!(
                "Face loop cycle of face {:?} exceeds max length",
                face_h
            ));
        }

        cur_h = next_h;
        if cur_h == first_h {
            break;
        }
    }

    if count != face.len as usize {
        return Err(format!(
            "Face {:?} has len={} but loop cycle has {} elements",
            face_h, face.len, count
        ));
    }

    Ok(count)
}
