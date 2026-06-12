//! FAFB Header Implementation
//!
//! The 32-byte header that identifies and describes a .fafb file.
//!
//! Layout:
//! ```text
//! Offset  Size  Field
//! ------  ----  -----
//! 0       4     magic (b"FAFB")
//! 4       1     version_major
//! 5       1     version_minor
//! 6       2     flags
//! 8       4     source_checksum (CRC32)
//! 12      8     created_timestamp (Unix)
//! 20      2     section_count
//! 22      4     section_table_offset
//! 26      2     string_table_index
//! 28      4     total_size
//! ------  ----
//! Total: 32 bytes
//! ```

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

use super::error::{FafbError, FafbResult};
use super::flags::{FLAG_STRING_TABLE, Flags};

/// Magic number identifying FAFB files: "FAFB" in ASCII
pub const MAGIC: [u8; 4] = *b"FAFB";

/// Magic number as u32 (little-endian)
pub const MAGIC_U32: u32 = 0x4246_4146; // "FAFB" little-endian

/// Format major version
pub const VERSION_MAJOR: u8 = 1;

/// Current format minor version (additive changes)
pub const VERSION_MINOR: u8 = 0;

/// Header size in bytes
pub const HEADER_SIZE: usize = 32;

/// Maximum allowed section count (DoS protection)
pub const MAX_SECTIONS: u16 = 256;

/// Maximum allowed file size (DoS protection): 10MB
pub const MAX_FILE_SIZE: u32 = 10 * 1024 * 1024;

/// The 32-byte FAFB file header
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FafbHeader {
    // Identification (8 bytes)
    /// Format version - major (breaking changes)
    pub version_major: u8,
    /// Format version - minor (additive changes)
    pub version_minor: u8,
    /// Feature flags
    pub flags: Flags,

    // Integrity (12 bytes)
    /// CRC32 checksum of original .faf YAML source
    pub source_checksum: u32,
    /// Unix timestamp when .fafb was created
    pub created_timestamp: u64,

    // Index (8 bytes)
    /// Number of sections in the file
    pub section_count: u16,
    /// Byte offset to section table (from start of file)
    pub section_table_offset: u32,
    /// Index of string table entry in section table
    pub string_table_index: u16,

    // Size (4 bytes)
    /// Total file size in bytes
    pub total_size: u32,
}

impl FafbHeader {
    /// Create a new header with default version and STRING_TABLE flag set
    pub fn new() -> Self {
        Self {
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            flags: Flags::from_raw(FLAG_STRING_TABLE),
            source_checksum: 0,
            created_timestamp: 0,
            section_count: 0,
            section_table_offset: HEADER_SIZE as u32,
            string_table_index: 0,
            total_size: HEADER_SIZE as u32,
        }
    }

    /// Create header with current timestamp
    pub fn with_timestamp() -> Self {
        let mut header = Self::new();
        header.created_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        header
    }

    /// Compute CRC32 checksum of source YAML
    pub fn compute_checksum(yaml_source: &[u8]) -> u32 {
        crc32fast::hash(yaml_source)
    }

    /// Set the source checksum from YAML content
    pub fn set_source_checksum(&mut self, yaml_source: &[u8]) {
        self.source_checksum = Self::compute_checksum(yaml_source);
    }

    /// Write header to a byte buffer
    pub fn write<W: Write>(&self, writer: &mut W) -> FafbResult<()> {
        writer.write_all(&MAGIC)?;
        writer.write_u8(self.version_major)?;
        writer.write_u8(self.version_minor)?;
        writer.write_u16::<LittleEndian>(self.flags.raw())?;
        writer.write_u32::<LittleEndian>(self.source_checksum)?;
        writer.write_u64::<LittleEndian>(self.created_timestamp)?;
        writer.write_u16::<LittleEndian>(self.section_count)?;
        writer.write_u32::<LittleEndian>(self.section_table_offset)?;
        writer.write_u16::<LittleEndian>(self.string_table_index)?;
        writer.write_u32::<LittleEndian>(self.total_size)?;
        Ok(())
    }

    /// Write header to a new `Vec<u8>`
    pub fn to_bytes(&self) -> FafbResult<Vec<u8>> {
        let mut buf = Vec::with_capacity(HEADER_SIZE);
        self.write(&mut buf)?;
        Ok(buf)
    }

    /// Read header from a byte buffer
    pub fn read<R: Read>(reader: &mut R) -> FafbResult<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        let magic_u32 = u32::from_le_bytes(magic);
        if magic_u32 != MAGIC_U32 {
            return Err(FafbError::InvalidMagic(magic_u32));
        }

        let version_major = reader.read_u8()?;
        let version_minor = reader.read_u8()?;

        if version_major != VERSION_MAJOR {
            return Err(FafbError::IncompatibleVersion {
                expected: VERSION_MAJOR,
                actual: version_major,
            });
        }

        let flags = Flags::from_raw(reader.read_u16::<LittleEndian>()?);
        let source_checksum = reader.read_u32::<LittleEndian>()?;
        let created_timestamp = reader.read_u64::<LittleEndian>()?;

        let section_count = reader.read_u16::<LittleEndian>()?;
        if section_count > MAX_SECTIONS {
            return Err(FafbError::TooManySections {
                count: section_count,
                max: MAX_SECTIONS,
            });
        }

        let section_table_offset = reader.read_u32::<LittleEndian>()?;
        let string_table_index = reader.read_u16::<LittleEndian>()?;

        let total_size = reader.read_u32::<LittleEndian>()?;
        if total_size > MAX_FILE_SIZE {
            return Err(FafbError::SizeMismatch {
                header_size: total_size,
                actual_size: MAX_FILE_SIZE as usize,
            });
        }

        Ok(Self {
            version_major,
            version_minor,
            flags,
            source_checksum,
            created_timestamp,
            section_count,
            section_table_offset,
            string_table_index,
            total_size,
        })
    }

    /// Read header from a byte slice
    pub fn from_bytes(data: &[u8]) -> FafbResult<Self> {
        if data.len() < HEADER_SIZE {
            return Err(FafbError::FileTooSmall {
                expected: HEADER_SIZE,
                actual: data.len(),
            });
        }

        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor)
    }

    /// Validate header against actual file data
    pub fn validate(&self, file_data: &[u8]) -> FafbResult<()> {
        if self.total_size as usize != file_data.len() {
            return Err(FafbError::SizeMismatch {
                header_size: self.total_size,
                actual_size: file_data.len(),
            });
        }

        if self.section_table_offset > self.total_size {
            return Err(FafbError::InvalidSectionTableOffset {
                offset: self.section_table_offset,
                file_size: self.total_size,
            });
        }

        Ok(())
    }

    /// Check if this header is compatible with the current version
    pub fn is_compatible(&self) -> bool {
        self.version_major == VERSION_MAJOR
    }

    /// Get version as string (e.g., "1.0")
    pub fn version_string(&self) -> String {
        format!("{}.{}", self.version_major, self.version_minor)
    }
}

impl Default for FafbHeader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        let header = FafbHeader::new();
        let bytes = header.to_bytes().unwrap();
        assert_eq!(bytes.len(), HEADER_SIZE);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_magic_bytes() {
        let header = FafbHeader::new();
        let bytes = header.to_bytes().unwrap();
        assert_eq!(&bytes[0..4], b"FAFB");
    }

    #[test]
    fn test_roundtrip() {
        let mut original = FafbHeader::with_timestamp();
        original.source_checksum = 0xDEADBEEF;
        original.section_count = 5;
        original.section_table_offset = 1024;
        original.total_size = 2048;
        original.string_table_index = 4;
        original.flags.set_compressed(true);
        original.flags.set_embeddings(true);

        let bytes = original.to_bytes().unwrap();
        let recovered = FafbHeader::from_bytes(&bytes).unwrap();

        assert_eq!(original.version_major, recovered.version_major);
        assert_eq!(original.version_minor, recovered.version_minor);
        assert_eq!(original.flags, recovered.flags);
        assert_eq!(original.source_checksum, recovered.source_checksum);
        assert_eq!(original.created_timestamp, recovered.created_timestamp);
        assert_eq!(original.section_count, recovered.section_count);
        assert_eq!(
            original.section_table_offset,
            recovered.section_table_offset
        );
        assert_eq!(original.string_table_index, recovered.string_table_index);
        assert_eq!(original.total_size, recovered.total_size);
    }

    #[test]
    fn test_invalid_magic() {
        let mut bytes = FafbHeader::new().to_bytes().unwrap();
        bytes[0] = 0x00;

        let result = FafbHeader::from_bytes(&bytes);
        assert!(matches!(result, Err(FafbError::InvalidMagic(_))));
    }

    #[test]
    fn test_incompatible_version() {
        let mut bytes = FafbHeader::new().to_bytes().unwrap();
        bytes[4] = 99;

        let result = FafbHeader::from_bytes(&bytes);
        assert!(matches!(
            result,
            Err(FafbError::IncompatibleVersion {
                expected: 1,
                actual: 99
            })
        ));
    }

    #[test]
    fn test_file_too_small() {
        let bytes = vec![0u8; 16];

        let result = FafbHeader::from_bytes(&bytes);
        assert!(matches!(
            result,
            Err(FafbError::FileTooSmall {
                expected: 32,
                actual: 16
            })
        ));
    }

    #[test]
    fn test_too_many_sections() {
        let mut header = FafbHeader::new();
        header.section_count = 300;

        let bytes = header.to_bytes().unwrap();
        let result = FafbHeader::from_bytes(&bytes);

        assert!(matches!(
            result,
            Err(FafbError::TooManySections {
                count: 300,
                max: 256
            })
        ));
    }

    #[test]
    fn test_checksum_computation() {
        let yaml = b"faf_version: 2.5.0\nproject:\n  name: test";
        let checksum = FafbHeader::compute_checksum(yaml);
        assert_eq!(checksum, FafbHeader::compute_checksum(yaml));

        let yaml2 = b"faf_version: 2.5.0\nproject:\n  name: different";
        assert_ne!(checksum, FafbHeader::compute_checksum(yaml2));
    }

    #[test]
    fn test_validate_size_mismatch() {
        let mut header = FafbHeader::new();
        header.total_size = 100;

        let data = vec![0u8; 50];

        let result = header.validate(&data);
        assert!(matches!(
            result,
            Err(FafbError::SizeMismatch {
                header_size: 100,
                actual_size: 50
            })
        ));
    }

    #[test]
    fn test_validate_invalid_section_offset() {
        let mut header = FafbHeader::new();
        header.total_size = 100;
        header.section_table_offset = 200;

        let data = vec![0u8; 100];

        let result = header.validate(&data);
        assert!(matches!(
            result,
            Err(FafbError::InvalidSectionTableOffset {
                offset: 200,
                file_size: 100
            })
        ));
    }

    #[test]
    fn test_version_string() {
        let header = FafbHeader::new();
        assert_eq!(header.version_string(), "1.0");
    }

    #[test]
    fn test_flags_preserved() {
        let mut header = FafbHeader::new();
        header.flags.set_compressed(true);
        header.flags.set_signed(true);

        let bytes = header.to_bytes().unwrap();
        let recovered = FafbHeader::from_bytes(&bytes).unwrap();

        assert!(recovered.flags.is_compressed());
        assert!(recovered.flags.is_signed());
        assert!(!recovered.flags.has_embeddings());
        assert!(recovered.flags.has_string_table());
    }

    #[test]
    fn test_unknown_flags_ignored() {
        let mut header = FafbHeader::new();
        header.flags = Flags::from_raw(0xFF00 | FLAG_STRING_TABLE);

        let bytes = header.to_bytes().unwrap();
        let recovered = FafbHeader::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.flags.raw(), 0xFF00 | FLAG_STRING_TABLE);
    }

    #[test]
    fn test_string_table_flag_always_set() {
        let header = FafbHeader::new();
        assert!(header.flags.has_string_table());
    }

    #[test]
    fn test_string_table_index_roundtrip() {
        let mut header = FafbHeader::new();
        header.string_table_index = 7;
        header.total_size = 1000;
        header.section_count = 8;

        let bytes = header.to_bytes().unwrap();
        let recovered = FafbHeader::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.string_table_index, 7);
    }
}
