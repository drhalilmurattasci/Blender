//! Face extrusion — duplicates faces and connects them with side walls.

use forge3d_alloc::Handle;
use forge3d_math::Vec3;
use forge3d_mesh::{Face, Mesh, Vert};

use crate::common::{OpError, OpResult, SelectionHelper};
use super::ExtrudeParams;

/// Extrudes selected faces along their normals.
///
/// For each selected face:
/// 1. Duplicate the face's vertices, offset along the face normal.
/// 2. Create side-wall quads connecting old and new boundary edges.
/// 3. Replace the original face with the new (extruded) face.
pub fn extrude_faces(mesh: &mut Mesh, params: &ExtrudeParams) -> OpResult<()> {
    let selected = SelectionHelper::selected_faces(mesh);
    if selected.is_empty() {
        return Err(OpError::InvalidSelection("no faces selected".into()));
    }

    let offset = params.offset;

    for &face_h in &selected {
        if mesh.faces.contains(face_h) {
            extrude_single_face(mesh, face_h, offset)?;
        }
    }

    Ok(())
}

fn extrude_single_face(
    mesh: &mut Mesh,
    face_h: Handle<Face>,
    offset: f32,
) -> OpResult<()> {
    let face = mesh.faces.get(face_h).ok_or_else(|| {
        OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", face_h)))
    })?;
    let first_lh = face.loop_first;

    if first_lh.is_dangling() {
        return Ok(());
    }

    // Collect original vertices in order.
    let mut orig_verts: Vec<Handle<Vert>> = Vec::new();
    let mut cur_lh = first_lh;
    loop {
        let l = mesh.loops.get(cur_lh).ok_or_else(|| {
            OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", cur_lh)))
        })?;
        orig_verts.push(l.vert);
        cur_lh = l.next;
        if cur_lh == first_lh {
            break;
        }
    }

    let n = orig_verts.len();

    // Compute the face normal from vertex positions using Newell's method.
    // This ensures we always have a valid normal even if compute_face_normals
    // hasn't been called.
    let positions: Vec<Vec3> = orig_verts
        .iter()
        .map(|&vh| mesh.verts.get(vh).unwrap().co)
        .collect();

    let mut normal = Vec3::ZERO;
    for i in 0..n {
        let cur = positions[i];
        let next = positions[(i + 1) % n];
        normal.x += (cur.y - next.y) * (cur.z + next.z);
        normal.y += (cur.z - next.z) * (cur.x + next.x);
        normal.z += (cur.x - next.x) * (cur.y + next.y);
    }
    let len = normal.length();
    if len > f32::EPSILON {
        normal /= len;
    }

    let displacement = normal * offset;

    // Create new offset vertices.
    let mut new_verts: Vec<Handle<Vert>> = Vec::new();
    for &vh in &orig_verts {
        let co = mesh.verts.get(vh).unwrap().co;
        let new_h = mesh.create_vert(co + displacement);
        new_verts.push(new_h);
    }

    // Kill the original face.
    mesh.kill_face(face_h)?;

    // Create the new top face.
    mesh.create_face(&new_verts)?;

    // Create side-wall quads.
    for i in 0..n {
        let next = (i + 1) % n;
        let quad = [
            orig_verts[i],
            orig_verts[next],
            new_verts[next],
            new_verts[i],
        ];
        mesh.create_face(&quad)?;
    }

    Ok(())
}
