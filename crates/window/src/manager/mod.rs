//! Window manager: owns the OS window, event loop bridge, and the active
//! screen layout.
//!
//! Blender's event dispatch order:
//!   1. Window-level modal handlers (file select dialogs)
//!   2. Per-screen → per-area → per-region handlers
//!   3. Operator modal handlers consume events first within their region
//!   4. Keymap matching for new operator invocations

use crate::event::InputEvent;
use crate::screen::{Screen, ScreenHandle};
use forge3d_alloc::Arena;

/// Central manager for all windows and screens.
pub struct WindowManager {
    /// Screens (one per OS window in multi-window setups).
    pub screens: Arena<Screen>,
    /// The currently active screen.
    pub active_screen: Option<ScreenHandle>,
    /// Queued input events waiting to be dispatched.
    event_queue: Vec<InputEvent>,
    /// Physical window size in pixels.
    pub window_width: u32,
    pub window_height: u32,
    /// Whether a modal operator is currently running. When true, events
    /// are routed to the modal handler first (Blender's `win->modalhandlers`).
    pub modal_active: bool,
}

impl WindowManager {
    /// Create a new window manager (no OS window opened yet).
    pub fn new() -> Self {
        Self {
            screens: Arena::new(),
            active_screen: None,
            event_queue: Vec::with_capacity(64),
            window_width: 1280,
            window_height: 720,
            modal_active: false,
        }
    }

    /// Add a screen and make it active.
    pub fn add_screen(&mut self, screen: Screen) -> ScreenHandle {
        let handle = self.screens.insert(screen);
        self.active_screen = Some(handle);
        handle
    }

    /// Push a raw input event into the queue.
    pub fn push_event(&mut self, event: InputEvent) {
        self.event_queue.push(event);
    }

    /// Drain all queued events.
    pub fn drain_events(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.event_queue)
    }

    /// Get the active screen, if any.
    pub fn active_screen(&self) -> Option<&Screen> {
        self.active_screen
            .and_then(|h| self.screens.get(h))
    }

    /// Get a mutable reference to the active screen.
    pub fn active_screen_mut(&mut self) -> Option<&mut Screen> {
        self.active_screen
            .and_then(|h| self.screens.get_mut(h))
    }

    /// Resize the window and notify the active screen.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.window_width = width;
        self.window_height = height;
        if let Some(screen) = self.active_screen_mut() {
            screen.rect = [0, 0, width, height];
        }
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}
