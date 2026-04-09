//! Fan triangulation — simple pivot-based triangulation.

use forge3d_alloc::Handle;
use forge3d_mesh::{Face, Mesh, Vert};

use crate::common::{OpError, OpResult};

/// Triangulates a face using a fan from its first vertex.
///
/// For vertex list `[a, b, c, d, ...]` produces triangles
/// `(a,b,c)`, `(a,c,d)`, ...
///
/// This replaces the original face with triangle faces in the mesh.
pub fn fan_triangulate_face(mesh: &mut Mesh, face_h: Handle<Face>) -> OpResult<()> {
    let face = mesh.faces.get(face_h).ok_or_else(|| {
        OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", face_h)))
    })?;

    if face.len <= 3 {
        return Ok(()); // Already a triangle.
    }

    let first_lh = face.loop_first;
    if first_lh.is_dangling() {
        return Ok(());
    }

    // Collect vertices.
    let mut verts: Vec<Handle<Vert>> = Vec::new();
    let mut cur_lh = first_lh;
    loop {
        let l = mesh.loops.get(cur_lh).unwrap();
        verts.push(l.vert);
        cur_lh = l.next;
        if cur_lh == first_lh {
            break;
        }
    }

    // Remove the original face.
    mesh.kill_face(face_h)?;

    // Create triangle fan.
    let pivot = verts[0];
    for i in 1..verts.len() - 1 {
        mesh.create_face(&[pivot, verts[i], verts[i + 1]])?;
    }

    Ok(())
}

/// Triangulates all faces in the mesh using fan triangulation.
pub fn fan_triangulate_all(mesh: &mut Mesh) -> OpResult<()> {
    let face_handles: Vec<Handle<Face>> = mesh
        .faces
        .iter()
        .filter(|(_, f)| f.len > 3)
        .map(|(h, _)| h)
        .collect();

    for fh in face_handles {
        if mesh.faces.contains(fh) {
            fan_triangulate_face(mesh, fh)?;
        }
    }

    Ok(())
}
