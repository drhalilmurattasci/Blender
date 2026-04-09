//! Edge bevel — offsets each selected edge to create new face strips.

use forge3d_alloc::Handle;
use forge3d_math::Vec3;
use forge3d_mesh::{Edge, Mesh, Vert};
use smallvec::SmallVec;

use crate::common::{OpError, OpResult, SelectionHelper};
use super::BevelParams;

/// Bevels each selected edge by replacing it with a face strip.
///
/// For each selected edge, two new edges are created at `offset` distance,
/// and a quad connects them.
pub fn edge_bevel(mesh: &mut Mesh, params: &BevelParams) -> OpResult<()> {
    let selected = SelectionHelper::selected_edges(mesh);
    if selected.is_empty() {
        return Err(OpError::InvalidSelection("no edges selected".into()));
    }

    let offset = params.offset.max(0.0);
    if offset < f32::EPSILON {
        return Ok(());
    }

    let segments = params.segments.max(1);

    for edge_h in &selected {
        if mesh.edges.contains(*edge_h) {
            bevel_single_edge(mesh, *edge_h, offset, segments)?;
        }
    }

    Ok(())
}

/// Bevels a single edge.
fn bevel_single_edge(
    mesh: &mut Mesh,
    edge_h: Handle<Edge>,
    offset: f32,
    segments: u32,
) -> OpResult<()> {
    let edge = mesh
        .edges
        .get(edge_h)
        .ok_or_else(|| OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", edge_h))))?;

    let v1_h = edge.v1;
    let v2_h = edge.v2;

    let v1 = mesh.verts.get(v1_h).ok_or_else(|| {
        OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", v1_h)))
    })?;
    let v2 = mesh.verts.get(v2_h).ok_or_else(|| {
        OpError::Mesh(forge3d_mesh::MeshError::StaleHandle(format!("{:?}", v2_h)))
    })?;

    let co1 = v1.co;
    let co2 = v2.co;
    let n1 = v1.normal;
    let n2 = v2.normal;

    // Compute offset direction perpendicular to the edge and the average normal.
    let edge_dir = (co2 - co1).normalize_or_zero();
    let avg_normal = ((n1 + n2) * 0.5).normalize_or_zero();
    let offset_dir = if avg_normal.length_squared() > f32::EPSILON {
        edge_dir.cross(avg_normal).normalize_or_zero()
    } else {
        // Fallback: pick an arbitrary perpendicular.
        let arbitrary = if edge_dir.x.abs() < 0.9 {
            Vec3::X
        } else {
            Vec3::Y
        };
        edge_dir.cross(arbitrary).normalize_or_zero()
    };

    // Kill the original edge (and any faces using it) before creating the bevel geometry.
    mesh.kill_edge(edge_h).map_err(OpError::Mesh)?;

    // Create new vertices on each side for each segment.
    // The bevel replaces the edge with a strip of quads. At t=0 we place verts
    // offset by -offset/2 and at t=segments we place them at +offset/2, centering
    // the bevel strip around the original edge position.
    let half = offset * 0.5;
    let step = offset / segments as f32;
    let mut left_verts: SmallVec<[Handle<Vert>; 4]> = SmallVec::new();
    let mut right_verts: SmallVec<[Handle<Vert>; 4]> = SmallVec::new();

    for s in 0..=segments {
        let t = -(half) + s as f32 * step;
        left_verts.push(mesh.create_vert(co1 + offset_dir * t));
        right_verts.push(mesh.create_vert(co2 + offset_dir * t));
    }

    // Create quad faces between segments.
    for s in 0..segments as usize {
        let face_verts = [
            left_verts[s],
            right_verts[s],
            right_verts[s + 1],
            left_verts[s + 1],
        ];
        mesh.create_face(&face_verts)?;
    }

    Ok(())
}
