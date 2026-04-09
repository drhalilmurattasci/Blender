//! Dependency graph nodes: represent evaluation units.

mod component;
mod id;

pub use component::{ComponentNode, ComponentType};
pub use id::{IdNode, IdType};

use crate::tag::DirtyTags;
use crate::NodeId;

/// A node in the dependency graph.
#[derive(Debug)]
pub struct DepNode {
    /// Unique ID within the graph.
    pub id: NodeId,
    /// Human-readable label (for debugging/tracing).
    pub label: String,
    /// Node type classification.
    pub node_type: DepNodeType,
    /// IDs of nodes this node depends on (must be evaluated first).
    pub dependencies: Vec<NodeId>,
    /// IDs of nodes that depend on this node.
    pub dependents: Vec<NodeId>,
    /// Dirty flags indicating what needs re-evaluation.
    pub dirty_tags: DirtyTags,
    /// Whether this node has been evaluated in the current cycle.
    pub evaluated: bool,
    /// Topological sort depth (distance from root).
    pub depth: u32,
}

impl DepNode {
    /// Create a new dependency graph node.
    pub fn new(id: NodeId, label: impl Into<String>, node_type: DepNodeType) -> Self {
        Self {
            id,
            label: label.into(),
            node_type,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            dirty_tags: DirtyTags::empty(),
            evaluated: false,
            depth: 0,
        }
    }

    /// Whether all dependencies have been evaluated.
    pub fn dependencies_satisfied(&self, nodes: &[DepNode]) -> bool {
        self.dependencies.iter().all(|&dep_id| {
            nodes
                .get(dep_id as usize)
                .is_some_and(|n| n.evaluated)
        })
    }

    /// Mark this node as needing re-evaluation.
    pub fn mark_dirty(&mut self, tags: DirtyTags) {
        self.dirty_tags |= tags;
        self.evaluated = false;
    }

    /// Clear dirty state after evaluation.
    pub fn mark_clean(&mut self) {
        self.dirty_tags = DirtyTags::empty();
        self.evaluated = true;
    }
}

/// Classification of dependency graph nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepNodeType {
    /// Root/time source node.
    TimeSource,
    /// ID data block (object, mesh, armature, etc.).
    Id(IdType),
    /// Component of an ID (transform, geometry, animation, etc.).
    Component(ComponentType),
    /// Operation within a component (specific evaluation step).
    Operation,
}
