//! Blender .blend file import support.
//!
//! The .blend format is a memory dump of Blender's internal data structures.
//! This module provides a low-level parser for the file header, block table,
//! and DNA structs, plus a high-level converter that populates a Forge3D scene.

use crate::{IoError, IoResult};

/// Magic bytes at the start of every .blend file.
const BLEND_MAGIC: &[u8; 7] = b"BLENDER";

/// Pointer size used in the .blend file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerSize {
    Bits32,
    Bits64,
}

/// Endianness of the .blend file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

/// Parsed .blend file header.
#[derive(Debug, Clone)]
pub struct BlendHeader {
    pub pointer_size: PointerSize,
    pub endianness: Endianness,
    pub version: [u8; 3],
}

impl BlendHeader {
    /// Parse the header from the first 12 bytes of a .blend file.
    pub fn parse(data: &[u8]) -> IoResult<Self> {
        if data.len() < 12 {
            return Err(IoError::ParseError {
                offset: 0,
                detail: "file too short for .blend header".into(),
            });
        }

        if &data[0..7] != BLEND_MAGIC {
            return Err(IoError::ParseError {
                offset: 0,
                detail: "not a .blend file (bad magic)".into(),
            });
        }

        let pointer_size = match data[7] {
            b'_' => PointerSize::Bits32,
            b'-' => PointerSize::Bits64,
            c => {
                return Err(IoError::ParseError {
                    offset: 7,
                    detail: format!("unknown pointer size byte: 0x{c:02X}"),
                })
            }
        };

        let endianness = match data[8] {
            b'v' => Endianness::Little,
            b'V' => Endianness::Big,
            c => {
                return Err(IoError::ParseError {
                    offset: 8,
                    detail: format!("unknown endianness byte: 0x{c:02X}"),
                })
            }
        };

        let version = [data[9], data[10], data[11]];

        // Validate version digits are ASCII '0'..'9'.
        for (i, &byte) in version.iter().enumerate() {
            if !byte.is_ascii_digit() {
                return Err(IoError::ParseError {
                    offset: 9 + i as u64,
                    detail: format!("invalid version digit: 0x{byte:02X}"),
                });
            }
        }

        Ok(Self {
            pointer_size,
            endianness,
            version,
        })
    }

    /// Version as a human-readable string (e.g. "4.01").
    ///
    /// Blender encodes the version as three ASCII digits. For example,
    /// `b"401"` means version 4.01 (major = hundreds digit, minor = tens
    /// and units digits combined). This matches Blender's `bhead->version`
    /// convention where the integer `401` = version 4.01.
    pub fn version_string(&self) -> String {
        let major = self.version[0] - b'0';
        let minor_tens = self.version[1] - b'0';
        let minor_units = self.version[2] - b'0';
        format!("{}.{}{}", major, minor_tens, minor_units)
    }

    /// Version as a numeric triple (major, minor_tens, minor_units).
    pub fn version_number(&self) -> u32 {
        let major = (self.version[0] - b'0') as u32;
        let minor_tens = (self.version[1] - b'0') as u32;
        let minor_units = (self.version[2] - b'0') as u32;
        major * 100 + minor_tens * 10 + minor_units
    }
}

/// A raw file-block entry in the .blend block table.
#[derive(Debug, Clone)]
pub struct BlendBlock {
    /// 4-byte block code (e.g. b"OB\0\0" for Object).
    pub code: [u8; 4],
    /// Total byte size of the block data.
    pub size: u32,
    /// Original memory address (used for pointer resolution).
    pub old_address: u64,
    /// DNA struct index.
    pub sdna_index: u32,
    /// Number of structs stored in this block.
    pub count: u32,
    /// Byte offset of the block data within the file.
    pub data_offset: u64,
}

/// Low-level .blend file representation.
#[derive(Debug)]
pub struct BlendFile {
    pub header: BlendHeader,
    pub blocks: Vec<BlendBlock>,
}

impl BlendFile {
    /// Parse the block table from raw file bytes.
    pub fn parse(data: &[u8]) -> IoResult<Self> {
        let header = BlendHeader::parse(data)?;

        let ptr_size: usize = match header.pointer_size {
            PointerSize::Bits32 => 4,
            PointerSize::Bits64 => 8,
        };

        let mut blocks = Vec::new();
        let mut cursor = 12usize; // skip header

        // Block header size: 4 (code) + 4 (size) + ptr_size (old_address)
        //                    + 4 (sdna_index) + 4 (count)
        let block_header_size = 4 + 4 + ptr_size + 4 + 4;

        while cursor + block_header_size <= data.len() {
            let code: [u8; 4] = data[cursor..cursor + 4].try_into().unwrap();

            // ENDB marks the end of the block table.
            if &code == b"ENDB" {
                break;
            }

            let size = read_u32(&data[cursor + 4..], header.endianness);

            let old_address = if ptr_size == 4 {
                read_u32(&data[cursor + 8..], header.endianness) as u64
            } else {
                read_u64(&data[cursor + 8..], header.endianness)
            };

            let sdna_offset = cursor + 8 + ptr_size;
            let sdna_index = read_u32(&data[sdna_offset..], header.endianness);
            let count = read_u32(&data[sdna_offset + 4..], header.endianness);
            let data_offset = (cursor + block_header_size) as u64;

            blocks.push(BlendBlock {
                code,
                size,
                old_address,
                sdna_index,
                count,
                data_offset,
            });

            let next = cursor + block_header_size + size as usize;
            if next > data.len() {
                // Truncated block -- stop parsing rather than panicking.
                break;
            }
            cursor = next;
        }

        Ok(Self { header, blocks })
    }
}

fn read_u32(data: &[u8], endianness: Endianness) -> u32 {
    let bytes: [u8; 4] = data
        .get(..4)
        .and_then(|s| s.try_into().ok())
        .unwrap_or([0; 4]);
    match endianness {
        Endianness::Little => u32::from_le_bytes(bytes),
        Endianness::Big => u32::from_be_bytes(bytes),
    }
}

fn read_u64(data: &[u8], endianness: Endianness) -> u64 {
    let bytes: [u8; 8] = data
        .get(..8)
        .and_then(|s| s.try_into().ok())
        .unwrap_or([0; 8]);
    match endianness {
        Endianness::Little => u64::from_le_bytes(bytes),
        Endianness::Big => u64::from_be_bytes(bytes),
    }
}
