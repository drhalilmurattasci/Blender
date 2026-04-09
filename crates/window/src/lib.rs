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
