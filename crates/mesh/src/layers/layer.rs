//! A single custom data layer.

use serde::{Deserialize, Serialize};

use super::types::LayerType;

/// A named custom data layer that stores per-element data.
///
/// Each variant holds a `Vec` of the appropriate type, one entry per mesh
/// element (vert, edge, loop, or face — depending on where the layer is used).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Human-readable name (e.g. "UVMap", "Col", "crease").
    pub name: String,
    /// The type descriptor for this layer.
    pub layer_type: LayerType,
    /// The typed data storage.
    pub data: LayerData,
}

/// Typed storage for a custom data layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerData {
    /// Per-element `f32` values.
    Float(Vec<f32>),
    /// Per-element `[f32; 2]` values (e.g. UV coordinates).
    Vec2(Vec<[f32; 2]>),
    /// Per-element `[f32; 3]` values.
    Vec3(Vec<[f32; 3]>),
    /// Per-element `[f32; 4]` RGBA color values.
    Color4f(Vec<[f32; 4]>),
    /// Per-element `i32` values.
    Int(Vec<i32>),
    /// Per-element boolean values.
    Bool(Vec<bool>),
}

impl Layer {
    /// Creates a new empty layer with the given name and type.
    pub fn new(name: impl Into<String>, layer_type: LayerType) -> Self {
        let data = match layer_type {
            LayerType::Float => LayerData::Float(Vec::new()),
            LayerType::Vec2 => LayerData::Vec2(Vec::new()),
            LayerType::Vec3 => LayerData::Vec3(Vec::new()),
            LayerType::Color4f => LayerData::Color4f(Vec::new()),
            LayerType::Int => LayerData::Int(Vec::new()),
            LayerType::Bool => LayerData::Bool(Vec::new()),
        };
        Self {
            name: name.into(),
            layer_type,
            data,
        }
    }

    /// Returns the number of elements stored in this layer.
    pub fn len(&self) -> usize {
        match &self.data {
            LayerData::Float(v) => v.len(),
            LayerData::Vec2(v) => v.len(),
            LayerData::Vec3(v) => v.len(),
            LayerData::Color4f(v) => v.len(),
            LayerData::Int(v) => v.len(),
            LayerData::Bool(v) => v.len(),
        }
    }

    /// Returns `true` if this layer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pushes a default value for the layer type.
    pub fn push_default(&mut self) {
        match &mut self.data {
            LayerData::Float(v) => v.push(0.0),
            LayerData::Vec2(v) => v.push([0.0; 2]),
            LayerData::Vec3(v) => v.push([0.0; 3]),
            LayerData::Color4f(v) => v.push([1.0, 1.0, 1.0, 1.0]),
            LayerData::Int(v) => v.push(0),
            LayerData::Bool(v) => v.push(false),
        }
    }

    /// Removes the element at the given index (swap-remove for performance).
    pub fn swap_remove(&mut self, index: usize) {
        match &mut self.data {
            LayerData::Float(v) => { v.swap_remove(index); }
            LayerData::Vec2(v) => { v.swap_remove(index); }
            LayerData::Vec3(v) => { v.swap_remove(index); }
            LayerData::Color4f(v) => { v.swap_remove(index); }
            LayerData::Int(v) => { v.swap_remove(index); }
            LayerData::Bool(v) => { v.swap_remove(index); }
        }
    }
}
