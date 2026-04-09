//! # forge3d-io
//!
//! File I/O for Forge3D: .blend import, native binary format (rkyv),
//! read/write pipeline, and file versioning.

pub mod blend_import;
pub mod format;
pub mod read;
pub mod versioning;
pub mod write;

pub use format::{FileFormat, FormatExporter, FormatImporter, FormatRegistry};
pub use read::FileReader;
pub use versioning::{Migration, VersioningPipeline, CURRENT_VERSION};
pub use write::FileWriter;

use thiserror::Error;

/// Errors from file I/O operations.
#[derive(Debug, Error)]
pub enum IoError {
    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("parse error at offset {offset}: {detail}")]
    ParseError { offset: u64, detail: String },

    #[error("version mismatch: file v{file_version}, expected v{expected_version}")]
    VersionMismatch {
        file_version: u32,
        expected_version: u32,
    },

    #[error("serialization failed: {0}")]
    SerializationError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

/// Result alias.
pub type IoResult<T> = Result<T, IoError>;
