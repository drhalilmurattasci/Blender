//! Auto-bind utilities: helpers for automatically generating Python
//! bindings from Rust types annotated with `#[pyclass]`.
//!
//! This module provides registration helpers and type introspection
//! utilities that the build system can use to generate binding stubs.

use pyo3::prelude::*;

/// Trait for types that can self-register with a Python module.
pub trait AutoBind {
    /// The Python class name.
    fn python_name() -> &'static str;

    /// Register this type with the given module.
    fn register(m: &Bound<'_, PyModule>) -> PyResult<()>;
}

/// A registry of auto-bindable types, collected at startup.
pub struct BindingRegistry {
    registrations: Vec<Box<dyn Fn(&Bound<'_, PyModule>) -> PyResult<()> + Send + Sync>>,
}

impl BindingRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            registrations: Vec::new(),
        }
    }

    /// Add a registration function.
    pub fn add<F>(&mut self, f: F)
    where
        F: Fn(&Bound<'_, PyModule>) -> PyResult<()> + Send + Sync + 'static,
    {
        self.registrations.push(Box::new(f));
    }

    /// Register a type that implements [`AutoBind`].
    pub fn add_type<T: AutoBind + 'static>(&mut self) {
        self.registrations.push(Box::new(|m| T::register(m)));
    }

    /// Apply all registrations to a module.
    pub fn apply_all(&self, m: &Bound<'_, PyModule>) -> PyResult<()> {
        for reg in &self.registrations {
            reg(m)?;
        }
        Ok(())
    }

    /// Number of pending registrations.
    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }
}

impl Default for BindingRegistry {
    fn default() -> Self {
        Self::new()
    }
}
