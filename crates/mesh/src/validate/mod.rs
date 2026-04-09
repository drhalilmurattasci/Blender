//! Mesh validation utilities.

pub mod integrity;
pub mod topology;

pub use integrity::validate_referential_integrity;
pub use topology::{validate_disk_cycle, validate_face_loop_cycle, validate_radial_cycle};

use forge3d_alloc::Arena;

use crate::elements::{Edge, Face, LoopElem, Vert};

/// Performs a full validation pass on the mesh topology.
///
/// Returns a list of error descriptions. An empty list means the mesh is valid.
pub fn validate(
    verts: &Arena<Vert>,
    edges: &Arena<Edge>,
    loops: &Arena<LoopElem>,
    faces: &Arena<Face>,
) -> Vec<String> {
    let mut errors = Vec::new();

    // Referential integrity.
    errors.extend(validate_referential_integrity(verts, edges, loops, faces));

    // Disk cycles.
    for (vh, _) in verts.iter() {
        if let Err(e) = validate_disk_cycle(verts, edges, vh) {
            errors.push(e);
        }
    }

    // Radial cycles.
    for (eh, _) in edges.iter() {
        if let Err(e) = validate_radial_cycle(edges, loops, eh) {
            errors.push(e);
        }
    }

    // Face loop cycles.
    for (fh, _) in faces.iter() {
        if let Err(e) = validate_face_loop_cycle(faces, loops, fh) {
            errors.push(e);
        }
    }

    errors
}
