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
pub use fields::{
    BinaryMathOp, Field, FieldDataType, FieldDomain, FieldValue, UnaryMathOp,
};
pub use graph::{Link, NodeGraph, NodeId};
pub use nodes::{Node, NodeCategory, NodeType};
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_node(nt: NodeType, label: &str) -> Node {
        Node::new(nt, label, NodeCategory::Utilities)
    }

    // ---------------------------------------------------------------
    // Empty graph
    // ---------------------------------------------------------------

    #[test]
    fn empty_graph_topo_order() {
        let g = NodeGraph::new();
        let order = g.topological_order().unwrap();
        assert!(order.is_empty());
    }

    // ---------------------------------------------------------------
    // Single node
    // ---------------------------------------------------------------

    #[test]
    fn single_node_topo_order() {
        let mut g = NodeGraph::new();
        let id = g.add_node(make_test_node(NodeType::MathNode, "Math"));
        let order = g.topological_order().unwrap();
        assert_eq!(order, vec![id]);
    }

    // ---------------------------------------------------------------
    // Linear chain
    // ---------------------------------------------------------------

    #[test]
    fn linear_chain() {
        let mut g = NodeGraph::new();
        let a = g.add_node(
            make_test_node(NodeType::GroupInput, "In")
                .with_output(Socket::output("Geo", SocketType::Geometry)),
        );
        let b = g.add_node(
            make_test_node(NodeType::TransformGeometry, "Transform")
                .with_input(Socket::input("Geo", SocketType::Geometry, SocketValue::Geometry))
                .with_output(Socket::output("Geo", SocketType::Geometry)),
        );
        let c = g.add_node(
            make_test_node(NodeType::GroupOutput, "Out")
                .with_input(Socket::input("Geo", SocketType::Geometry, SocketValue::Geometry)),
        );

        g.add_link(Link { from_node: a, from_socket: 0, to_node: b, to_socket: 0 }).unwrap();
        g.add_link(Link { from_node: b, from_socket: 0, to_node: c, to_socket: 0 }).unwrap();

        let order = g.topological_order().unwrap();
        let pos_a = order.iter().position(|&x| x == a).unwrap();
        let pos_b = order.iter().position(|&x| x == b).unwrap();
        let pos_c = order.iter().position(|&x| x == c).unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    // ---------------------------------------------------------------
    // Cycle detection
    // ---------------------------------------------------------------

    #[test]
    fn cycle_detected() {
        let mut g = NodeGraph::new();
        let a = g.add_node(
            make_test_node(NodeType::MathNode, "A")
                .with_input(Socket::input("In", SocketType::Float, SocketValue::Float(0.0)))
                .with_output(Socket::output("Out", SocketType::Float)),
        );
        let b = g.add_node(
            make_test_node(NodeType::MathNode, "B")
                .with_input(Socket::input("In", SocketType::Float, SocketValue::Float(0.0)))
                .with_output(Socket::output("Out", SocketType::Float)),
        );

        g.add_link(Link { from_node: a, from_socket: 0, to_node: b, to_socket: 0 }).unwrap();
        let err = g.add_link(Link { from_node: b, from_socket: 0, to_node: a, to_socket: 0 });
        assert!(matches!(err, Err(GeoNodeError::CycleDetected)));
    }

    #[test]
    fn cycle_link_is_rolled_back() {
        let mut g = NodeGraph::new();
        let a = g.add_node(
            make_test_node(NodeType::MathNode, "A")
                .with_input(Socket::input("In", SocketType::Float, SocketValue::Float(0.0)))
                .with_output(Socket::output("Out", SocketType::Float)),
        );
        let b = g.add_node(
            make_test_node(NodeType::MathNode, "B")
                .with_input(Socket::input("In", SocketType::Float, SocketValue::Float(0.0)))
                .with_output(Socket::output("Out", SocketType::Float)),
        );

        g.add_link(Link { from_node: a, from_socket: 0, to_node: b, to_socket: 0 }).unwrap();
        let _ = g.add_link(Link { from_node: b, from_socket: 0, to_node: a, to_socket: 0 });

        // The graph should still be acyclic (the bad link was removed).
        g.topological_order().expect("graph should be acyclic after rollback");
        assert_eq!(g.links().len(), 1);
    }

    // ---------------------------------------------------------------
    // Socket type compatibility
    // ---------------------------------------------------------------

    #[test]
    fn type_mismatch_rejected() {
        let mut g = NodeGraph::new();
        let a = g.add_node(
            make_test_node(NodeType::MathNode, "A")
                .with_output(Socket::output("Out", SocketType::Geometry)),
        );
        let b = g.add_node(
            make_test_node(NodeType::MathNode, "B")
                .with_input(Socket::input("In", SocketType::Float, SocketValue::Float(0.0))),
        );

        let err = g.add_link(Link { from_node: a, from_socket: 0, to_node: b, to_socket: 0 });
        assert!(matches!(err, Err(GeoNodeError::TypeMismatch { .. })));
    }

    #[test]
    fn numeric_implicit_conversion_allowed() {
        let mut g = NodeGraph::new();
        let a = g.add_node(
            make_test_node(NodeType::MathNode, "A")
                .with_output(Socket::output("Out", SocketType::Int)),
        );
        let b = g.add_node(
            make_test_node(NodeType::MathNode, "B")
                .with_input(Socket::input("In", SocketType::Float, SocketValue::Float(0.0))),
        );

        g.add_link(Link { from_node: a, from_socket: 0, to_node: b, to_socket: 0 })
            .expect("Int -> Float should be allowed");
    }

    #[test]
    fn float_to_vector_broadcast_allowed() {
        let mut g = NodeGraph::new();
        let a = g.add_node(
            make_test_node(NodeType::MathNode, "A")
                .with_output(Socket::output("Out", SocketType::Float)),
        );
        let b = g.add_node(
            make_test_node(NodeType::MathNode, "B")
                .with_input(Socket::input("In", SocketType::Vector, SocketValue::Vector([0.0; 3]))),
        );

        g.add_link(Link { from_node: a, from_socket: 0, to_node: b, to_socket: 0 })
            .expect("Float -> Vector broadcast should be allowed");
    }

    // ---------------------------------------------------------------
    // Remove node cleans up group references
    // ---------------------------------------------------------------

    #[test]
    fn remove_node_cleans_group_refs() {
        let mut g = NodeGraph::new();
        let gi = g.add_node(make_test_node(NodeType::GroupInput, "In"));
        let go = g.add_node(make_test_node(NodeType::GroupOutput, "Out"));
        g.group_input = Some(gi);
        g.group_outputs.push(go);

        g.remove_node(gi);
        assert_eq!(g.group_input, None);

        g.remove_node(go);
        assert!(g.group_outputs.is_empty());
    }

    // ---------------------------------------------------------------
    // Replacing a link to the same input socket
    // ---------------------------------------------------------------

    #[test]
    fn replacing_link_removes_old() {
        let mut g = NodeGraph::new();
        let a = g.add_node(
            make_test_node(NodeType::MathNode, "A")
                .with_output(Socket::output("Out", SocketType::Float)),
        );
        let b = g.add_node(
            make_test_node(NodeType::MathNode, "B")
                .with_output(Socket::output("Out", SocketType::Float)),
        );
        let c = g.add_node(
            make_test_node(NodeType::MathNode, "C")
                .with_input(Socket::input("In", SocketType::Float, SocketValue::Float(0.0))),
        );

        g.add_link(Link { from_node: a, from_socket: 0, to_node: c, to_socket: 0 }).unwrap();
        g.add_link(Link { from_node: b, from_socket: 0, to_node: c, to_socket: 0 }).unwrap();

        // Only one link should remain (the second one replaced the first).
        assert_eq!(g.links().len(), 1);
        assert_eq!(g.links()[0].from_node, b);
    }

    // ---------------------------------------------------------------
    // Evaluator basic test
    // ---------------------------------------------------------------

    #[test]
    fn evaluator_empty_graph() {
        let g = NodeGraph::new();
        let mut eval = Evaluator::new();
        let result = eval.evaluate(&g, 1.0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn evaluator_passthrough() {
        let mut g = NodeGraph::new();
        let gi = g.add_node(
            make_test_node(NodeType::GroupInput, "In")
                .with_output(Socket::output("Value", SocketType::Float)),
        );
        let go = g.add_node(
            make_test_node(NodeType::GroupOutput, "Out")
                .with_input(Socket::input("Value", SocketType::Float, SocketValue::Float(42.0))),
        );
        g.group_outputs.push(go);
        g.add_link(Link { from_node: gi, from_socket: 0, to_node: go, to_socket: 0 }).unwrap();

        let mut eval = Evaluator::new();
        let result = eval.evaluate(&g, 1.0).unwrap();
        // The pass-through evaluator copies inputs to outputs.
        assert!(result.contains_key(&0));
    }

    // ---------------------------------------------------------------
    // FieldValue default
    // ---------------------------------------------------------------

    #[test]
    fn field_value_default() {
        let v = FieldValue::default();
        assert_eq!(v, FieldValue::Float(0.0));
    }

    // ---------------------------------------------------------------
    // SocketValue conversions
    // ---------------------------------------------------------------

    #[test]
    fn socket_value_as_float() {
        assert_eq!(SocketValue::Float(1.5).as_float(), Some(1.5));
        assert_eq!(SocketValue::Int(3).as_float(), Some(3.0));
        assert_eq!(SocketValue::Bool(true).as_float(), Some(1.0));
        assert_eq!(SocketValue::Bool(false).as_float(), Some(0.0));
        assert_eq!(SocketValue::Geometry.as_float(), None);
    }

    #[test]
    fn socket_value_as_vector() {
        assert_eq!(
            SocketValue::Vector([1.0, 2.0, 3.0]).as_vector(),
            Some([1.0, 2.0, 3.0])
        );
        assert_eq!(
            SocketValue::Float(5.0).as_vector(),
            Some([5.0, 5.0, 5.0])
        );
        assert_eq!(SocketValue::Int(1).as_vector(), None);
    }
}
