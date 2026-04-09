//! Plugin context: the API surface exposed to plugins at runtime.

use std::any::Any;

/// The context object passed to plugins during their lifecycle callbacks.
///
/// Provides controlled access to engine subsystems (scene, UI, mesh, etc.)
/// without exposing internal implementation details.
pub struct PluginContext {
    /// Opaque references to engine subsystems, keyed by name.
    subsystems: Vec<(&'static str, Box<dyn Any + Send>)>,
}

impl PluginContext {
    /// Create a new, empty plugin context.
    pub fn new() -> Self {
        Self {
            subsystems: Vec::new(),
        }
    }

    /// Register a subsystem reference that plugins can access.
    pub fn register_subsystem<T: Any + Send + 'static>(
        &mut self,
        name: &'static str,
        subsystem: T,
    ) {
        self.subsystems.push((name, Box::new(subsystem)));
    }

    /// Get a typed reference to a subsystem.
    pub fn get_subsystem<T: Any + Send + 'static>(&self, name: &str) -> Option<&T> {
        self.subsystems
            .iter()
            .find(|(n, _)| *n == name)
            .and_then(|(_, v)| v.downcast_ref::<T>())
    }

    /// Get a mutable typed reference to a subsystem.
    pub fn get_subsystem_mut<T: Any + Send + 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.subsystems
            .iter_mut()
            .find(|(n, _)| *n == name)
            .and_then(|(_, v)| v.downcast_mut::<T>())
    }

    /// List all registered subsystem names.
    pub fn subsystem_names(&self) -> impl Iterator<Item = &str> {
        self.subsystems.iter().map(|(n, _)| *n)
    }
}

impl Default for PluginContext {
    fn default() -> Self {
        Self::new()
    }
}
