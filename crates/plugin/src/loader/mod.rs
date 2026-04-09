//! Dynamic plugin loader: discovers and loads plugin shared libraries.

use crate::{PluginError, PluginResult, PLUGIN_ABI_VERSION};
use std::path::PathBuf;

/// Metadata returned by a plugin's `forge3d_plugin_info` symbol.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PluginManifest {
    /// Plugin ABI version (must match [`PLUGIN_ABI_VERSION`]).
    pub abi_version: u32,
    /// Plugin name (null-terminated C string pointer).
    pub name: [u8; 64],
    /// Plugin version string (null-terminated).
    pub version: [u8; 32],
    /// Plugin author (null-terminated).
    pub author: [u8; 64],
    /// Plugin description (null-terminated).
    pub description: [u8; 256],
}

impl PluginManifest {
    /// Read the name as a Rust string.
    pub fn name_str(&self) -> &str {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(self.name.len());
        std::str::from_utf8(&self.name[..end]).unwrap_or("<invalid utf8>")
    }

    /// Read the version as a Rust string.
    pub fn version_str(&self) -> &str {
        let end = self.version.iter().position(|&b| b == 0).unwrap_or(self.version.len());
        std::str::from_utf8(&self.version[..end]).unwrap_or("<invalid>")
    }
}

/// Discovers and loads plugin shared libraries from a directory.
pub struct PluginLoader {
    /// Directories to search for plugins.
    search_paths: Vec<PathBuf>,
}

impl PluginLoader {
    /// Create a loader with the given search paths.
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self { search_paths }
    }

    /// Add a search path.
    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) {
        self.search_paths.push(path.into());
    }

    /// Discover plugin files in all search paths.
    pub fn discover(&self) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let ext = plugin_extension();
        for dir in &self.search_paths {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some(ext) {
                        found.push(path);
                    }
                }
            }
        }
        found
    }

    /// Validate that a manifest is compatible.
    pub fn validate_manifest(manifest: &PluginManifest) -> PluginResult<()> {
        if manifest.abi_version != PLUGIN_ABI_VERSION {
            return Err(PluginError::VersionMismatch {
                name: manifest.name_str().to_string(),
                expected: format!("{PLUGIN_ABI_VERSION}"),
                actual: format!("{}", manifest.abi_version),
            });
        }
        Ok(())
    }
}

/// Platform-specific shared library extension.
fn plugin_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}
