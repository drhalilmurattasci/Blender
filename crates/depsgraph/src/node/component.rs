//! Component nodes: represent sub-evaluations within an ID node.

/// Type of component within a data block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentType {
    /// Transform evaluation (location, rotation, scale).
    Transform,
    /// Geometry evaluation (mesh data, modifiers).
    Geometry,
    /// Animation evaluation (F-Curves, NLA).
    Animation,
    /// Constraint evaluation.
    Constraints,
    /// Bone transform (armature-specific).
    Bone,
    /// Particle system evaluation.
    Particles,
    /// Shading / material evaluation.
    Shading,
    /// Parameters (generic property updates).
    Parameters,
    /// Pose evaluation (full armature).
    Pose,
    /// Cache (physics / simulation cache).
    Cache,
    /// Proxy / override evaluation.
    Proxy,
    /// Synchronization barrier.
    Synchronization,
}

/// A component node in the dependency graph.
#[derive(Debug)]
pub struct ComponentNode {
    /// Component type.
    pub component_type: ComponentType,
    /// Name of the owning data block.
    pub owner_name: String,
    /// Whether this component needs evaluation.
    pub needs_update: bool,
}

impl ComponentNode {
    pub fn new(component_type: ComponentType, owner_name: impl Into<String>) -> Self {
        Self {
            component_type,
            owner_name: owner_name.into(),
            needs_update: true,
        }
    }
}
