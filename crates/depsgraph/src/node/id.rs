//! ID nodes: represent top-level data blocks in the dependency graph.

/// Type of ID data block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdType {
    /// Scene.
    Scene,
    /// Object (mesh, armature, empty, camera, light, etc.).
    Object,
    /// Mesh data.
    Mesh,
    /// Armature data.
    Armature,
    /// Material.
    Material,
    /// Texture.
    Texture,
    /// Camera.
    Camera,
    /// Light / Lamp.
    Light,
    /// World settings.
    World,
    /// Particle system settings.
    Particle,
    /// Action (animation data).
    Action,
    /// Node tree (shader/compositor/geometry).
    NodeTree,
    /// Collection.
    Collection,
}

/// An ID node in the dependency graph.
///
/// Represents a single data-block (e.g., one Object, one Mesh).
#[derive(Debug)]
pub struct IdNode {
    /// Name of the data block.
    pub name: String,
    /// Type of the data block.
    pub id_type: IdType,
    /// Whether this ID node needs a copy-on-write.
    pub needs_cow: bool,
    /// Whether this data block has been flagged for update.
    pub needs_update: bool,
}

impl IdNode {
    pub fn new(name: impl Into<String>, id_type: IdType) -> Self {
        Self {
            name: name.into(),
            id_type,
            needs_cow: false,
            needs_update: true,
        }
    }
}
