//! Input event types: keyboard, mouse, and modifier state.
//!
//! Blender's event dispatch order is: Window -> Area -> Region -> Operator.
//! Each handler in the chain returns a [`HandlerResult`] controlling whether
//! the event propagates further.

use bitflags::bitflags;

/// Result returned by an event handler, mirroring Blender's
/// `WM_HANDLER_BREAK` / `WM_HANDLER_CONTINUE` pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandlerResult {
    /// Event was consumed — stop propagation.
    Break,
    /// Event was not consumed — pass to the next handler.
    Continue,
}

/// The phase an event is currently in during dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventPhase {
    /// Window-level handlers (file select, window management).
    Window,
    /// Area-level handlers (editor-specific keymaps).
    Area,
    /// Region-level handlers (header, sidebar, main region).
    Region,
    /// Operator modal handlers.
    Modal,
}

/// Mouse button identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

/// Keyboard key codes (subset of the most commonly used keys).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    Escape, Tab, Space, Enter, Backspace, Delete,
    Left, Right, Up, Down,
    Home, End, PageUp, PageDown,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    NumPad0, NumPad1, NumPad2, NumPad3, NumPad4,
    NumPad5, NumPad6, NumPad7, NumPad8, NumPad9,
    NumPadPlus, NumPadMinus, NumPadMultiply, NumPadDivide, NumPadEnter, NumPadDot,
}

bitflags! {
    /// Active modifier keys.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ModifierKeys: u8 {
        const SHIFT = 0b0001;
        const CTRL  = 0b0010;
        const ALT   = 0b0100;
        const SUPER = 0b1000;
    }
}

/// Unified input event.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// A key was pressed.
    KeyPress {
        key: KeyCode,
        modifiers: ModifierKeys,
    },
    /// A key was released.
    KeyRelease {
        key: KeyCode,
        modifiers: ModifierKeys,
    },
    /// Mouse button pressed at (x, y) in window pixels.
    MousePress {
        button: MouseButton,
        x: f64,
        y: f64,
        modifiers: ModifierKeys,
    },
    /// Mouse button released.
    MouseRelease {
        button: MouseButton,
        x: f64,
        y: f64,
        modifiers: ModifierKeys,
    },
    /// Mouse moved to (x, y).
    MouseMove {
        x: f64,
        y: f64,
        modifiers: ModifierKeys,
    },
    /// Scroll wheel delta (horizontal, vertical).
    Scroll {
        dx: f64,
        dy: f64,
        modifiers: ModifierKeys,
    },
    /// Window resize event.
    Resize {
        width: u32,
        height: u32,
    },
    /// Window close requested.
    CloseRequested,
    /// Window gained or lost focus.
    FocusChanged(bool),
}
