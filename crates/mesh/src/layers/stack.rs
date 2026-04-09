//! Layer stack — manages multiple custom data layers for a given element domain.

use serde::{Deserialize, Serialize};

use super::layer::Layer;
use super::types::LayerType;

/// A collection of custom data layers for a single element domain
/// (e.g. all per-vertex layers, all per-loop layers, etc.).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayerStack {
    layers: Vec<Layer>,
}

impl LayerStack {
    /// Creates an empty layer stack.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
        }
    }

    /// Adds a new layer with the given name and type.
    /// Returns the index of the newly added layer.
    pub fn add_layer(&mut self, name: impl Into<String>, layer_type: LayerType) -> usize {
        let idx = self.layers.len();
        self.layers.push(Layer::new(name, layer_type));
        idx
    }

    /// Removes the layer at the given index.
    pub fn remove_layer(&mut self, index: usize) -> Option<Layer> {
        if index < self.layers.len() {
            Some(self.layers.remove(index))
        } else {
            None
        }
    }

    /// Returns a reference to the layer at the given index.
    pub fn get(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }

    /// Returns a mutable reference to the layer at the given index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }

    /// Finds a layer by name.
    pub fn find_by_name(&self, name: &str) -> Option<(usize, &Layer)> {
        self.layers
            .iter()
            .enumerate()
            .find(|(_, l)| l.name == name)
    }

    /// Returns the number of layers in the stack.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Returns `true` if the stack contains no layers.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Returns an iterator over all layers.
    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.layers.iter()
    }

    /// Returns a mutable iterator over all layers.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Layer> {
        self.layers.iter_mut()
    }

    /// Pushes a default value on every layer (called when a new element is added).
    pub fn push_defaults(&mut self) {
        for layer in &mut self.layers {
            layer.push_default();
        }
    }

    /// Swap-removes an element at `index` from every layer.
    pub fn swap_remove(&mut self, index: usize) {
        for layer in &mut self.layers {
            layer.swap_remove(index);
        }
    }
}
