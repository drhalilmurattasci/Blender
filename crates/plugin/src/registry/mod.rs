//! Plugin registry: manages loaded plugins and their lifecycle.

use crate::context::PluginContext;
use crate::{PluginError, PluginResult};

/// Lifecycle trait that every plugin must implement (via the C ABI or
/// a Rust trait object for in-process plugins).
pub trait Plugin: Send + Sync {
    /// Unique plugin name.
    fn name(&self) -> &str;

    /// Semantic version string.
    fn version(&self) -> &str;

    /// Called once when the plugin is first loaded.
    fn on_register(&mut self, ctx: &mut PluginContext) -> PluginResult<()>;

    /// Called once when the plugin is about to be unloaded.
    fn on_unregister(&mut self, ctx: &mut PluginContext) -> PluginResult<()>;

    /// Called every frame (optional).
    fn on_frame(&mut self, _ctx: &mut PluginContext) -> PluginResult<()> {
        Ok(())
    }
}

/// Stored metadata about a loaded plugin.
pub struct PluginInfo {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Whether the plugin is currently enabled.
    pub enabled: bool,
}

/// Central plugin registry.
pub struct PluginRegistry {
    plugins: Vec<(PluginInfo, Box<dyn Plugin>)>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register and initialize a plugin.
    pub fn register(
        &mut self,
        mut plugin: Box<dyn Plugin>,
        ctx: &mut PluginContext,
    ) -> PluginResult<()> {
        let name = plugin.name().to_string();

        if self.plugins.iter().any(|(info, _)| info.name == name) {
            return Err(PluginError::ApiError(format!(
                "plugin `{name}` is already registered"
            )));
        }

        tracing::info!(plugin = %name, "registering plugin");
        plugin.on_register(ctx)?;

        let info = PluginInfo {
            name,
            version: plugin.version().to_string(),
            enabled: true,
        };

        self.plugins.push((info, plugin));
        Ok(())
    }

    /// Unregister a plugin by name.
    pub fn unregister(&mut self, name: &str, ctx: &mut PluginContext) -> PluginResult<()> {
        let idx = self
            .plugins
            .iter()
            .position(|(info, _)| info.name == name)
            .ok_or_else(|| PluginError::NotRegistered {
                name: name.to_string(),
            })?;

        let (_, mut plugin) = self.plugins.remove(idx);
        tracing::info!(plugin = %name, "unregistering plugin");
        plugin.on_unregister(ctx)?;
        Ok(())
    }

    /// Enable or disable a plugin.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> PluginResult<()> {
        let (info, _) = self
            .plugins
            .iter_mut()
            .find(|(info, _)| info.name == name)
            .ok_or_else(|| PluginError::NotRegistered {
                name: name.to_string(),
            })?;
        info.enabled = enabled;
        Ok(())
    }

    /// Tick all enabled plugins for a frame.
    pub fn tick_frame(&mut self, ctx: &mut PluginContext) -> PluginResult<()> {
        for (info, plugin) in &mut self.plugins {
            if info.enabled {
                plugin.on_frame(ctx)?;
            }
        }
        Ok(())
    }

    /// List all registered plugins.
    pub fn list(&self) -> impl Iterator<Item = &PluginInfo> {
        self.plugins.iter().map(|(info, _)| info)
    }

    /// Number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
