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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_default_values() {
        let s = Scene::default();
        assert_eq!(s.name, "Scene");
        assert_eq!(s.fps, 24.0);
        assert_eq!(s.frame_start, 1);
        assert_eq!(s.frame_end, 250);
        assert_eq!(s.frame_current, 1.0);
        assert!(s.active_camera.is_none());
    }

    #[test]
    fn world_default_values() {
        let w = World::default();
        assert!(!w.mist_enabled);
        assert_eq!(w.mist_start, 5.0);
        assert_eq!(w.mist_depth, 25.0);
        assert_eq!(w.background_type, world::BackgroundType::Color);
        assert_eq!(w.ambient.energy, 1.0);
    }

    #[test]
    fn add_and_remove_object() {
        let mut s = Scene::default();
        let obj = Object::empty("TestObj");
        let h = s.add_object(obj);
        assert!(s.get_object(h).is_some());
        assert_eq!(s.get_object(h).unwrap().name, "TestObj");

        let removed = s.remove_object(h);
        assert!(removed.is_some());
        assert!(s.get_object(h).is_none());
    }

    #[test]
    fn remove_object_cleans_collections() {
        let mut s = Scene::default();
        let h = s.add_object(Object::empty("A"));
        // Object should be in root collection.
        let root = s.collections.get(s.root_collection).unwrap();
        assert!(root.objects.contains(&h));

        s.remove_object(h);
        let root = s.collections.get(s.root_collection).unwrap();
        assert!(!root.objects.contains(&h));
    }

    #[test]
    fn iter_objects() {
        let mut s = Scene::default();
        s.add_object(Object::empty("A"));
        s.add_object(Object::empty("B"));
        let names: Vec<&str> = s.iter_objects().map(|(_, o)| o.name.as_str()).collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"A"));
        assert!(names.contains(&"B"));
    }

    #[test]
    fn transform_identity() {
        let t = Transform::IDENTITY;
        assert_eq!(t.location, [0.0, 0.0, 0.0]);
        assert_eq!(t.scale, [1.0, 1.0, 1.0]);
        // Identity matrix diagonal should be 1.
        for i in 0..4 {
            assert_eq!(t.matrix_local[i][i], 1.0);
        }
    }

    #[test]
    fn transform_recompute_translation_only() {
        let mut t = Transform::IDENTITY;
        t.location = [3.0, 4.0, 5.0];
        t.recompute_matrix();
        // Column 3 should hold the translation.
        assert_eq!(t.matrix_local[3][0], 3.0);
        assert_eq!(t.matrix_local[3][1], 4.0);
        assert_eq!(t.matrix_local[3][2], 5.0);
        assert_eq!(t.matrix_local[3][3], 1.0);
    }

    #[test]
    fn transform_recompute_scale() {
        let mut t = Transform::IDENTITY;
        t.scale = [2.0, 3.0, 4.0];
        t.recompute_matrix();
        // With identity rotation, diagonal should be the scale values.
        assert!((t.matrix_local[0][0] - 2.0).abs() < 1e-6);
        assert!((t.matrix_local[1][1] - 3.0).abs() < 1e-6);
        assert!((t.matrix_local[2][2] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn visibility_default_is_all() {
        let v = VisibilityFlags::default();
        assert!(v.contains(VisibilityFlags::VIEWPORT));
        assert!(v.contains(VisibilityFlags::RENDER));
        assert!(v.contains(VisibilityFlags::SHADOW));
    }

    #[test]
    fn collection_add_child_dedup() {
        let mut c = Collection::new("Test");
        use forge3d_alloc::Arena;
        let mut arena: Arena<Collection> = Arena::new();
        let h = arena.insert(Collection::new("Child"));
        c.add_child(h);
        c.add_child(h); // duplicate
        assert_eq!(c.children.len(), 1);
    }

    #[test]
    fn camera_defaults() {
        let cam = object::CameraData::default();
        assert_eq!(cam.focal_length, 50.0);
        assert_eq!(cam.sensor_width, 36.0);
        assert!(!cam.is_orthographic);
    }

    #[test]
    fn light_defaults() {
        let light = object::LightData::default();
        assert_eq!(light.kind, object::LightKind::Point);
        assert!(light.cast_shadow);
        assert_eq!(light.color, [1.0, 1.0, 1.0]);
    }
}
