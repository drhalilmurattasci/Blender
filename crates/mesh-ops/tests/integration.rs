//! Integration tests: create triangle -> validate -> extrude -> validate ->
//! subdivide -> validate.

use forge3d_math::Vec3;
use forge3d_mesh::{Mesh, MeshError};
use forge3d_mesh_ops::{
    CatmullClarkParams, ExtrudeParams,
    subdivide::subdivide_simple,
    extrude::extrude_faces,
    common::SelectionHelper,
};

/// Helper: creates a single-triangle mesh and validates it.
fn make_triangle() -> Mesh {
    let mut mesh = Mesh::new();
    let v0 = mesh.create_vert(Vec3::new(0.0, 0.0, 0.0));
    let v1 = mesh.create_vert(Vec3::new(1.0, 0.0, 0.0));
    let v2 = mesh.create_vert(Vec3::new(0.5, 1.0, 0.0));
    mesh.create_face(&[v0, v1, v2]).expect("create triangle");
    mesh
}

fn assert_valid(mesh: &Mesh) {
    let errors = mesh.validate();
    assert!(errors.is_empty(), "Mesh validation failed:\n{}", errors.join("\n"));
}

// -----------------------------------------------------------------------
// Basic creation & validation
// -----------------------------------------------------------------------

#[test]
fn create_triangle_validates() {
    let mesh = make_triangle();
    assert_eq!(mesh.vert_count(), 3);
    assert_eq!(mesh.edge_count(), 3);
    assert_eq!(mesh.face_count(), 1);
    assert_eq!(mesh.loop_count(), 3);
    assert_valid(&mesh);
}

#[test]
fn create_quad_validates() {
    let mut mesh = Mesh::new();
    let v0 = mesh.create_vert(Vec3::new(0.0, 0.0, 0.0));
    let v1 = mesh.create_vert(Vec3::new(1.0, 0.0, 0.0));
    let v2 = mesh.create_vert(Vec3::new(1.0, 1.0, 0.0));
    let v3 = mesh.create_vert(Vec3::new(0.0, 1.0, 0.0));
    mesh.create_face(&[v0, v1, v2, v3]).expect("create quad");
    assert_eq!(mesh.face_count(), 1);
    assert_valid(&mesh);
}

// -----------------------------------------------------------------------
// Edge case: creating a face with 0, 1, or 2 vertices should error
// -----------------------------------------------------------------------

#[test]
fn face_with_zero_verts_errors() {
    let mut mesh = Mesh::new();
    let result = mesh.create_face(&[]);
    assert!(matches!(result, Err(MeshError::TooFewVertices(0))));
}

#[test]
fn face_with_one_vert_errors() {
    let mut mesh = Mesh::new();
    let v0 = mesh.create_vert(Vec3::ZERO);
    let result = mesh.create_face(&[v0]);
    assert!(matches!(result, Err(MeshError::TooFewVertices(1))));
}

#[test]
fn face_with_two_verts_errors() {
    let mut mesh = Mesh::new();
    let v0 = mesh.create_vert(Vec3::ZERO);
    let v1 = mesh.create_vert(Vec3::X);
    let result = mesh.create_face(&[v0, v1]);
    assert!(matches!(result, Err(MeshError::TooFewVertices(2))));
}

#[test]
fn face_with_duplicate_vertex_errors() {
    let mut mesh = Mesh::new();
    let v0 = mesh.create_vert(Vec3::ZERO);
    let v1 = mesh.create_vert(Vec3::X);
    let result = mesh.create_face(&[v0, v1, v0]);
    assert!(matches!(result, Err(MeshError::DegenerateTopology(_))));
}

#[test]
fn duplicate_edge_errors() {
    let mut mesh = Mesh::new();
    let v0 = mesh.create_vert(Vec3::ZERO);
    let v1 = mesh.create_vert(Vec3::X);
    mesh.create_edge(v0, v1).expect("first edge");
    let result = mesh.create_edge(v0, v1);
    assert!(matches!(result, Err(MeshError::DuplicateEdge)));
}

#[test]
fn self_loop_edge_errors() {
    let mut mesh = Mesh::new();
    let v0 = mesh.create_vert(Vec3::ZERO);
    let result = mesh.create_edge(v0, v0);
    assert!(matches!(result, Err(MeshError::DegenerateTopology(_))));
}

// -----------------------------------------------------------------------
// Pipeline: create triangle -> extrude -> validate -> subdivide -> validate
// -----------------------------------------------------------------------

#[test]
fn triangle_extrude_validate() {
    let mut mesh = make_triangle();
    assert_valid(&mesh);

    // Select all faces for extrusion.
    SelectionHelper::select_all(&mut mesh);

    let params = ExtrudeParams {
        offset: 1.0,
        direction: None,
    };
    extrude_faces(&mut mesh, &params).expect("extrude faces");

    // After extruding a triangle: 1 top triangle + 3 side quads = 4 faces.
    // Original triangle is killed, so we get 4 new faces.
    assert!(mesh.face_count() >= 4, "expected at least 4 faces after extrude, got {}", mesh.face_count());
    assert_valid(&mesh);
}

#[test]
fn triangle_extrude_subdivide_validate() {
    let mut mesh = make_triangle();
    assert_valid(&mesh);

    // Extrude.
    SelectionHelper::select_all(&mut mesh);
    let params = ExtrudeParams {
        offset: 0.5,
        direction: None,
    };
    extrude_faces(&mut mesh, &params).expect("extrude faces");
    assert_valid(&mesh);

    // Simple subdivision.
    subdivide_simple(&mut mesh).expect("subdivide simple");
    assert!(mesh.face_count() > 4, "subdivision should increase face count");
    assert_valid(&mesh);
}

#[test]
fn triangle_extrude_catmull_clark_validate() {
    let mut mesh = make_triangle();
    assert_valid(&mesh);

    // Extrude.
    SelectionHelper::select_all(&mut mesh);
    let params = ExtrudeParams {
        offset: 0.5,
        direction: None,
    };
    extrude_faces(&mut mesh, &params).expect("extrude faces");
    assert_valid(&mesh);

    // Catmull-Clark subdivision.
    let cc_params = CatmullClarkParams {
        iterations: 1,
        smooth: true,
    };
    forge3d_mesh_ops::subdivide::catmull_clark(&mut mesh, &cc_params)
        .expect("catmull-clark");
    assert!(mesh.face_count() > 4, "CC subdivision should increase face count");
    assert_valid(&mesh);
}

// -----------------------------------------------------------------------
// Kill operations maintain valid topology
// -----------------------------------------------------------------------

#[test]
fn kill_face_keeps_edges_and_verts() {
    let mut mesh = make_triangle();
    let face_h = mesh.faces.iter().next().unwrap().0;
    mesh.kill_face(face_h).expect("kill face");

    assert_eq!(mesh.face_count(), 0);
    assert_eq!(mesh.loop_count(), 0);
    // Edges and verts should remain.
    assert_eq!(mesh.vert_count(), 3);
    assert_eq!(mesh.edge_count(), 3);
    assert_valid(&mesh);
}

#[test]
fn kill_edge_removes_connected_faces() {
    let mut mesh = make_triangle();
    let edge_h = mesh.edges.iter().next().unwrap().0;
    mesh.kill_edge(edge_h).expect("kill edge");

    // The triangle face should have been killed.
    assert_eq!(mesh.face_count(), 0);
    assert_eq!(mesh.loop_count(), 0);
    // One edge killed, two remain.
    assert_eq!(mesh.edge_count(), 2);
    assert_valid(&mesh);
}

#[test]
fn kill_vert_removes_connected_edges_and_faces() {
    let mut mesh = make_triangle();
    let vert_h = mesh.verts.iter().next().unwrap().0;
    mesh.kill_vert(vert_h).expect("kill vert");

    assert_eq!(mesh.face_count(), 0);
    // Two edges connected to the killed vertex are gone; the third remains.
    assert_eq!(mesh.edge_count(), 1);
    assert_eq!(mesh.vert_count(), 2);
    assert_valid(&mesh);
}

// -----------------------------------------------------------------------
// Stale handle detection
// -----------------------------------------------------------------------

#[test]
fn stale_vert_handle_detected() {
    let mut mesh = Mesh::new();
    let v = mesh.create_vert(Vec3::ZERO);
    mesh.verts.remove(v);
    let result = mesh.kill_vert(v);
    assert!(matches!(result, Err(MeshError::StaleHandle(_))));
}

#[test]
fn stale_edge_handle_detected() {
    let mut mesh = Mesh::new();
    let v0 = mesh.create_vert(Vec3::ZERO);
    let v1 = mesh.create_vert(Vec3::X);
    let e = mesh.create_edge(v0, v1).unwrap();
    mesh.kill_edge(e).unwrap();
    let result = mesh.kill_edge(e);
    assert!(matches!(result, Err(MeshError::StaleHandle(_))));
}

// -----------------------------------------------------------------------
// Two faces sharing an edge (manifold)
// -----------------------------------------------------------------------

#[test]
fn two_triangles_sharing_edge_validates() {
    let mut mesh = Mesh::new();
    let v0 = mesh.create_vert(Vec3::new(0.0, 0.0, 0.0));
    let v1 = mesh.create_vert(Vec3::new(1.0, 0.0, 0.0));
    let v2 = mesh.create_vert(Vec3::new(0.5, 1.0, 0.0));
    let v3 = mesh.create_vert(Vec3::new(0.5, -1.0, 0.0));

    mesh.create_face(&[v0, v1, v2]).expect("face 1");
    mesh.create_face(&[v1, v0, v3]).expect("face 2");

    assert_eq!(mesh.face_count(), 2);
    assert_valid(&mesh);
}

// -----------------------------------------------------------------------
// Thread safety
// -----------------------------------------------------------------------

#[test]
fn mesh_is_send_and_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<Mesh>();
    assert_sync::<Mesh>();
}
