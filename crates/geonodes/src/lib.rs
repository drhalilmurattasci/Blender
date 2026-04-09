//! # forge3d-geonodes
//!
//! Geometry Nodes system for Forge3D: a visual node-graph that processes
//! geometry through composable operations. Includes fields (lazy per-element
//! evaluation), sockets, nodes, graph structure, and a parallel evaluator.

pub mod evaluation;
pub mod fields;
pub mod graph;
pub mod nodes;
pub mod sockets;

pub use evaluation::{EvalContext, Evaluator};
pub use fields::{Field, FieldDomain};
pub use graph::{NodeGraph, NodeId};
pub use nodes::{Node, NodeType};
pub use sockets::{Socket, SocketDirection, SocketType, SocketValue};

use thiserror::Error;

/// Errors from geometry-node evaluation.
#[derive(Debug, Error)]
pub enum GeoNodeError {
    #[error("node {node_id} has no input for socket `{socket}`")]
    MissingInput { node_id: u64, socket: String },

    #[error("type mismatch: socket `{socket}` expected {expected}, got {actual}")]
    TypeMismatch {
        socket: String,
        expected: String,
        actual: String,
    },

    #[error("graph contains a cycle")]
    CycleDetected,

    #[error("node type `{0}` is not registered")]
    UnknownNodeType(String),

    #[error("field evaluation failed: {0}")]
    FieldError(String),

    #[error("evaluation cancelled")]
    Cancelled,
}

/// Result alias.
pub type GeoNodeResult<T> = Result<T, GeoNodeError>;
