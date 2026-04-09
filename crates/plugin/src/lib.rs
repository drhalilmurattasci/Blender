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

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: &'static str,
        registered: bool,
    }

    impl TestPlugin {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                registered: false,
            }
        }
    }

    impl registry::Plugin for TestPlugin {
        fn name(&self) -> &str {
            self.name
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn on_register(&mut self, _ctx: &mut PluginContext) -> PluginResult<()> {
            self.registered = true;
            Ok(())
        }
        fn on_unregister(&mut self, _ctx: &mut PluginContext) -> PluginResult<()> {
            Ok(())
        }
    }

    #[test]
    fn plugin_registry_register_and_list() {
        let mut reg = registry::PluginRegistry::new();
        let mut ctx = PluginContext::new();
        assert!(reg.is_empty());

        reg.register(Box::new(TestPlugin::new("test_plugin")), &mut ctx)
            .unwrap();
        assert_eq!(reg.len(), 1);
        let names: Vec<&str> = reg.list().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["test_plugin"]);
    }

    #[test]
    fn plugin_registry_duplicate_rejected() {
        let mut reg = registry::PluginRegistry::new();
        let mut ctx = PluginContext::new();
        reg.register(Box::new(TestPlugin::new("dup")), &mut ctx).unwrap();
        let result = reg.register(Box::new(TestPlugin::new("dup")), &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn plugin_registry_unregister() {
        let mut reg = registry::PluginRegistry::new();
        let mut ctx = PluginContext::new();
        reg.register(Box::new(TestPlugin::new("removeme")), &mut ctx).unwrap();
        assert_eq!(reg.len(), 1);
        reg.unregister("removeme", &mut ctx).unwrap();
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn plugin_registry_unregister_not_found() {
        let mut reg = registry::PluginRegistry::new();
        let mut ctx = PluginContext::new();
        let result = reg.unregister("nonexistent", &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn plugin_registry_set_enabled() {
        let mut reg = registry::PluginRegistry::new();
        let mut ctx = PluginContext::new();
        reg.register(Box::new(TestPlugin::new("toggle")), &mut ctx).unwrap();
        reg.set_enabled("toggle", false).unwrap();
        let info = reg.list().next().unwrap();
        assert!(!info.enabled);
    }

    #[test]
    fn plugin_context_subsystems() {
        let mut ctx = PluginContext::new();
        ctx.register_subsystem("counter", 42u32);
        assert_eq!(ctx.get_subsystem::<u32>("counter"), Some(&42));
        assert!(ctx.get_subsystem::<String>("counter").is_none());
        assert!(ctx.get_subsystem::<u32>("missing").is_none());
    }

    #[test]
    fn plugin_manifest_name_str() {
        let mut m = loader::PluginManifest {
            abi_version: PLUGIN_ABI_VERSION,
            name: [0; 64],
            version: [0; 32],
            author: [0; 64],
            description: [0; 256],
        };
        m.name[..5].copy_from_slice(b"hello");
        assert_eq!(m.name_str(), "hello");
    }

    #[test]
    fn plugin_manifest_validate_ok() {
        let m = loader::PluginManifest {
            abi_version: PLUGIN_ABI_VERSION,
            name: [0; 64],
            version: [0; 32],
            author: [0; 64],
            description: [0; 256],
        };
        assert!(loader::PluginLoader::validate_manifest(&m).is_ok());
    }

    #[test]
    fn plugin_manifest_validate_mismatch() {
        let m = loader::PluginManifest {
            abi_version: PLUGIN_ABI_VERSION + 99,
            name: [0; 64],
            version: [0; 32],
            author: [0; 64],
            description: [0; 256],
        };
        assert!(loader::PluginLoader::validate_manifest(&m).is_err());
    }

    #[test]
    fn tick_frame_skips_disabled() {
        let mut reg = registry::PluginRegistry::new();
        let mut ctx = PluginContext::new();
        reg.register(Box::new(TestPlugin::new("p")), &mut ctx).unwrap();
        reg.set_enabled("p", false).unwrap();
        // Should not error even though plugin is disabled.
        reg.tick_frame(&mut ctx).unwrap();
    }
}
