//! # forge3d-ui-core
//!
//! Core UI framework for the Forge3D editor: widgets, layout engine,
//! operators, theming, input/keymap handling, and 2D painting.

pub mod input;
pub mod keymap;
pub mod layout;
pub mod operator;
pub mod painter;
pub mod theme;
pub mod widget;

pub use input::{DragState, InputState};
pub use keymap::{KeyBinding, KeyMap, KeyMapRegistry};
pub use layout::{Layout, LayoutDirection, LayoutItem};
pub use operator::{Operator, OperatorContext, OperatorResult, OperatorRegistry};
pub use painter::Painter;
pub use theme::{Theme, ThemeColor};
pub use widget::{Widget, WidgetId, WidgetResponse};

use thiserror::Error;

/// Errors originating from the UI subsystem.
#[derive(Debug, Error)]
pub enum UiError {
    #[error("widget `{0}` not found")]
    WidgetNotFound(String),

    #[error("operator `{idname}` failed: {reason}")]
    OperatorFailed { idname: String, reason: String },

    #[error("layout overflow: {0}")]
    LayoutOverflow(String),

    #[error("keymap conflict: `{0}` is already bound")]
    KeymapConflict(String),
}

/// Result alias for UI operations.
pub type UiResult<T> = Result<T, UiError>;
