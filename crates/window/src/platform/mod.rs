//! Platform backend abstraction: creates the OS window and drives the
//! event loop using `winit`.

use crate::WindowResult;

/// Opaque handle to the platform window.
pub struct PlatformWindow {
    /// Window title.
    pub title: String,
    /// Physical width.
    pub width: u32,
    /// Physical height.
    pub height: u32,
    /// Whether the window is in fullscreen mode.
    pub fullscreen: bool,
}

/// Platform backend trait abstracting over OS-level windowing.
pub trait PlatformBackend {
    /// Create and show a new OS window.
    fn create_window(&mut self, title: &str, width: u32, height: u32) -> WindowResult<PlatformWindow>;

    /// Set the window title.
    fn set_title(&mut self, title: &str);

    /// Set window dimensions.
    fn set_size(&mut self, width: u32, height: u32);

    /// Enter or exit fullscreen.
    fn set_fullscreen(&mut self, fullscreen: bool);

    /// Request the window to close.
    fn request_close(&mut self);

    /// Poll for pending OS events. Returns `true` if the application should
    /// continue running, `false` if a quit was requested.
    fn poll_events(&mut self) -> bool;
}

/// Stub platform backend for headless testing.
pub struct HeadlessBackend {
    pub should_close: bool,
}

impl HeadlessBackend {
    pub fn new() -> Self {
        Self { should_close: false }
    }
}

impl Default for HeadlessBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformBackend for HeadlessBackend {
    fn create_window(&mut self, title: &str, width: u32, height: u32) -> WindowResult<PlatformWindow> {
        Ok(PlatformWindow {
            title: title.to_string(),
            width,
            height,
            fullscreen: false,
        })
    }

    fn set_title(&mut self, _title: &str) {}
    fn set_size(&mut self, _width: u32, _height: u32) {}
    fn set_fullscreen(&mut self, _fullscreen: bool) {}

    fn request_close(&mut self) {
        self.should_close = true;
    }

    fn poll_events(&mut self) -> bool {
        !self.should_close
    }
}
