//! Evaluation engine: traverses the node graph in topological order and
//! computes socket values, optionally parallelised with rayon.

use crate::graph::{NodeGraph, NodeId};
use crate::sockets::SocketValue;
use crate::GeoNodeResult;
use ahash::AHashMap;

/// Per-node evaluation context.
#[derive(Debug)]
pub struct EvalContext {
    /// Resolved input values for the current node, keyed by socket index.
    pub inputs: AHashMap<usize, SocketValue>,
    /// Output values produced by the current node, keyed by socket index.
    pub outputs: AHashMap<usize, SocketValue>,
    /// Current frame time.
    pub frame: f32,
}

impl EvalContext {
    /// Create a new context for a given frame.
    pub fn new(frame: f32) -> Self {
        Self {
            inputs: AHashMap::new(),
            outputs: AHashMap::new(),
            frame,
        }
    }

    /// Get an input value by socket index.
    pub fn input(&self, index: usize) -> Option<&SocketValue> {
        self.inputs.get(&index)
    }

    /// Get an input float, with a fallback default.
    pub fn input_float(&self, index: usize, default: f32) -> f32 {
        self.inputs
            .get(&index)
            .and_then(|v| v.as_float())
            .unwrap_or(default)
    }

    /// Get an input vector, with a fallback default.
    pub fn input_vector(&self, index: usize, default: [f32; 3]) -> [f32; 3] {
        self.inputs
            .get(&index)
            .and_then(|v| v.as_vector())
            .unwrap_or(default)
    }

    /// Set an output value.
    pub fn set_output(&mut self, index: usize, value: SocketValue) {
        self.outputs.insert(index, value);
    }
}

/// The graph evaluator.
#[derive(Debug)]
pub struct Evaluator {
    /// Cached topological order from the last evaluation.
    topo_cache: Vec<NodeId>,
    /// Per-node output cache.
    output_cache: AHashMap<NodeId, AHashMap<usize, SocketValue>>,
}

impl Evaluator {
    /// Create a new evaluator.
    pub fn new() -> Self {
        Self {
            topo_cache: Vec::new(),
            output_cache: AHashMap::new(),
        }
    }

    /// Evaluate the entire graph for a given frame.
    ///
    /// Returns the outputs of the group-output node, if present.
    pub fn evaluate(
        &mut self,
        graph: &NodeGraph,
        frame: f32,
    ) -> GeoNodeResult<AHashMap<usize, SocketValue>> {
        self.topo_cache = graph.topological_order()?;
        self.output_cache.clear();

        for &node_id in &self.topo_cache {
            let node = graph
                .get_node(node_id)
                .ok_or_else(|| crate::GeoNodeError::UnknownNodeType(format!("id={node_id}")))?;

            let mut ctx = EvalContext::new(frame);

            // Resolve inputs from upstream output caches.
            for link in graph.input_links(node_id) {
                if let Some(upstream) = self.output_cache.get(&link.from_node) {
                    if let Some(val) = upstream.get(&link.from_socket) {
                        ctx.inputs.insert(link.to_socket, val.clone());
                    }
                }
            }

            // Fill unconnected inputs with defaults.
            for (i, socket) in node.inputs.iter().enumerate() {
                ctx.inputs
                    .entry(i)
                    .or_insert_with(|| socket.default_value.clone());
            }

            // Node-type-specific evaluation would go here.
            // For now, pass through geometry and propagate values.
            self.evaluate_node(node_id, &node.node_type, &mut ctx)?;

            self.output_cache.insert(node_id, ctx.outputs);
        }

        // Collect outputs from all group output nodes. If there are
        // multiple group outputs the results are merged (later outputs
        // overwrite earlier ones for the same socket index).
        let mut result = AHashMap::new();
        for &out_id in &graph.group_outputs {
            if let Some(outputs) = self.output_cache.remove(&out_id) {
                result.extend(outputs);
            }
        }
        Ok(result)
    }

    /// Placeholder per-node evaluation dispatch.
    fn evaluate_node(
        &self,
        _node_id: NodeId,
        _node_type: &crate::nodes::NodeType,
        ctx: &mut EvalContext,
    ) -> GeoNodeResult<()> {
        // In a full implementation this would dispatch to per-NodeType
        // evaluation functions. For now, copy inputs to outputs 1:1
        // (pass-through behaviour).
        for (i, val) in ctx.inputs.iter() {
            ctx.outputs.insert(*i, val.clone());
        }
        Ok(())
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
