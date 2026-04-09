//! File writing pipeline: serialize scene data and write to disk.

use crate::format::{FileFormat, FormatRegistry};
use crate::{IoError, IoResult};
use std::path::{Path, PathBuf};

/// High-level file writer.
pub struct FileWriter {
    /// Destination path.
    pub path: PathBuf,
    /// Target format.
    pub format: FileFormat,
}

impl FileWriter {
    /// Create a writer for the given path, auto-detecting the format.
    pub fn create(path: impl AsRef<Path>) -> IoResult<Self> {
        let path = path.as_ref().to_path_buf();
        let format = FileFormat::from_path(&path).ok_or_else(|| {
            IoError::UnsupportedFormat(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("(none)")
                    .to_string(),
            )
        })?;
        Ok(Self { path, format })
    }

    /// Create a writer with an explicit format.
    pub fn with_format(path: impl AsRef<Path>, format: FileFormat) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            format,
        }
    }

    /// Serialize the scene and write to disk.
    pub fn write(
        &self,
        scene: &forge3d_scene::Scene,
        registry: &FormatRegistry,
    ) -> IoResult<()> {
        let exporter = registry
            .find_exporter(self.format)
            .ok_or_else(|| IoError::UnsupportedFormat(format!("{:?}", self.format)))?;

        tracing::info!(
            path = %self.path.display(),
            format = ?self.format,
            "writing file"
        );

        let data = exporter.export(scene)?;

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&self.path, &data)?;

        tracing::info!(
            bytes = data.len(),
            "file written successfully"
        );

        Ok(())
    }
}
