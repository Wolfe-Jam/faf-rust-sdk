//! FAFB Section Entry and Section Table
//!
//! The section table is located at the end of the file (at section_table_offset).
//! Each entry is 16 bytes and describes one section's location and metadata.
//!
//! ## Section Entry Layout (16 bytes)
//!
//! ```text
//! Offset  Size  Field
//! ------  ----  -----
//! 0       1     section_type
//! 1       1     priority
//! 2       4     offset
//! 6       4     length
//! 10      2     token_count
//! 12      4     flags (section-specific)
//! ------  ----
//! Total: 16 bytes
//! ```

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use super::chunk_registry::{CLASSIFICATION_MASK, ChunkClassification};
use super::error::{FafbError, FafbResult};
use super::priority::Priority;
use super::section_type::SectionType;

/// Size of a single section entry in bytes
pub const SECTION_ENTRY_SIZE: usize = 16;

/// A single section entry in the section table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionEntry {
    /// Section type identifier
    pub section_type: SectionType,
    /// Truncation priority (0-255, higher = more important)
    pub priority: Priority,
    /// Byte offset to section data (from start of file)
    pub offset: u32,
    /// Section data length in bytes
    pub length: u32,
    /// Pre-computed token count estimate
    pub token_count: u16,
    /// Section-specific flags (4 bytes for alignment)
    pub flags: u32,
}

impl SectionEntry {
    /// Create a new section entry with default priority
    pub fn new(section_type: SectionType, offset: u32, length: u32) -> Self {
        Self {
            section_type,
            priority: Priority::new(section_type.default_priority()),
            offset,
            length,
            token_count: estimate_tokens(length),
            flags: 0,
        }
    }

    /// Create with explicit priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Create with explicit token count
    pub fn with_token_count(mut self, count: u16) -> Self {
        self.token_count = count;
        self
    }

    /// Create with section-specific flags
    pub fn with_flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }

    /// Set classification in the low 2 bits of flags (v2)
    pub fn with_classification(mut self, classification: ChunkClassification) -> Self {
        // Clear low 2 bits, then set classification
        self.flags = (self.flags & !CLASSIFICATION_MASK) | classification.bits();
        self
    }

    /// Get the classification from the low 2 bits of flags (v2)
    pub fn classification(&self) -> ChunkClassification {
        ChunkClassification::from_bits(self.flags)
    }

    /// Get section-specific flags (bits 2+, excluding classification)
    pub fn section_flags(&self) -> u32 {
        self.flags & !CLASSIFICATION_MASK
    }

    /// Write entry to a byte buffer
    pub fn write<W: Write>(&self, writer: &mut W) -> FafbResult<()> {
        writer.write_u8(self.section_type.id())?;
        writer.write_u8(self.priority.value())?;
        writer.write_u32::<LittleEndian>(self.offset)?;
        writer.write_u32::<LittleEndian>(self.length)?;
        writer.write_u16::<LittleEndian>(self.token_count)?;
        writer.write_u32::<LittleEndian>(self.flags)?;
        Ok(())
    }

    /// Write entry to a new `Vec<u8>`
    pub fn to_bytes(&self) -> FafbResult<Vec<u8>> {
        let mut buf = Vec::with_capacity(SECTION_ENTRY_SIZE);
        self.write(&mut buf)?;
        Ok(buf)
    }

    /// Read entry from a byte buffer
    pub fn read<R: Read>(reader: &mut R) -> FafbResult<Self> {
        let section_type = SectionType::from(reader.read_u8()?);
        let priority = Priority::from(reader.read_u8()?);
        let offset = reader.read_u32::<LittleEndian>()?;
        let length = reader.read_u32::<LittleEndian>()?;
        let token_count = reader.read_u16::<LittleEndian>()?;
        let flags = reader.read_u32::<LittleEndian>()?;

        Ok(Self {
            section_type,
            priority,
            offset,
            length,
            token_count,
            flags,
        })
    }

    /// Read entry from a byte slice
    pub fn from_bytes(data: &[u8]) -> FafbResult<Self> {
        if data.len() < SECTION_ENTRY_SIZE {
            return Err(FafbError::FileTooSmall {
                expected: SECTION_ENTRY_SIZE,
                actual: data.len(),
            });
        }
        let mut cursor = std::io::Cursor::new(data);
        Self::read(&mut cursor)
    }

    /// Check if this section's data range is valid within a file
    pub fn validate_bounds(&self, file_size: u32) -> FafbResult<()> {
        // WHY: checked_add prevents integer overflow attacks where offset + length wraps
        // around u32::MAX to produce a small "end" that passes the bounds check
        // Example attack: offset=0xFFFFFF00, length=0x200 would wrap to 0x100
        let end =
            self.offset
                .checked_add(self.length)
                .ok_or(FafbError::InvalidSectionTableOffset {
                    offset: self.offset,
                    file_size,
                })?;

        // WHY: Bounds check prevents reading past file end - memory safety
        if end > file_size {
            return Err(FafbError::InvalidSectionTableOffset {
                offset: self.offset,
                file_size,
            });
        }

        Ok(())
    }
}

/// The section table containing all section entries
#[derive(Debug, Clone, Default)]
pub struct SectionTable {
    entries: Vec<SectionEntry>,
}

impl SectionTable {
    /// Create an empty section table
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Add a section entry
    pub fn push(&mut self, entry: SectionEntry) {
        self.entries.push(entry);
    }

    /// Get number of sections
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if table is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get entry by index
    pub fn get(&self, index: usize) -> Option<&SectionEntry> {
        self.entries.get(index)
    }

    /// Get entry by section type
    pub fn get_by_type(&self, section_type: SectionType) -> Option<&SectionEntry> {
        self.entries.iter().find(|e| e.section_type == section_type)
    }

    /// Get all entries
    pub fn entries(&self) -> &[SectionEntry] {
        &self.entries
    }

    /// Get entries sorted by priority (highest first)
    pub fn entries_by_priority(&self) -> Vec<&SectionEntry> {
        let mut sorted: Vec<_> = self.entries.iter().collect();
        sorted.sort_by_key(|e| std::cmp::Reverse(e.priority));
        sorted
    }

    /// Get entries that fit within a token budget
    pub fn entries_within_budget(&self, budget: u16) -> Vec<&SectionEntry> {
        // WHY: Priority-first traversal ensures highest-value sections get budget first
        // This is a greedy algorithm - optimal for most use cases where priorities
        // accurately reflect importance
        let mut result = Vec::new();
        let mut remaining = budget;

        for entry in self.entries_by_priority() {
            if entry.token_count <= remaining {
                result.push(entry);
                remaining -= entry.token_count;
            } else if entry.priority.is_critical() {
                // WHY: Critical sections always included - they define project identity
                // (e.g., project name, version) and are small enough to never skip
                result.push(entry);
            }
            // WHY: Non-critical sections that don't fit are silently dropped
            // This enables graceful degradation under tight token budgets
        }

        result
    }

    /// Calculate total token count
    pub fn total_tokens(&self) -> u32 {
        self.entries.iter().map(|e| e.token_count as u32).sum()
    }

    /// Calculate total size in bytes (for section table only)
    pub fn table_size(&self) -> usize {
        self.entries.len() * SECTION_ENTRY_SIZE
    }

    /// Write section table to a byte buffer
    pub fn write<W: Write>(&self, writer: &mut W) -> FafbResult<()> {
        for entry in &self.entries {
            entry.write(writer)?;
        }
        Ok(())
    }

    /// Write section table to a new `Vec<u8>`
    pub fn to_bytes(&self) -> FafbResult<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.table_size());
        self.write(&mut buf)?;
        Ok(buf)
    }

    /// Read section table from a byte buffer
    pub fn read<R: Read>(reader: &mut R, count: usize) -> FafbResult<Self> {
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(SectionEntry::read(reader)?);
        }
        Ok(Self { entries })
    }

    /// Read section table from a byte slice
    pub fn from_bytes(data: &[u8], count: usize) -> FafbResult<Self> {
        let expected_size = count * SECTION_ENTRY_SIZE;
        if data.len() < expected_size {
            return Err(FafbError::FileTooSmall {
                expected: expected_size,
                actual: data.len(),
            });
        }
        let mut cursor = std::io::Cursor::new(data);
        Self::read(&mut cursor, count)
    }

    /// Validate all entries against file size
    pub fn validate_bounds(&self, file_size: u32) -> FafbResult<()> {
        for entry in &self.entries {
            entry.validate_bounds(file_size)?;
        }
        Ok(())
    }
}

impl IntoIterator for SectionTable {
    type Item = SectionEntry;
    type IntoIter = std::vec::IntoIter<SectionEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a> IntoIterator for &'a SectionTable {
    type Item = &'a SectionEntry;
    type IntoIter = std::slice::Iter<'a, SectionEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

/// Estimate token count from byte length
/// Rough estimate: ~4 bytes per token for English text
fn estimate_tokens(byte_length: u32) -> u16 {
    // WHY: 4 bytes/token is the empirical average for English prose in BPE tokenizers
    // Code tends to be slightly higher (3-3.5), YAML slightly lower (4-5)
    // WHY: u16::MAX cap prevents overflow - sections >256KB truncate to max tokens
    // This is acceptable because such large sections will likely be truncated anyway
    std::cmp::min(byte_length / 4, u16::MAX as u32) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_entry_size() {
        let entry = SectionEntry::new(SectionType::Meta, 32, 100);
        let bytes = entry.to_bytes().unwrap();
        assert_eq!(bytes.len(), SECTION_ENTRY_SIZE);
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_section_entry_roundtrip() {
        let original = SectionEntry::new(SectionType::TechStack, 64, 256)
            .with_priority(Priority::high())
            .with_token_count(100)
            .with_flags(0xDEADBEEF);

        let bytes = original.to_bytes().unwrap();
        let recovered = SectionEntry::from_bytes(&bytes).unwrap();

        assert_eq!(original.section_type, recovered.section_type);
        assert_eq!(original.priority, recovered.priority);
        assert_eq!(original.offset, recovered.offset);
        assert_eq!(original.length, recovered.length);
        assert_eq!(original.token_count, recovered.token_count);
        assert_eq!(original.flags, recovered.flags);
    }

    #[test]
    fn test_section_entry_default_priority() {
        let meta = SectionEntry::new(SectionType::Meta, 0, 100);
        assert_eq!(meta.priority.value(), 255); // Critical

        let tech = SectionEntry::new(SectionType::TechStack, 0, 100);
        assert_eq!(tech.priority.value(), 200); // High

        let context = SectionEntry::new(SectionType::Context, 0, 100);
        assert_eq!(context.priority.value(), 64); // Low
    }

    #[test]
    fn test_token_estimation() {
        assert_eq!(estimate_tokens(0), 0);
        assert_eq!(estimate_tokens(4), 1);
        assert_eq!(estimate_tokens(100), 25);
        assert_eq!(estimate_tokens(1000), 250);
    }

    #[test]
    fn test_token_estimation_cap() {
        // Should cap at u16::MAX
        let huge = estimate_tokens(u32::MAX);
        assert_eq!(huge, u16::MAX);
    }

    #[test]
    fn test_section_table_empty() {
        let table = SectionTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
        assert_eq!(table.table_size(), 0);
    }

    #[test]
    fn test_section_table_push() {
        let mut table = SectionTable::new();
        table.push(SectionEntry::new(SectionType::Meta, 32, 100));
        table.push(SectionEntry::new(SectionType::TechStack, 132, 200));

        assert_eq!(table.len(), 2);
        assert_eq!(table.table_size(), 32);
    }

    #[test]
    fn test_section_table_roundtrip() {
        let mut original = SectionTable::new();
        original.push(SectionEntry::new(SectionType::Meta, 32, 100));
        original.push(SectionEntry::new(SectionType::TechStack, 132, 200));
        original.push(SectionEntry::new(SectionType::KeyFiles, 332, 500));

        let bytes = original.to_bytes().unwrap();
        assert_eq!(bytes.len(), 48); // 3 × 16 bytes

        let recovered = SectionTable::from_bytes(&bytes, 3).unwrap();
        assert_eq!(recovered.len(), 3);

        for (orig, recv) in original.entries().iter().zip(recovered.entries().iter()) {
            assert_eq!(orig.section_type, recv.section_type);
            assert_eq!(orig.offset, recv.offset);
            assert_eq!(orig.length, recv.length);
        }
    }

    #[test]
    fn test_section_table_get_by_type() {
        let mut table = SectionTable::new();
        table.push(SectionEntry::new(SectionType::Meta, 32, 100));
        table.push(SectionEntry::new(SectionType::TechStack, 132, 200));

        let meta = table.get_by_type(SectionType::Meta);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().offset, 32);

        let missing = table.get_by_type(SectionType::KeyFiles);
        assert!(missing.is_none());
    }

    #[test]
    fn test_section_table_priority_sorting() {
        let mut table = SectionTable::new();
        table.push(SectionEntry::new(SectionType::Context, 0, 100).with_priority(Priority::low()));
        table
            .push(SectionEntry::new(SectionType::Meta, 0, 100).with_priority(Priority::critical()));
        table.push(
            SectionEntry::new(SectionType::TechStack, 0, 100).with_priority(Priority::high()),
        );

        let sorted = table.entries_by_priority();
        assert_eq!(sorted[0].section_type, SectionType::Meta); // Critical first
        assert_eq!(sorted[1].section_type, SectionType::TechStack); // High second
        assert_eq!(sorted[2].section_type, SectionType::Context); // Low last
    }

    #[test]
    fn test_section_table_budget() {
        let mut table = SectionTable::new();
        table.push(
            SectionEntry::new(SectionType::Meta, 0, 100)
                .with_priority(Priority::critical())
                .with_token_count(50),
        );
        table.push(
            SectionEntry::new(SectionType::TechStack, 0, 200)
                .with_priority(Priority::high())
                .with_token_count(100),
        );
        table.push(
            SectionEntry::new(SectionType::Context, 0, 1000)
                .with_priority(Priority::low())
                .with_token_count(500),
        );

        // Budget of 200 should include Meta (50) and TechStack (100)
        let within_budget = table.entries_within_budget(200);
        assert_eq!(within_budget.len(), 2);

        // Meta should always be included (critical)
        assert!(
            within_budget
                .iter()
                .any(|e| e.section_type == SectionType::Meta)
        );
    }

    #[test]
    fn test_section_table_total_tokens() {
        let mut table = SectionTable::new();
        table.push(SectionEntry::new(SectionType::Meta, 0, 100).with_token_count(50));
        table.push(SectionEntry::new(SectionType::TechStack, 0, 200).with_token_count(100));

        assert_eq!(table.total_tokens(), 150);
    }

    #[test]
    fn test_section_entry_validate_bounds() {
        let entry = SectionEntry::new(SectionType::Meta, 100, 50);

        // Valid: offset 100, length 50, file size 200
        assert!(entry.validate_bounds(200).is_ok());

        // Invalid: offset 100, length 50 = end 150, but file only 100
        assert!(entry.validate_bounds(100).is_err());
    }

    #[test]
    fn test_unknown_section_type_preserved() {
        let entry = SectionEntry {
            section_type: SectionType::Unknown(0x99),
            priority: Priority::medium(),
            offset: 0,
            length: 100,
            token_count: 25,
            flags: 0,
        };

        let bytes = entry.to_bytes().unwrap();
        let recovered = SectionEntry::from_bytes(&bytes).unwrap();

        assert!(matches!(recovered.section_type, SectionType::Unknown(0x99)));
    }
}
