//! # forge3d-window
//!
//! Windowing, screen layout, areas, regions, event handling, and platform
//! abstraction for the Forge3D editor.
//!
//! Built on top of `winit` for cross-platform window management and input,
//! with a Blender-inspired area/region subdivision model.

pub mod area;
pub mod event;
pub mod manager;
pub mod platform;
pub mod region;
pub mod screen;

pub use area::{Area, AreaHandle, AreaType};
pub use event::{EventPhase, HandlerResult, InputEvent, KeyCode, ModifierKeys, MouseButton};
pub use manager::WindowManager;
pub use platform::PlatformBackend;
pub use region::{Region, RegionHandle, RegionType};
pub use screen::{Screen, ScreenHandle};

use thiserror::Error;

/// Errors originating from the windowing subsystem.
#[derive(Debug, Error)]
pub enum WindowError {
    #[error("failed to create window: {0}")]
    Creation(String),

    #[error("platform backend error: {0}")]
    Platform(String),

    #[error("area {0:?} not found")]
    AreaNotFound(AreaHandle),

    #[error("region {0:?} not found")]
    RegionNotFound(RegionHandle),

    #[error("event loop exited unexpectedly")]
    EventLoopExit,
}

/// Result alias for window operations.
pub type WindowResult<T> = Result<T, WindowError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_manager_default() {
        let wm = WindowManager::default();
        assert!(wm.active_screen.is_none());
        assert_eq!(wm.window_width, 1280);
        assert_eq!(wm.window_height, 720);
        assert!(!wm.modal_active);
    }

    #[test]
    fn push_and_drain_events() {
        let mut wm = WindowManager::new();
        wm.push_event(InputEvent::CloseRequested);
        wm.push_event(InputEvent::FocusChanged(true));
        let events = wm.drain_events();
        assert_eq!(events.len(), 2);
        // Queue should be empty after drain.
        assert!(wm.drain_events().is_empty());
    }

    #[test]
    fn add_screen_makes_active() {
        let mut wm = WindowManager::new();
        let s = Screen::new("Default", 1920, 1080);
        let h = wm.add_screen(s);
        assert_eq!(wm.active_screen, Some(h));
        assert!(wm.active_screen().is_some());
    }

    #[test]
    fn resize_updates_screen() {
        let mut wm = WindowManager::new();
        wm.add_screen(Screen::new("Main", 800, 600));
        wm.resize(1920, 1080);
        assert_eq!(wm.window_width, 1920);
        assert_eq!(wm.window_height, 1080);
        let screen = wm.active_screen().unwrap();
        assert_eq!(screen.rect, [0, 0, 1920, 1080]);
    }

    #[test]
    fn area_contains() {
        let area = Area::new(AreaType::Viewport3D, [10, 20, 100, 50]);
        assert!(area.contains(10, 20));
        assert!(area.contains(50, 40));
        assert!(!area.contains(9, 20));
        assert!(!area.contains(110, 20));
        assert!(!area.contains(10, 70));
    }

    #[test]
    fn region_contains() {
        let r = Region::new(RegionType::Main, [0, 0, 200, 100]);
        assert!(r.contains(0, 0));
        assert!(r.contains(199, 99));
        assert!(!r.contains(200, 0));
        assert!(!r.contains(0, 100));
    }

    #[test]
    fn screen_add_area() {
        let mut s = Screen::new("Test", 800, 600);
        use forge3d_alloc::Arena;
        let mut arena: Arena<Area> = Arena::new();
        let h = arena.insert(Area::new(AreaType::Outliner, [0, 0, 200, 600]));
        s.add_area(h);
        assert_eq!(s.areas.len(), 1);
    }

    #[test]
    fn headless_backend() {
        use platform::{HeadlessBackend, PlatformBackend};
        let mut backend = HeadlessBackend::new();
        assert!(backend.poll_events());
        backend.request_close();
        assert!(!backend.poll_events());
    }

    #[test]
    fn event_phase_order() {
        // Verify phases exist in correct dispatch order.
        let phases = [
            EventPhase::Window,
            EventPhase::Area,
            EventPhase::Region,
            EventPhase::Modal,
        ];
        assert_eq!(phases.len(), 4);
    }

    #[test]
    fn modifier_keys_bitflags() {
        let mods = ModifierKeys::SHIFT | ModifierKeys::CTRL;
        assert!(mods.contains(ModifierKeys::SHIFT));
        assert!(mods.contains(ModifierKeys::CTRL));
        assert!(!mods.contains(ModifierKeys::ALT));
    }
}
