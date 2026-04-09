//! # forge3d-scene
//!
//! Scene graph, objects, collections, transforms, visibility, and world
//! settings for the Forge3D engine.
//!
//! The scene is the top-level container that owns all objects, collections,
//! and world environment data. Objects carry transform, visibility, and
//! data-block references (mesh, armature, camera, light, etc.).

pub mod collection;
pub mod object;
pub mod transform;
pub mod visibility;
pub mod world;

pub use collection::{Collection, CollectionHandle};
pub use object::{Object, ObjectData, ObjectHandle, ObjectType};
pub use transform::Transform;
pub use visibility::{SelectState, VisibilityFlags};
pub use world::{AmbientLight, World};

use forge3d_alloc::Arena;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Scene
// ---------------------------------------------------------------------------

/// Unique identifier for a scene within a file.
pub type SceneId = u64;

/// The top-level scene container.
#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    /// Human-readable name.
    pub name: String,
    /// Unique identifier.
    pub id: SceneId,
    /// All objects owned by this scene.
    #[serde(skip)]
    pub objects: Arena<Object>,
    /// All collections owned by this scene.
    #[serde(skip)]
    pub collections: Arena<Collection>,
    /// Root collection that implicitly contains every top-level object.
    pub root_collection: CollectionHandle,
    /// World / environment settings.
    pub world: World,
    /// The currently active camera object (if any).
    pub active_camera: Option<ObjectHandle>,
    /// Frame range for playback.
    pub frame_start: i32,
    /// End frame (inclusive).
    pub frame_end: i32,
    /// Current frame.
    pub frame_current: f32,
    /// Frames per second.
    pub fps: f32,
}

impl Scene {
    /// Create a new, empty scene.
    pub fn new(name: impl Into<String>) -> Self {
        let mut collections = Arena::new();
        let root = Collection {
            name: "Scene Collection".into(),
            children: Vec::new(),
            objects: Vec::new(),
            parent: None,
            is_visible: true,
            is_selectable: true,
            color_tag: None,
        };
        let root_handle = collections.insert(root);

        Self {
            name: name.into(),
            id: 0,
            objects: Arena::new(),
            collections,
            root_collection: root_handle,
            world: World::default(),
            active_camera: None,
            frame_start: 1,
            frame_end: 250,
            frame_current: 1.0,
            fps: 24.0,
        }
    }

    /// Insert an object into the scene and add it to the root collection.
    pub fn add_object(&mut self, obj: Object) -> ObjectHandle {
        let handle = self.objects.insert(obj);
        if let Some(root) = self.collections.get_mut(self.root_collection) {
            root.objects.push(handle);
        }
        handle
    }

    /// Remove an object by handle. Returns the object if it existed.
    pub fn remove_object(&mut self, handle: ObjectHandle) -> Option<Object> {
        // Remove from all collections.
        for (_h, col) in self.collections.iter_mut() {
            col.objects.retain(|&o| o != handle);
        }
        self.objects.remove(handle)
    }

    /// Get an immutable reference to an object.
    pub fn get_object(&self, handle: ObjectHandle) -> Option<&Object> {
        self.objects.get(handle)
    }

    /// Get a mutable reference to an object.
    pub fn get_object_mut(&mut self, handle: ObjectHandle) -> Option<&mut Object> {
        self.objects.get_mut(handle)
    }

    /// Iterate all objects in the scene.
    pub fn iter_objects(&self) -> impl Iterator<Item = (ObjectHandle, &Object)> {
        self.objects.iter()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Scene")
    }
}
