//! Custom data layers for per-element attributes (UVs, vertex colors, etc.).

pub mod layer;
pub mod stack;
pub mod types;

pub use layer::{Layer, LayerData};
pub use stack::LayerStack;
pub use types::LayerType;
