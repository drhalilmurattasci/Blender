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

#[cfg(test)]
mod tests {
    use super::*;
    use blend_import::{BlendHeader, BlendFile, PointerSize, Endianness};

    fn make_blend_header(ptr: u8, endian: u8, ver: &[u8; 3]) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"BLENDER");
        data.push(ptr);
        data.push(endian);
        data.extend_from_slice(ver);
        data
    }

    #[test]
    fn blend_header_valid_32bit_le() {
        let data = make_blend_header(b'_', b'v', b"401");
        let h = BlendHeader::parse(&data).unwrap();
        assert_eq!(h.pointer_size, PointerSize::Bits32);
        assert_eq!(h.endianness, Endianness::Little);
        assert_eq!(h.version_string(), "4.01");
        assert_eq!(h.version_number(), 401);
    }

    #[test]
    fn blend_header_valid_64bit_be() {
        let data = make_blend_header(b'-', b'V', b"310");
        let h = BlendHeader::parse(&data).unwrap();
        assert_eq!(h.pointer_size, PointerSize::Bits64);
        assert_eq!(h.endianness, Endianness::Big);
        assert_eq!(h.version_string(), "3.10");
        assert_eq!(h.version_number(), 310);
    }

    #[test]
    fn blend_header_too_short() {
        let data = b"BLENDER";
        let result = BlendHeader::parse(data);
        assert!(result.is_err());
    }

    #[test]
    fn blend_header_bad_magic() {
        let data = b"NOTBLEND____";
        let result = BlendHeader::parse(data);
        assert!(result.is_err());
    }

    #[test]
    fn blend_header_bad_pointer_size() {
        let data = make_blend_header(b'X', b'v', b"401");
        let result = BlendHeader::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn blend_header_bad_endianness() {
        let data = make_blend_header(b'-', b'x', b"401");
        let result = BlendHeader::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn blend_header_invalid_version_digit() {
        let data = make_blend_header(b'-', b'v', b"4A1");
        let result = BlendHeader::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn blend_file_empty_block_table() {
        // Header + ENDB block.
        let mut data = make_blend_header(b'-', b'v', b"401");
        data.extend_from_slice(b"ENDB");
        data.extend_from_slice(&[0u8; 20]); // Padding for block header fields.
        let bf = BlendFile::parse(&data).unwrap();
        assert!(bf.blocks.is_empty());
    }

    #[test]
    fn blend_file_truncated_block() {
        // Header + partial block (data goes past EOF).
        let mut data = make_blend_header(b'_', b'v', b"401");
        // Block header for 32-bit: code(4) + size(4) + addr(4) + sdna(4) + count(4) = 20
        data.extend_from_slice(b"OB\0\0");
        data.extend_from_slice(&100u32.to_le_bytes()); // size = 100
        data.extend_from_slice(&[0u8; 12]); // old_address + sdna + count
        // No block data follows -- should not panic.
        let bf = BlendFile::parse(&data).unwrap();
        // The block header is read but data is truncated, so we break early.
        assert!(bf.blocks.len() <= 1);
    }

    #[test]
    fn file_format_from_extension() {
        use format::FileFormat;
        assert_eq!(FileFormat::from_extension("blend"), Some(FileFormat::Blend));
        assert_eq!(FileFormat::from_extension("f3d"), Some(FileFormat::Forge3D));
        assert_eq!(FileFormat::from_extension("GLTF"), Some(FileFormat::GlTF));
        assert_eq!(FileFormat::from_extension("unknown"), None);
    }

    #[test]
    fn file_format_from_path() {
        use format::FileFormat;
        use std::path::Path;
        assert_eq!(
            FileFormat::from_path(Path::new("model.obj")),
            Some(FileFormat::Obj)
        );
        assert_eq!(FileFormat::from_path(Path::new("no_ext")), None);
    }

    #[test]
    fn format_registry_empty() {
        let reg = format::FormatRegistry::new();
        assert!(reg.find_importer(format::FileFormat::Blend).is_none());
        assert!(reg.find_exporter(format::FileFormat::Blend).is_none());
    }

    #[test]
    fn versioning_pipeline_empty() {
        let p = versioning::VersioningPipeline::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn file_reader_nonexistent() {
        let result = read::FileReader::open("/tmp/nonexistent_file.blend");
        assert!(result.is_err());
    }

    #[test]
    fn file_reader_unknown_format() {
        let result = read::FileReader::open("/tmp/test.xyz");
        assert!(result.is_err());
    }
}
