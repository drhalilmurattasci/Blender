//! File format detection and registration.

use crate::IoResult;
use std::path::Path;

/// Supported file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileFormat {
    /// Native Forge3D binary (rkyv-based).
    Forge3D,
    /// Blender .blend file (import only).
    Blend,
    /// glTF 2.0 (.gltf / .glb).
    GlTF,
    /// Wavefront OBJ.
    Obj,
    /// FBX.
    Fbx,
    /// STL (stereolithography).
    Stl,
    /// PLY (polygon file format).
    Ply,
    /// USD / USDA / USDC.
    Usd,
    /// Alembic (.abc).
    Alembic,
}

impl FileFormat {
    /// Detect the format from a file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "f3d" | "forge3d" => Some(Self::Forge3D),
            "blend" => Some(Self::Blend),
            "gltf" | "glb" => Some(Self::GlTF),
            "obj" => Some(Self::Obj),
            "fbx" => Some(Self::Fbx),
            "stl" => Some(Self::Stl),
            "ply" => Some(Self::Ply),
            "usd" | "usda" | "usdc" | "usdz" => Some(Self::Usd),
            "abc" => Some(Self::Alembic),
            _ => None,
        }
    }

    /// Detect the format from a file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    /// Canonical file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Forge3D => "f3d",
            Self::Blend => "blend",
            Self::GlTF => "glb",
            Self::Obj => "obj",
            Self::Fbx => "fbx",
            Self::Stl => "stl",
            Self::Ply => "ply",
            Self::Usd => "usdc",
            Self::Alembic => "abc",
        }
    }
}

/// Trait for pluggable format importers.
pub trait FormatImporter: Send + Sync {
    /// The format this importer handles.
    fn format(&self) -> FileFormat;

    /// Import from bytes into the scene.
    fn import(&self, data: &[u8], scene: &mut forge3d_scene::Scene) -> IoResult<()>;
}

/// Trait for pluggable format exporters.
pub trait FormatExporter: Send + Sync {
    /// The format this exporter handles.
    fn format(&self) -> FileFormat;

    /// Export the scene into bytes.
    fn export(&self, scene: &forge3d_scene::Scene) -> IoResult<Vec<u8>>;
}

/// Registry of format importers and exporters.
pub struct FormatRegistry {
    importers: Vec<Box<dyn FormatImporter>>,
    exporters: Vec<Box<dyn FormatExporter>>,
}

impl FormatRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            importers: Vec::new(),
            exporters: Vec::new(),
        }
    }

    /// Register an importer.
    pub fn register_importer(&mut self, importer: Box<dyn FormatImporter>) {
        self.importers.push(importer);
    }

    /// Register an exporter.
    pub fn register_exporter(&mut self, exporter: Box<dyn FormatExporter>) {
        self.exporters.push(exporter);
    }

    /// Find an importer for the given format.
    pub fn find_importer(&self, format: FileFormat) -> Option<&dyn FormatImporter> {
        self.importers
            .iter()
            .find(|i| i.format() == format)
            .map(|i| i.as_ref())
    }

    /// Find an exporter for the given format.
    pub fn find_exporter(&self, format: FileFormat) -> Option<&dyn FormatExporter> {
        self.exporters
            .iter()
            .find(|e| e.format() == format)
            .map(|e| e.as_ref())
    }
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}
