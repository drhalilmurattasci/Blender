//! # forge3d-mesh
//!
//! BMesh-style half-edge mesh data structure for the Forge3D engine.
//!
//! This crate provides the core `Mesh` type with generational-arena-backed
//! elements (vertices, edges, loops, faces) connected by disk, radial, and
//! face-loop cycles.

pub mod convert;
pub mod cycles;
pub mod elements;
pub mod iterators;
pub mod layers;
pub mod validate;

use forge3d_alloc::{Arena, Handle};
use forge3d_math::Vec3;
use smallvec::SmallVec;
use thiserror::Error;

pub use elements::{DiskLink, Edge, ElemFlags, Face, LoopElem, Vert};
pub use iterators::{EdgeFaces, FaceEdges, FaceLoops, FaceVerts, VertEdges, VertFaces};
pub use layers::{Layer, LayerData, LayerStack, LayerType};

/// Errors that can occur during mesh operations.
#[derive(Debug, Error)]
pub enum MeshError {
    /// A handle referred to an element that no longer exists.
    #[error("stale handle: {0}")]
    StaleHandle(String),

    /// The operation would create degenerate topology.
    #[error("degenerate topology: {0}")]
    DegenerateTopology(String),

    /// A face was specified with fewer than 3 vertices.
    #[error("face requires at least 3 vertices, got {0}")]
    TooFewVertices(usize),

    /// An edge already exists between the two vertices.
    #[error("duplicate edge between vertices")]
    DuplicateEdge,

    /// Generic validation failure.
    #[error("validation failed: {0}")]
    ValidationFailed(String),
}

/// The core BMesh-style mesh structure.
#[derive(Debug)]
pub struct Mesh {
    /// Vertex arena.
    pub verts: Arena<Vert>,
    /// Edge arena.
    pub edges: Arena<Edge>,
    /// Loop (corner) arena.
    pub loops: Arena<LoopElem>,
    /// Face arena.
    pub faces: Arena<Face>,

    /// Per-vertex custom data layers.
    pub vert_layers: LayerStack,
    /// Per-edge custom data layers.
    pub edge_layers: LayerStack,
    /// Per-loop custom data layers (UVs, vertex colors, etc.).
    pub loop_layers: LayerStack,
    /// Per-face custom data layers.
    pub face_layers: LayerStack,
}

// Static assertion: Mesh must be Send + Sync because it contains only owned
// arena data with no interior mutability, Rc, or raw pointers.
const _: fn() = || {
    fn must_be_send_sync<T: Send + Sync>() {}
    must_be_send_sync::<Mesh>();
};

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesh {
    /// Creates a new empty mesh.
    pub fn new() -> Self {
        Self {
            verts: Arena::new(),
            edges: Arena::new(),
            loops: Arena::new(),
            faces: Arena::new(),
            vert_layers: LayerStack::new(),
            edge_layers: LayerStack::new(),
            loop_layers: LayerStack::new(),
            face_layers: LayerStack::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Vertex operations
    // -----------------------------------------------------------------------

    /// Creates a new isolated vertex at the given position.
    pub fn create_vert(&mut self, co: Vec3) -> Handle<Vert> {
        let h = self.verts.insert(Vert::new(co));
        self.vert_layers.push_defaults();
        h
    }

    /// Removes a vertex and all edges/faces that reference it.
    pub fn kill_vert(&mut self, vert_h: Handle<Vert>) -> Result<(), MeshError> {
        if !self.verts.contains(vert_h) {
            return Err(MeshError::StaleHandle(format!("{:?}", vert_h)));
        }

        // Collect all edges connected to this vertex.
        let edge_handles: Vec<Handle<Edge>> = {
            let mut handles = Vec::new();
            if let Some(vert) = self.verts.get(vert_h) {
                if let Some(first_h) = vert.edge {
                    let mut cur_h = first_h;
                    loop {
                        handles.push(cur_h);
                        let edge = self.edges.get(cur_h).ok_or_else(|| {
                            MeshError::StaleHandle(format!("edge {:?}", cur_h))
                        })?;
                        cur_h = edge.disk_link(vert_h).next;
                        if cur_h == first_h {
                            break;
                        }
                    }
                }
            }
            handles
        };

        // Kill all connected edges (which also kills connected faces).
        for edge_h in edge_handles {
            if self.edges.contains(edge_h) {
                self.kill_edge(edge_h)?;
            }
        }

        self.verts.remove(vert_h);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Edge operations
    // -----------------------------------------------------------------------

    /// Creates an edge between two vertices.
    pub fn create_edge(
        &mut self,
        v1: Handle<Vert>,
        v2: Handle<Vert>,
    ) -> Result<Handle<Edge>, MeshError> {
        if !self.verts.contains(v1) {
            return Err(MeshError::StaleHandle(format!("v1 {:?}", v1)));
        }
        if !self.verts.contains(v2) {
            return Err(MeshError::StaleHandle(format!("v2 {:?}", v2)));
        }
        if v1 == v2 {
            return Err(MeshError::DegenerateTopology(
                "edge endpoints must be distinct".into(),
            ));
        }

        // Check for duplicate edge.
        if self.find_edge(v1, v2).is_some() {
            return Err(MeshError::DuplicateEdge);
        }

        let edge_h = self.edges.insert(Edge::new(v1, v2));
        self.edge_layers.push_defaults();

        // Insert into both disk cycles.
        cycles::disk::disk_append(&mut self.verts, &mut self.edges, v1, edge_h);
        cycles::disk::disk_append(&mut self.verts, &mut self.edges, v2, edge_h);

        Ok(edge_h)
    }

    /// Removes an edge and all faces that reference it.
    pub fn kill_edge(&mut self, edge_h: Handle<Edge>) -> Result<(), MeshError> {
        let edge = self
            .edges
            .get(edge_h)
            .ok_or_else(|| MeshError::StaleHandle(format!("{:?}", edge_h)))?;

        let v1 = edge.v1;
        let v2 = edge.v2;

        // Collect faces that use this edge (via radial cycle).
        let face_handles: Vec<Handle<Face>> = {
            let mut faces = Vec::new();
            if let Some(first_lh) = edge.loop_first {
                let mut cur_lh = first_lh;
                loop {
                    if let Some(l) = self.loops.get(cur_lh) {
                        if !faces.contains(&l.face) {
                            faces.push(l.face);
                        }
                        cur_lh = l.radial_next;
                        if cur_lh == first_lh {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            faces
        };

        // Kill all connected faces.
        for face_h in face_handles {
            if self.faces.contains(face_h) {
                self.kill_face(face_h)?;
            }
        }

        // Remove from disk cycles.
        cycles::disk::disk_remove(&mut self.verts, &mut self.edges, v1, edge_h);
        cycles::disk::disk_remove(&mut self.verts, &mut self.edges, v2, edge_h);

        self.edges.remove(edge_h);
        Ok(())
    }

    /// Finds an existing edge between two vertices, if any.
    pub fn find_edge(&self, v1: Handle<Vert>, v2: Handle<Vert>) -> Option<Handle<Edge>> {
        let vert = self.verts.get(v1)?;
        let first_h = vert.edge?;

        let mut cur_h = first_h;
        loop {
            let edge = self.edges.get(cur_h)?;
            if edge.other_vert(v1) == v2 {
                return Some(cur_h);
            }
            cur_h = edge.disk_link(v1).next;
            if cur_h == first_h {
                break;
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Face operations
    // -----------------------------------------------------------------------

    /// Creates a face from an ordered list of vertex handles.
    ///
    /// Edges between consecutive vertices are created if they do not already
    /// exist. The face's loop cycle is built and wired into the radial cycles.
    /// Works for any N-gon (N >= 3).
    pub fn create_face(
        &mut self,
        vert_handles: &[Handle<Vert>],
    ) -> Result<Handle<Face>, MeshError> {
        let n = vert_handles.len();
        if n < 3 {
            return Err(MeshError::TooFewVertices(n));
        }

        // Validate all vertex handles and check for duplicates.
        for (i, &vh) in vert_handles.iter().enumerate() {
            if !self.verts.contains(vh) {
                return Err(MeshError::StaleHandle(format!("{:?}", vh)));
            }
            for &vh2 in &vert_handles[i + 1..] {
                if vh == vh2 {
                    return Err(MeshError::DegenerateTopology(
                        "duplicate vertex in face".into(),
                    ));
                }
            }
        }

        // Ensure edges exist between consecutive vertices.
        let mut edge_handles: SmallVec<[Handle<Edge>; 8]> = SmallVec::new();
        for i in 0..n {
            let v_cur = vert_handles[i];
            let v_next = vert_handles[(i + 1) % n];
            let edge_h = match self.find_edge(v_cur, v_next) {
                Some(h) => h,
                None => self.create_edge(v_cur, v_next)?,
            };
            edge_handles.push(edge_h);
        }

        // Create the face with a placeholder loop_first (will set below).
        let face_h = self.faces.insert(Face::new(Handle::dangling(), n as u32));
        self.face_layers.push_defaults();

        // Create loops and build the face loop cycle.
        let mut loop_handles: SmallVec<[Handle<LoopElem>; 8]> = SmallVec::new();
        for i in 0..n {
            let l = LoopElem::new(vert_handles[i], edge_handles[i], face_h);
            let lh = self.loops.insert(l);
            self.loop_layers.push_defaults();
            loop_handles.push(lh);
        }

        // Wire up the face loop cycle (circular doubly-linked list).
        for i in 0..n {
            let next_i = (i + 1) % n;
            let prev_i = (i + n - 1) % n;
            let l = self.loops.get_mut(loop_handles[i]).unwrap();
            l.next = loop_handles[next_i];
            l.prev = loop_handles[prev_i];
        }

        // Set the face's first loop.
        self.faces.get_mut(face_h).unwrap().loop_first = loop_handles[0];

        // Wire loops into the radial cycles of their edges.
        for i in 0..n {
            cycles::radial::radial_append(
                &mut self.edges,
                &mut self.loops,
                edge_handles[i],
                loop_handles[i],
            );
        }

        Ok(face_h)
    }

    /// Removes a face and its loops from the mesh. Edges and vertices are kept.
    pub fn kill_face(&mut self, face_h: Handle<Face>) -> Result<(), MeshError> {
        let face = self
            .faces
            .get(face_h)
            .ok_or_else(|| MeshError::StaleHandle(format!("{:?}", face_h)))?;

        let first_lh = face.loop_first;
        if first_lh.is_dangling() {
            self.faces.remove(face_h);
            return Ok(());
        }

        // Collect all loops in this face.
        let mut loop_handles = Vec::new();
        let mut cur_lh = first_lh;
        loop {
            loop_handles.push(cur_lh);
            let l = self.loops.get(cur_lh).ok_or_else(|| {
                MeshError::StaleHandle(format!("loop {:?}", cur_lh))
            })?;
            cur_lh = l.next;
            if cur_lh == first_lh {
                break;
            }
        }

        // Remove each loop from its radial cycle, then free it.
        for &lh in &loop_handles {
            let edge_h = self.loops.get(lh).unwrap().edge;
            cycles::radial::radial_remove(&mut self.edges, &mut self.loops, edge_h, lh);
            self.loops.remove(lh);
        }

        self.faces.remove(face_h);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Returns the total number of vertices.
    pub fn vert_count(&self) -> usize {
        self.verts.len()
    }

    /// Returns the total number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns the total number of loops.
    pub fn loop_count(&self) -> usize {
        self.loops.len()
    }

    /// Returns the total number of faces.
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Runs a full validation pass and returns any errors.
    pub fn validate(&self) -> Vec<String> {
        validate::validate(&self.verts, &self.edges, &self.loops, &self.faces)
    }
}
