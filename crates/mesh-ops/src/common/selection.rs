//! Helpers for working with element selections.

use forge3d_alloc::Handle;
use forge3d_mesh::{Edge, ElemFlags, Face, Mesh, Vert};

/// Utility for collecting selected elements from a mesh.
pub struct SelectionHelper;

impl SelectionHelper {
    /// Collects handles to all selected vertices.
    pub fn selected_verts(mesh: &Mesh) -> Vec<Handle<Vert>> {
        mesh.verts
            .iter()
            .filter(|(_, v)| v.flags.contains(ElemFlags::SELECT))
            .map(|(h, _)| h)
            .collect()
    }

    /// Collects handles to all selected edges.
    pub fn selected_edges(mesh: &Mesh) -> Vec<Handle<Edge>> {
        mesh.edges
            .iter()
            .filter(|(_, e)| e.flags.contains(ElemFlags::SELECT))
            .map(|(h, _)| h)
            .collect()
    }

    /// Collects handles to all selected faces.
    pub fn selected_faces(mesh: &Mesh) -> Vec<Handle<Face>> {
        mesh.faces
            .iter()
            .filter(|(_, f)| f.flags.contains(ElemFlags::SELECT))
            .map(|(h, _)| h)
            .collect()
    }

    /// Selects all elements.
    pub fn select_all(mesh: &mut Mesh) {
        for (_, v) in mesh.verts.iter_mut() {
            v.flags.insert(ElemFlags::SELECT);
        }
        for (_, e) in mesh.edges.iter_mut() {
            e.flags.insert(ElemFlags::SELECT);
        }
        for (_, f) in mesh.faces.iter_mut() {
            f.flags.insert(ElemFlags::SELECT);
        }
    }

    /// Deselects all elements.
    pub fn deselect_all(mesh: &mut Mesh) {
        for (_, v) in mesh.verts.iter_mut() {
            v.flags.remove(ElemFlags::SELECT);
        }
        for (_, e) in mesh.edges.iter_mut() {
            e.flags.remove(ElemFlags::SELECT);
        }
        for (_, f) in mesh.faces.iter_mut() {
            f.flags.remove(ElemFlags::SELECT);
        }
    }
}
