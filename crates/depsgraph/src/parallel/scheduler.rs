//! Parallel scheduler: evaluates independent nodes concurrently.

use crate::builder::Depsgraph;
use crate::DepsgraphResult;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Evaluate the dependency graph in parallel using rayon.
///
/// Groups nodes by depth level and evaluates each level as a parallel batch.
/// Nodes at the same depth have no dependencies on each other and can safely
/// run concurrently.
///
/// The `eval_fn` must be `Sync` since it will be called from multiple threads.
pub fn evaluate_parallel(
    graph: &mut Depsgraph,
    eval_fn: &(dyn Fn(u32, &str) -> Result<(), String> + Sync),
) -> DepsgraphResult<()> {
    if graph.needs_rebuild {
        graph.compute_eval_order()?;
    }

    graph.reset_evaluated();

    // Group nodes by depth.
    let max_depth = graph
        .nodes
        .iter()
        .map(|n| n.depth)
        .max()
        .unwrap_or(0);

    let errors: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let evaluated_count = Arc::new(AtomicU32::new(0));

    for depth in 0..=max_depth {
        // Mark clean (non-dirty) nodes at this depth as evaluated so that
        // downstream `dependencies_satisfied` checks pass correctly.
        for node in graph.nodes.iter_mut().filter(|n| n.depth == depth && n.dirty_tags.is_empty()) {
            node.evaluated = true;
        }

        // Collect all dirty nodes at this depth.
        let nodes_at_depth: Vec<(u32, String)> = graph
            .nodes
            .iter()
            .filter(|n| n.depth == depth && !n.dirty_tags.is_empty())
            .map(|n| (n.id, n.label.clone()))
            .collect();

        if nodes_at_depth.is_empty() {
            continue;
        }

        let errors_clone = errors.clone();
        let evaluated_count_clone = evaluated_count.clone();

        rayon::scope(|s| {
            for (node_id, label) in &nodes_at_depth {
                let node_id = *node_id;
                let label = label.clone();
                let errors = errors_clone.clone();
                let evaluated_count = evaluated_count_clone.clone();

                s.spawn(move |_| {
                    tracing::trace!(node_id, label = %label, "parallel eval");

                    match eval_fn(node_id, &label) {
                        Ok(()) => {
                            evaluated_count.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(msg) => {
                            tracing::error!(node_id, label = %label, error = %msg, "parallel eval failed");
                            errors.lock().push((label, msg));
                        }
                    }
                });
            }
        });

        // Check for errors before marking nodes as evaluated.
        {
            let errs = errors.lock();
            if let Some((label, msg)) = errs.first() {
                return Err(crate::DepsgraphError::EvalFailed(
                    label.clone(),
                    msg.clone(),
                ));
            }
        }

        // Only mark nodes as evaluated after the entire depth level succeeded.
        for (node_id, _) in &nodes_at_depth {
            let idx = *node_id as usize;
            if idx < graph.nodes.len() {
                graph.nodes[idx].mark_clean();
            }
        }
    }

    tracing::debug!(
        evaluated = evaluated_count.load(Ordering::Relaxed),
        total = graph.nodes.len(),
        "parallel evaluation complete"
    );

    Ok(())
}
