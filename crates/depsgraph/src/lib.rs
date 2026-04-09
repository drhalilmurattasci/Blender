//! # forge3d-depsgraph
//!
//! Dependency graph for Forge3D.
//!
//! The depsgraph tracks relationships between scene data (objects, bones,
//! constraints, modifiers, etc.) and evaluates them in topologically-sorted
//! order with parallel execution via rayon.

pub mod builder;
pub mod evaluate;
pub mod node;
pub mod parallel;
pub mod tag;

pub use builder::{Depsgraph, RelationBuilder, ObjectNodes, ArmatureNodes};
pub use evaluate::{evaluate_graph, tag_id_update, tag_time_update};
pub use node::{ComponentNode, ComponentType, DepNode, DepNodeType, IdNode, IdType};
pub use parallel::evaluate_parallel;
pub use tag::{DirtyTags, UpdateTag};

use thiserror::Error;

/// Errors from the dependency graph.
#[derive(Debug, Error)]
pub enum DepsgraphError {
    #[error("node `{0}` not found in the dependency graph")]
    NodeNotFound(String),

    #[error("cyclic dependency detected involving node `{0}`")]
    CyclicDependency(String),

    #[error("evaluation failed for node `{0}`: {1}")]
    EvalFailed(String, String),

    #[error("graph build error: {0}")]
    BuildError(String),
}

pub type DepsgraphResult<T> = Result<T, DepsgraphError>;

/// Unique identifier for a node in the dependency graph.
pub type NodeId = u32;

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Empty graph
    // ---------------------------------------------------------------

    #[test]
    fn empty_graph_eval_order() {
        let mut g = Depsgraph::new();
        g.compute_eval_order().expect("empty graph should succeed");
        assert!(g.eval_order.is_empty());
        assert_eq!(g.node_count(), 0);
    }

    #[test]
    fn empty_graph_evaluate() {
        let mut g = Depsgraph::new();
        g.compute_eval_order().unwrap();
        evaluate::evaluate_graph(&mut g, &|_id, _label| Ok(()))
            .expect("evaluating empty graph should succeed");
    }

    // ---------------------------------------------------------------
    // Single node
    // ---------------------------------------------------------------

    #[test]
    fn single_node_eval_order() {
        let mut g = Depsgraph::new();
        let _n = g.add_node("Root", node::DepNodeType::TimeSource);
        g.compute_eval_order().unwrap();
        assert_eq!(g.eval_order.len(), 1);
        assert_eq!(g.nodes[0].depth, 0);
    }

    // ---------------------------------------------------------------
    // Linear chain: A -> B -> C
    // ---------------------------------------------------------------

    #[test]
    fn linear_chain_order() {
        let mut g = Depsgraph::new();
        let a = g.add_node("A", node::DepNodeType::TimeSource);
        let b = g.add_node("B", node::DepNodeType::Operation);
        let c = g.add_node("C", node::DepNodeType::Operation);
        g.add_dependency(b, a); // B depends on A
        g.add_dependency(c, b); // C depends on B
        g.compute_eval_order().unwrap();

        // Order must be A, B, C.
        assert_eq!(g.eval_order, vec![a, b, c]);
        assert_eq!(g.nodes[a as usize].depth, 0);
        assert_eq!(g.nodes[b as usize].depth, 1);
        assert_eq!(g.nodes[c as usize].depth, 2);
    }

    // ---------------------------------------------------------------
    // Diamond: A -> B, A -> C, B -> D, C -> D
    // ---------------------------------------------------------------

    #[test]
    fn diamond_order() {
        let mut g = Depsgraph::new();
        let a = g.add_node("A", node::DepNodeType::TimeSource);
        let b = g.add_node("B", node::DepNodeType::Operation);
        let c = g.add_node("C", node::DepNodeType::Operation);
        let d = g.add_node("D", node::DepNodeType::Operation);

        g.add_dependency(b, a);
        g.add_dependency(c, a);
        g.add_dependency(d, b);
        g.add_dependency(d, c);
        g.compute_eval_order().unwrap();

        // A must come before B and C; D must be last.
        let pos_a = g.eval_order.iter().position(|&x| x == a).unwrap();
        let pos_b = g.eval_order.iter().position(|&x| x == b).unwrap();
        let pos_c = g.eval_order.iter().position(|&x| x == c).unwrap();
        let pos_d = g.eval_order.iter().position(|&x| x == d).unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
        assert_eq!(g.nodes[d as usize].depth, 2);
    }

    // ---------------------------------------------------------------
    // Cycle detection
    // ---------------------------------------------------------------

    #[test]
    fn cycle_detected_two_nodes() {
        let mut g = Depsgraph::new();
        let a = g.add_node("A", node::DepNodeType::Operation);
        let b = g.add_node("B", node::DepNodeType::Operation);
        g.add_dependency(a, b);
        g.add_dependency(b, a);

        let err = g.compute_eval_order().unwrap_err();
        assert!(
            matches!(err, DepsgraphError::CyclicDependency(_)),
            "expected CyclicDependency, got {err:?}"
        );
    }

    #[test]
    fn cycle_detected_three_nodes() {
        let mut g = Depsgraph::new();
        let a = g.add_node("A", node::DepNodeType::Operation);
        let b = g.add_node("B", node::DepNodeType::Operation);
        let c = g.add_node("C", node::DepNodeType::Operation);
        g.add_dependency(b, a);
        g.add_dependency(c, b);
        g.add_dependency(a, c); // Creates cycle A -> C -> B -> A.

        let err = g.compute_eval_order().unwrap_err();
        assert!(matches!(err, DepsgraphError::CyclicDependency(_)));
    }

    #[test]
    fn cycle_in_subgraph_detected() {
        // Disconnected component with a cycle should still be detected.
        let mut g = Depsgraph::new();
        let _root = g.add_node("Root", node::DepNodeType::TimeSource);
        let a = g.add_node("A", node::DepNodeType::Operation);
        let b = g.add_node("B", node::DepNodeType::Operation);
        g.add_dependency(a, b);
        g.add_dependency(b, a);

        let err = g.compute_eval_order().unwrap_err();
        assert!(matches!(err, DepsgraphError::CyclicDependency(_)));
    }

    // ---------------------------------------------------------------
    // Self-cycle
    // ---------------------------------------------------------------

    #[test]
    fn self_cycle_detected() {
        let mut g = Depsgraph::new();
        let a = g.add_node("A", node::DepNodeType::Operation);
        g.add_dependency(a, a);

        let err = g.compute_eval_order().unwrap_err();
        assert!(matches!(err, DepsgraphError::CyclicDependency(_)));
    }

    // ---------------------------------------------------------------
    // Dirty tag propagation
    // ---------------------------------------------------------------

    #[test]
    fn dirty_propagates_to_dependents() {
        let mut g = Depsgraph::new();
        let a = g.add_node("A", node::DepNodeType::TimeSource);
        let b = g.add_node("B", node::DepNodeType::Operation);
        let c = g.add_node("C", node::DepNodeType::Operation);
        g.add_dependency(b, a);
        g.add_dependency(c, b);

        g.tag_dirty(a, DirtyTags::TRANSFORM);

        assert!(g.nodes[a as usize].dirty_tags.contains(DirtyTags::TRANSFORM));
        assert!(g.nodes[b as usize].dirty_tags.contains(DirtyTags::TRANSFORM));
        assert!(g.nodes[c as usize].dirty_tags.contains(DirtyTags::TRANSFORM));
    }

    // ---------------------------------------------------------------
    // Duplicate dependency edges are ignored
    // ---------------------------------------------------------------

    #[test]
    fn duplicate_dependency_is_idempotent() {
        let mut g = Depsgraph::new();
        let a = g.add_node("A", node::DepNodeType::TimeSource);
        let b = g.add_node("B", node::DepNodeType::Operation);
        g.add_dependency(b, a);
        g.add_dependency(b, a); // duplicate

        assert_eq!(g.nodes[b as usize].dependencies.len(), 1);
        assert_eq!(g.nodes[a as usize].dependents.len(), 1);
    }

    // ---------------------------------------------------------------
    // Sequential evaluation: eval_fn is called for dirty nodes only
    // ---------------------------------------------------------------

    #[test]
    fn sequential_eval_skips_clean_nodes() {
        use std::sync::atomic::{AtomicU32, Ordering};

        let mut g = Depsgraph::new();
        let a = g.add_node("A", node::DepNodeType::TimeSource);
        let b = g.add_node("B", node::DepNodeType::Operation);
        g.add_dependency(b, a);
        g.compute_eval_order().unwrap();

        // Only mark A dirty.
        g.tag_dirty(a, DirtyTags::TIME);

        let count = AtomicU32::new(0);
        evaluate::evaluate_graph(&mut g, &|_id, _label| {
            count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })
        .unwrap();

        // Both A and B should be evaluated because dirty propagates.
        assert_eq!(count.load(Ordering::Relaxed), 2);
    }

    // ---------------------------------------------------------------
    // Relation builder produces valid graph
    // ---------------------------------------------------------------

    #[test]
    fn relation_builder_object_graph_is_acyclic() {
        let mut g = Depsgraph::new();
        {
            let mut rb = RelationBuilder::new(&mut g);
            let _ts = rb.add_time_source();
            let _obj = rb.add_object("Cube", true);
        }
        g.compute_eval_order()
            .expect("object graph from RelationBuilder should be acyclic");
    }

    #[test]
    fn relation_builder_armature_graph_is_acyclic() {
        let mut g = Depsgraph::new();
        {
            let mut rb = RelationBuilder::new(&mut g);
            let _ts = rb.add_time_source();
            let _arm = rb.add_armature_object("Armature", &["Root", "Spine", "Head"]);
        }
        g.compute_eval_order()
            .expect("armature graph from RelationBuilder should be acyclic");
    }

    // ---------------------------------------------------------------
    // Disconnected components are handled
    // ---------------------------------------------------------------

    #[test]
    fn disconnected_components() {
        let mut g = Depsgraph::new();
        let a = g.add_node("A", node::DepNodeType::TimeSource);
        let b = g.add_node("B", node::DepNodeType::Operation);
        // No edges -- two disconnected components.
        g.compute_eval_order().unwrap();
        assert_eq!(g.eval_order.len(), 2);
        assert!(g.eval_order.contains(&a));
        assert!(g.eval_order.contains(&b));
    }

    // ---------------------------------------------------------------
    // ID lookup
    // ---------------------------------------------------------------

    #[test]
    fn id_lookup_works() {
        let mut g = Depsgraph::new();
        let id = g.add_id_node("Cube", node::IdType::Object);
        assert_eq!(g.find_id("Cube"), Some(id));
        assert_eq!(g.find_id("Missing"), None);
    }
}
