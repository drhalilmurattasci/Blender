//! Common modifier infrastructure: the modifier trait, stack, and shared types.

use serde::{Deserialize, Serialize};
use crate::ModifierResult;

/// Modifier type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierType {
    // Generate
    Array,
    Mirror,
    Boolean,
    Screw,
    Solidify,
    Subdivision,
    Bevel,
    Remesh,
    Skin,
    Wireframe,
    // Deform
    Armature,
    Lattice,
    Shrinkwrap,
    SimpleDeform,
    SmoothCorrectiveSmooth,
    LaplacianSmooth,
    SurfaceDeform,
    MeshDeform,
    Cast,
    Curve,
    Warp,
    Wave,
    // Modify
    WeightedNormal,
    DataTransfer,
    NormalEdit,
    UVProject,
    UVWarp,
    VertexWeightEdit,
    VertexWeightMix,
    VertexWeightProximity,
    Triangulate,
    Decimate,
    EdgeSplit,
    // Physics
    Cloth,
    Collision,
    SoftBody,
    Fluid,
    // Geometry Nodes (external)
    GeometryNodes,
}

bitflags::bitflags! {
    /// Flags controlling how a modifier is displayed / applied.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ModifierFlags: u16 {
        /// Show in viewport.
        const VIEWPORT       = 0b0000_0001;
        /// Apply during render.
        const RENDER         = 0b0000_0010;
        /// Show in edit mode.
        const EDIT_MODE      = 0b0000_0100;
        /// Apply on spline (curves).
        const ON_SPLINE      = 0b0000_1000;
        /// Modifier is expanded in the UI.
        const EXPANDED       = 0b0001_0000;
        /// Modifier is currently active / selected.
        const ACTIVE         = 0b0010_0000;
    }
}

impl Default for ModifierFlags {
    fn default() -> Self {
        Self::VIEWPORT | Self::RENDER | Self::EXPANDED
    }
}

/// Trait that all modifier implementations must satisfy.
pub trait Modifier: Send + Sync {
    /// Modifier type tag.
    fn modifier_type(&self) -> ModifierType;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Flags.
    fn flags(&self) -> ModifierFlags;

    /// Set flags.
    fn set_flags(&mut self, flags: ModifierFlags);

    /// Apply the modifier to mesh data, returning a (possibly new) mesh.
    ///
    /// `mesh_data` is an opaque `&mut dyn std::any::Any` so the modifier
    /// crate does not need to be generic over the full mesh type.
    fn apply(&self, mesh_data: &mut dyn std::any::Any) -> ModifierResult<()>;
}

/// An ordered stack of modifiers applied top-to-bottom.
#[derive(Default)]
pub struct ModifierStack {
    modifiers: Vec<Box<dyn Modifier>>,
}

impl ModifierStack {
    /// Create an empty stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a modifier to the bottom of the stack.
    pub fn push(&mut self, modifier: Box<dyn Modifier>) {
        self.modifiers.push(modifier);
    }

    /// Insert at a specific index.
    pub fn insert(&mut self, index: usize, modifier: Box<dyn Modifier>) {
        self.modifiers.insert(index.min(self.modifiers.len()), modifier);
    }

    /// Remove a modifier by index.
    pub fn remove(&mut self, index: usize) -> Option<Box<dyn Modifier>> {
        if index < self.modifiers.len() {
            Some(self.modifiers.remove(index))
        } else {
            None
        }
    }

    /// Move a modifier from `from` to `to`.
    pub fn reorder(&mut self, from: usize, to: usize) {
        if from < self.modifiers.len() && to < self.modifiers.len() {
            let m = self.modifiers.remove(from);
            self.modifiers.insert(to, m);
        }
    }

    /// Number of modifiers.
    pub fn len(&self) -> usize {
        self.modifiers.len()
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }

    /// Get a modifier by index.
    pub fn get(&self, index: usize) -> Option<&dyn Modifier> {
        self.modifiers.get(index).map(|m| m.as_ref())
    }

    /// Get a mutable modifier by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut (dyn Modifier + '_)> {
        self.modifiers.get_mut(index).map(|m| m.as_mut() as &mut (dyn Modifier + '_))
    }

    /// Apply all enabled modifiers in order.
    pub fn apply_all(&self, mesh_data: &mut dyn std::any::Any) -> ModifierResult<()> {
        for m in &self.modifiers {
            if m.flags().contains(ModifierFlags::VIEWPORT) {
                m.apply(mesh_data)?;
            }
        }
        Ok(())
    }

    /// Iterate immutably.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Modifier> {
        self.modifiers.iter().map(|m| m.as_ref())
    }
}
