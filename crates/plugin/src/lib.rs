//! # forge3d-plugin
//!
//! Plugin system for Forge3D: dynamic loading, registration, lifecycle
//! management, and context passing for native Rust plugins.

pub mod context;
pub mod loader;
pub mod registry;

pub use context::PluginContext;
pub use loader::PluginLoader;
pub use registry::{PluginInfo, PluginRegistry};

use thiserror::Error;

/// Errors from the plugin subsystem.
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin `{name}` failed to load: {reason}")]
    LoadFailed { name: String, reason: String },

    #[error("plugin `{name}` is not registered")]
    NotRegistered { name: String },

    #[error("plugin `{name}` version mismatch: expected {expected}, got {actual}")]
    VersionMismatch {
        name: String,
        expected: String,
        actual: String,
    },

    #[error("plugin `{name}` initialization failed: {reason}")]
    InitFailed { name: String, reason: String },

    #[error("plugin API error: {0}")]
    ApiError(String),
}

/// Result alias.
pub type PluginResult<T> = Result<T, PluginError>;

/// The ABI version plugins must match.
pub const PLUGIN_ABI_VERSION: u32 = 1;
