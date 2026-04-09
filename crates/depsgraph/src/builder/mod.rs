//! Dependency graph builder: constructs the graph from scene data.

mod relations;

pub use relations::RelationBuilder;

use crate::node::{DepNode, DepNodeType, IdType};
use crate::tag::DirtyTags;
use crate::{DepsgraphResult, NodeId};
use indexmap::IndexMap;

/// The dependency graph: a directed acyclic graph of evaluation nodes.
#[derive(Debug)]
pub struct Depsgraph {
    /// All nodes in the graph, indexed by NodeId.
    pub nodes: Vec<DepNode>,
    /// Name-to-NodeId lookup for ID nodes.
    pub id_lookup: IndexMap<String, NodeId>,
    /// Current evaluation frame.
    pub current_frame: f32,
    /// Whether the graph needs a full rebuild.
    pub needs_rebuild: bool,
    /// Topologically sorted evaluation order.
    pub eval_order: Vec<NodeId>,
}

impl Depsgraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            id_lookup: IndexMap::new(),
            current_frame: 0.0,
            needs_rebuild: true,
            eval_order: Vec::new(),
        }
    }

    /// Add a node to the graph, returning its ID.
    pub fn add_node(&mut self, label: impl Into<String>, node_type: DepNodeType) -> NodeId {
        let id = self.nodes.len() as NodeId;
        let node = DepNode::new(id, label, node_type);
        self.nodes.push(node);
        id
    }

    /// Add an ID node and register it in the lookup table.
    pub fn add_id_node(&mut self, name: impl Into<String>, id_type: IdType) -> NodeId {
        let name = name.into();
        let label = format!("ID:{name}");
        let id = self.add_node(label, DepNodeType::Id(id_type));
        self.id_lookup.insert(name, id);
        id
    }

    /// Add a dependency edge: `from` depends on `to` (to must be evaluated first).
    pub fn add_dependency(&mut self, from: NodeId, to: NodeId) {
        let from_idx = from as usize;
        let to_idx = to as usize;
        if from_idx < self.nodes.len() && to_idx < self.nodes.len() {
            if !self.nodes[from_idx].dependencies.contains(&to) {
                self.nodes[from_idx].dependencies.push(to);
            }
            if !self.nodes[to_idx].dependents.contains(&from) {
                self.nodes[to_idx].dependents.push(from);
            }
        }
    }

    /// Look up an ID node by name.
    pub fn find_id(&self, name: &str) -> Option<NodeId> {
        self.id_lookup.get(name).copied()
    }

    /// Mark a node and its dependents as dirty.
    pub fn tag_dirty(&mut self, node_id: NodeId, tags: DirtyTags) {
        let mut stack = vec![(node_id, tags)];

        while let Some((id, tags)) = stack.pop() {
            let idx = id as usize;
            if idx >= self.nodes.len() {
                continue;
            }

            let node = &mut self.nodes[idx];
            let new_tags = tags;

            // Only propagate if we're adding new dirty flags.
            if node.dirty_tags.contains(new_tags) {
                continue;
            }

            node.mark_dirty(new_tags);

            // Propagate to dependents.
            let dependents = node.dependents.clone();
            for dep_id in dependents {
                stack.push((dep_id, new_tags));
            }
        }
    }

    /// Compute topological evaluation order using Kahn's algorithm.
    pub fn compute_eval_order(&mut self) -> DepsgraphResult<()> {
        let n = self.nodes.len();
        let mut in_degree = vec![0u32; n];

        // in_degree[i] = number of dependencies (predecessors) of node i.
        for i in 0..n {
            in_degree[i] = self.nodes[i].dependencies.len() as u32;
        }

        // Seed the queue with all nodes that have no dependencies (roots).
        // Use a VecDeque for proper FIFO (breadth-first) Kahn's traversal,
        // which ensures disconnected components are handled and depth
        // assignment is consistent.
        let mut queue: std::collections::VecDeque<NodeId> = std::collections::VecDeque::new();
        for i in 0..n {
            if in_degree[i] == 0 {
                queue.push_back(i as NodeId);
            }
        }

        let mut order = Vec::with_capacity(n);

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id);
            let idx = node_id as usize;

            let dependents = self.nodes[idx].dependents.clone();
            for dep_id in dependents {
                let dep_idx = dep_id as usize;
                if dep_idx < n {
                    in_degree[dep_idx] = in_degree[dep_idx].saturating_sub(1);
                    if in_degree[dep_idx] == 0 {
                        queue.push_back(dep_id);
                    }
                }
            }
        }

        if order.len() != n {
            // Find a node involved in a cycle for the error message.
            for i in 0..n {
                if in_degree[i] > 0 {
                    return Err(crate::DepsgraphError::CyclicDependency(
                        self.nodes[i].label.clone(),
                    ));
                }
            }
        }

        // Assign depths: root nodes (no dependencies) get depth 0.
        // Nodes with dependencies get max(dependency depths) + 1.
        for &node_id in &order {
            let idx = node_id as usize;
            if self.nodes[idx].dependencies.is_empty() {
                self.nodes[idx].depth = 0;
            } else {
                let max_dep_depth = self.nodes[idx]
                    .dependencies
                    .iter()
                    .filter_map(|&d| self.nodes.get(d as usize))
                    .map(|n| n.depth)
                    .max()
                    .unwrap_or(0);
                self.nodes[idx].depth = max_dep_depth + 1;
            }
        }

        self.eval_order = order;
        self.needs_rebuild = false;
        Ok(())
    }

    /// Number of nodes in the graph.
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Reset all nodes to unevaluated state.
    pub fn reset_evaluated(&mut self) {
        for node in &mut self.nodes {
            node.evaluated = false;
        }
    }
}

impl Default for Depsgraph {
    fn default() -> Self {
        Self::new()
    }
}
