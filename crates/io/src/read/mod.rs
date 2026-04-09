//! File reading pipeline: open, detect format, parse, and populate a scene.

use crate::format::{FileFormat, FormatRegistry};
use crate::{IoError, IoResult};
use std::path::{Path, PathBuf};

/// High-level file reader.
pub struct FileReader {
    /// Path to the file being read.
    pub path: PathBuf,
    /// Detected file format.
    pub format: FileFormat,
}

impl FileReader {
    /// Create a reader for the given path, auto-detecting the format.
    pub fn open(path: impl AsRef<Path>) -> IoResult<Self> {
        let path = path.as_ref().to_path_buf();
        let format = FileFormat::from_path(&path).ok_or_else(|| {
            IoError::UnsupportedFormat(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("(none)")
                    .to_string(),
            )
        })?;

        if !path.exists() {
            return Err(IoError::FileNotFound(path.display().to_string()));
        }

        Ok(Self { path, format })
    }

    /// Read the file into a scene using the given format registry.
    pub fn read_into(
        &self,
        scene: &mut forge3d_scene::Scene,
        registry: &FormatRegistry,
    ) -> IoResult<()> {
        let data = std::fs::read(&self.path)?;

        let importer = registry
            .find_importer(self.format)
            .ok_or_else(|| IoError::UnsupportedFormat(format!("{:?}", self.format)))?;

        tracing::info!(
            path = %self.path.display(),
            format = ?self.format,
            bytes = data.len(),
            "reading file"
        );

        importer.import(&data, scene)
    }
}
