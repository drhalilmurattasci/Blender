//! Node graph: the directed acyclic graph (DAG) of nodes and links.

use crate::nodes::Node;
use crate::sockets::SocketType;
use crate::GeoNodeResult;
use ahash::AHashMap;

/// Check whether an output socket type is compatible with an input socket type.
///
/// Geometry sockets only accept Geometry. Numeric types (Float, Int, Bool) are
/// implicitly convertible to each other. Vector and Color are interchangeable.
/// All other combinations are rejected.
fn sockets_compatible(from: SocketType, to: SocketType) -> bool {
    if from == to {
        return true;
    }
    match (from, to) {
        // Implicit numeric conversions.
        (SocketType::Float | SocketType::Int | SocketType::Bool,
         SocketType::Float | SocketType::Int | SocketType::Bool) => true,
        // Vector <-> Color (alpha filled / dropped).
        (SocketType::Vector, SocketType::Color) | (SocketType::Color, SocketType::Vector) => true,
        // Float can broadcast to Vector.
        (SocketType::Float, SocketType::Vector) | (SocketType::Float, SocketType::Color) => true,
        _ => false,
    }
}

/// Unique node identifier within a graph.
pub type NodeId = u64;

/// A directed link between an output socket and an input socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Link {
    /// Source node.
    pub from_node: NodeId,
    /// Index of the output socket on the source node.
    pub from_socket: usize,
    /// Destination node.
    pub to_node: NodeId,
    /// Index of the input socket on the destination node.
    pub to_socket: usize,
}

/// The geometry node graph.
#[derive(Debug)]
pub struct NodeGraph {
    /// All nodes keyed by their ID.
    nodes: AHashMap<NodeId, Node>,
    /// All links.
    links: Vec<Link>,
    /// Next auto-increment ID.
    next_id: NodeId,
    /// The group input node (entry point).
    pub group_input: Option<NodeId>,
    /// The group output node (exit point). Supports multiple outputs; the
    /// evaluator collects results from all of them.
    pub group_outputs: Vec<NodeId>,
}

impl NodeGraph {
    /// Create an empty node graph.
    pub fn new() -> Self {
        Self {
            nodes: AHashMap::new(),
            links: Vec::new(),
            next_id: 1,
            group_input: None,
            group_outputs: Vec::new(),
        }
    }

    /// Add a node to the graph. Returns the assigned [`NodeId`].
    pub fn add_node(&mut self, mut node: Node) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        node.id = id;
        self.nodes.insert(id, node);
        id
    }

    /// Remove a node and all its links.
    pub fn remove_node(&mut self, id: NodeId) -> Option<Node> {
        self.links.retain(|l| l.from_node != id && l.to_node != id);
        self.nodes.remove(&id)
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get a mutable node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// Add a link. Returns an error if socket types are incompatible or it
    /// would create a cycle.
    pub fn add_link(&mut self, link: Link) -> GeoNodeResult<()> {
        // Validate socket type compatibility.
        if let (Some(from_node), Some(to_node)) =
            (self.nodes.get(&link.from_node), self.nodes.get(&link.to_node))
        {
            if let (Some(out_socket), Some(in_socket)) = (
                from_node.outputs.get(link.from_socket),
                to_node.inputs.get(link.to_socket),
            ) {
                if !sockets_compatible(out_socket.socket_type, in_socket.socket_type) {
                    return Err(crate::GeoNodeError::TypeMismatch {
                        socket: in_socket.name.clone(),
                        expected: format!("{:?}", in_socket.socket_type),
                        actual: format!("{:?}", out_socket.socket_type),
                    });
                }
            }
        }

        // Remove any existing link to the same input socket.
        self.links.retain(|l| !(l.to_node == link.to_node && l.to_socket == link.to_socket));
        self.links.push(link);

        if self.has_cycle() {
            self.links.pop();
            Err(crate::GeoNodeError::CycleDetected)
        } else {
            Ok(())
        }
    }

    /// Remove a specific link.
    pub fn remove_link(&mut self, link: &Link) {
        self.links.retain(|l| l != link);
    }

    /// All links connected to a node's inputs.
    pub fn input_links(&self, node_id: NodeId) -> impl Iterator<Item = &Link> {
        self.links.iter().filter(move |l| l.to_node == node_id)
    }

    /// All links connected to a node's outputs.
    pub fn output_links(&self, node_id: NodeId) -> impl Iterator<Item = &Link> {
        self.links.iter().filter(move |l| l.from_node == node_id)
    }

    /// Iterate all nodes.
    pub fn nodes(&self) -> impl Iterator<Item = (&NodeId, &Node)> {
        self.nodes.iter()
    }

    /// Iterate all links.
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Produce a topological ordering of node IDs (for evaluation).
    ///
    /// Uses Kahn's algorithm with a BinaryHeap (min-heap via Reverse) for
    /// deterministic output: among ready nodes the smallest ID is dequeued
    /// first. This handles disconnected components and detects cycles.
    pub fn topological_order(&self) -> GeoNodeResult<Vec<NodeId>> {
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;

        let mut in_degree: AHashMap<NodeId, usize> = AHashMap::new();
        for &id in self.nodes.keys() {
            in_degree.insert(id, 0);
        }
        for link in &self.links {
            *in_degree.entry(link.to_node).or_insert(0) += 1;
        }

        // Seed with all zero-in-degree nodes. Using a min-heap (via Reverse)
        // guarantees deterministic ordering regardless of HashMap iteration.
        let mut heap: BinaryHeap<Reverse<NodeId>> = in_degree
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(id, _)| Reverse(*id))
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(Reverse(id)) = heap.pop() {
            order.push(id);
            for link in &self.links {
                if link.from_node == id {
                    if let Some(deg) = in_degree.get_mut(&link.to_node) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            heap.push(Reverse(link.to_node));
                        }
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            Err(crate::GeoNodeError::CycleDetected)
        } else {
            Ok(order)
        }
    }

    /// Simple cycle detection via DFS.
    fn has_cycle(&self) -> bool {
        self.topological_order().is_err()
    }
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}
