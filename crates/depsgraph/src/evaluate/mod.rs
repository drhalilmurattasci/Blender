//! Dependency graph evaluation: traverse and evaluate dirty nodes.

mod sequential;

pub use sequential::evaluate_sequential;

use crate::builder::Depsgraph;
use crate::tag::DirtyTags;
use crate::DepsgraphResult;

/// Evaluate the dependency graph.
///
/// Evaluates all dirty nodes in topological order. If the graph needs
/// rebuilding, it will be rebuilt first.
///
/// The `eval_fn` callback is invoked for each dirty node that needs evaluation.
/// It receives the node's ID and label, and should perform the actual work
/// (e.g., evaluate animation, compute transform, run constraints).
pub fn evaluate_graph(
    graph: &mut Depsgraph,
    eval_fn: &dyn Fn(u32, &str) -> Result<(), String>,
) -> DepsgraphResult<()> {
    if graph.needs_rebuild {
        graph.compute_eval_order()?;
    }

    evaluate_sequential(graph, eval_fn)
}

/// Tag the entire graph as needing time update (frame changed).
pub fn tag_time_update(graph: &mut Depsgraph) {
    let tags = DirtyTags::TIME | DirtyTags::ANIMATION;

    // Find all time source nodes and tag them.
    let time_sources: Vec<u32> = graph
        .nodes
        .iter()
        .filter(|n| matches!(n.node_type, crate::node::DepNodeType::TimeSource))
        .map(|n| n.id)
        .collect();

    for id in time_sources {
        graph.tag_dirty(id, tags);
    }
}

/// Tag a specific ID node and its components as dirty.
pub fn tag_id_update(graph: &mut Depsgraph, id_name: &str, tags: DirtyTags) {
    if let Some(node_id) = graph.find_id(id_name) {
        graph.tag_dirty(node_id, tags);
    }
}
