//! FAFB Binary Format — Unified Specification
//!
//! Compiles human-readable .faf (YAML) to AI-optimized binary.
//! String table for unlimited section names, classification bits for DNA/Context/Pointer.
//!
//! ## Features
//!
//! - **O(1) section lookup** - Section table at end for instant access
//! - **Priority truncation** - Smart context window management
//! - **Pre-computed tokens** - No runtime estimation
//! - **String table** - Unlimited section names (up to 256)
//! - **Classification** - DNA / Context / Pointer chunk types
//!
//! ## Usage
//!
//! ```ignore
//! use faf_rust_sdk::binary::{compile, decompile, CompileOptions};
//!
//! let yaml = "faf_version: 2.5.0\nproject:\n  name: my-project\n";
//! let opts = CompileOptions { use_timestamp: false };
//! let bytes = compile(yaml, &opts).unwrap();
//! let result = decompile(&bytes).unwrap();
//! let name = result.get_section_string_by_name("project").unwrap();
//! ```
//!
//! See FAFB-SPEC-UNIFIED.md for full specification.

pub mod chunk_registry;
pub mod compile;
pub mod error;
pub mod flags;
pub mod header;
pub mod priority;
pub mod section;
pub mod section_type;
pub mod string_table;

// Re-exports for convenience
pub use chunk_registry::{
    CLASSIFICATION_MASK, ChunkClassification, DNA_KEYS, POINTER_KEY, classify_key,
};
pub use compile::{CompileOptions, DecompiledFafb, compile, decompile};
pub use error::{FafbError, FafbResult};
pub use flags::{
    FLAG_COMPRESSED, FLAG_EMBEDDINGS, FLAG_MODEL_HINTS, FLAG_RESOLVED, FLAG_SIGNED,
    FLAG_STRING_TABLE, FLAG_TOKENIZED, FLAG_WEIGHTED, Flags,
};
pub use header::{
    FafbHeader, HEADER_SIZE, MAGIC, MAGIC_U32, MAX_FILE_SIZE, MAX_SECTIONS, VERSION_MAJOR,
    VERSION_MINOR,
};
pub use priority::{
    PRIORITY_CRITICAL, PRIORITY_HIGH, PRIORITY_LOW, PRIORITY_MEDIUM, PRIORITY_OPTIONAL, Priority,
};
pub use section::{SECTION_ENTRY_SIZE, SectionEntry, SectionTable};
pub use section_type::{
    SECTION_ARCHITECTURE, SECTION_BISYNC, SECTION_COMMANDS, SECTION_CONTEXT, SECTION_CUSTOM,
    SECTION_EMBEDDINGS, SECTION_KEY_FILES, SECTION_META, SECTION_MODEL_HINTS, SECTION_TECH_STACK,
    SECTION_TOKEN_MAP, SectionType,
};
pub use string_table::StringTable;
