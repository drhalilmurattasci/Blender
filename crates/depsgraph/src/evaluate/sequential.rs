//! Sequential (single-threaded) evaluation of the dependency graph.

use crate::builder::Depsgraph;
use crate::DepsgraphResult;

/// Evaluate the graph sequentially in topological order.
///
/// Only evaluates nodes that have dirty tags set.
pub fn evaluate_sequential(
    graph: &mut Depsgraph,
    eval_fn: &dyn Fn(u32, &str) -> Result<(), String>,
) -> DepsgraphResult<()> {
    graph.reset_evaluated();

    let order = graph.eval_order.clone();

    for &node_id in &order {
        let idx = node_id as usize;
        if idx >= graph.nodes.len() {
            continue;
        }

        // Skip clean nodes.
        if graph.nodes[idx].dirty_tags.is_empty() {
            graph.nodes[idx].evaluated = true;
            continue;
        }

        let label = graph.nodes[idx].label.clone();

        tracing::trace!(node_id, label = %label, "evaluating node");

        match eval_fn(node_id, &label) {
            Ok(()) => {
                graph.nodes[idx].mark_clean();
            }
            Err(msg) => {
                tracing::error!(node_id, label = %label, error = %msg, "node evaluation failed");
                return Err(crate::DepsgraphError::EvalFailed(label, msg));
            }
        }
    }

    Ok(())
}
