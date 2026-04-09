//! Forge3D application core.
//!
//! Provides the top-level [`Application`] struct that bootstraps a default scene,
//! initialises subsystems, and prints a ready message.

use tracing::info;

// ---------------------------------------------------------------------------
// Re-exports for convenience
// ---------------------------------------------------------------------------

pub use forge3d_scene as scene;
pub use forge3d_window as window;
pub use forge3d_gpu as gpu;
pub use forge3d_render_core as render_core;
pub use forge3d_depsgraph as depsgraph;

// ---------------------------------------------------------------------------
// Application
// ---------------------------------------------------------------------------

/// Top-level application state that owns every major subsystem.
pub struct Application {
    /// The active scene (collections, objects, world).
    pub scene: forge3d_scene::Scene,
    /// Window manager for event handling and screen layout.
    pub window_manager: forge3d_window::WindowManager,
    /// Plugin registry for user-installed add-ons.
    pub plugins: forge3d_plugin::PluginRegistry,
}

impl Application {
    /// Create a new application with an empty scene and default subsystems.
    ///
    /// GPU and realtime rendering are **not** initialised here because they
    /// require an async adapter request and a live window surface.  For now
    /// we keep the bootstrap synchronous and simple.
    pub fn new() -> Self {
        info!("Initialising Forge3D");

        let scene = forge3d_scene::Scene::new("Scene");
        let window_manager = forge3d_window::WindowManager::new();
        let plugins = forge3d_plugin::PluginRegistry::new();

        Self {
            scene,
            window_manager,
            plugins,
        }
    }

    /// Populate the scene with a default cube, camera, and point light.
    ///
    /// This mirrors the classic startup file every 3-D artist knows and loves.
    pub fn create_default_scene(&mut self) {
        info!("Creating default scene");

        // -- Default cube (mesh object with a placeholder data-block ref) ----
        let cube = forge3d_scene::Object {
            name: "Cube".into(),
            object_type: forge3d_scene::ObjectType::Mesh,
            transform: forge3d_scene::Transform {
                location: [0.0, 0.0, 0.0],
                ..forge3d_scene::Transform::IDENTITY
            },
            data: forge3d_scene::ObjectData::Mesh(forge3d_scene::object::DataBlockRef {
                index: 0,
                generation: 0,
            }),
            parent: None,
            children: Vec::new(),
            visibility: forge3d_scene::VisibilityFlags::default(),
            select_state: forge3d_scene::SelectState::None,
        };
        self.scene.add_object(cube);

        // -- Camera ----------------------------------------------------------
        let mut camera = forge3d_scene::Object::camera("Camera");
        camera.transform.location = [7.359, -6.926, 4.958];
        camera.transform.recompute_matrix();
        let cam_handle = self.scene.add_object(camera);
        self.scene.active_camera = Some(cam_handle);

        // -- Point light -----------------------------------------------------
        let mut light = forge3d_scene::Object::light(
            "Light",
            forge3d_scene::object::LightKind::Point,
        );
        light.transform.location = [4.076, 1.005, 5.904];
        light.transform.recompute_matrix();
        self.scene.add_object(light);

        let obj_count = self.scene.objects.len();
        info!(objects = obj_count, "Default scene created");
    }

    /// Print a ready message. A full event loop requires winit surface
    /// integration which is left for a future milestone.
    pub fn run(self) {
        info!("Forge3D initialised with {} object(s)", self.scene.objects.len());
        println!("Forge3D initialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_tracing() {
        // Ignore errors if already initialized.
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn application_new_does_not_panic() {
        init_tracing();
        let app = Application::new();
        assert_eq!(app.scene.name, "Scene");
        assert_eq!(app.scene.fps, 24.0);
        assert!(app.scene.active_camera.is_none());
        assert!(app.plugins.is_empty());
    }

    #[test]
    fn create_default_scene_adds_three_objects() {
        init_tracing();
        let mut app = Application::new();
        app.create_default_scene();
        // Should have Cube, Camera, Light.
        let names: Vec<&str> = app.scene.iter_objects().map(|(_, o)| o.name.as_str()).collect();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"Cube"));
        assert!(names.contains(&"Camera"));
        assert!(names.contains(&"Light"));
    }

    #[test]
    fn default_scene_has_active_camera() {
        init_tracing();
        let mut app = Application::new();
        app.create_default_scene();
        assert!(app.scene.active_camera.is_some());
        let cam_handle = app.scene.active_camera.unwrap();
        let cam = app.scene.get_object(cam_handle).unwrap();
        assert_eq!(cam.name, "Camera");
    }

    #[test]
    fn default_scene_camera_transform_recomputed() {
        init_tracing();
        let mut app = Application::new();
        app.create_default_scene();
        let cam_handle = app.scene.active_camera.unwrap();
        let cam = app.scene.get_object(cam_handle).unwrap();
        // Translation column should match location.
        assert!((cam.transform.matrix_local[3][0] - 7.359).abs() < 0.01);
    }
}
