//! FAF Rust SDK - Foundational AI-context Format
//!
//! Fast, zero-copy parser for FAF files optimized for inference workloads.
//!
//! # Example
//!
//! ```rust
//! use faf_rust_sdk::{parse, FafFile};
//!
//! let content = r#"
//! faf_version: 2.5.0
//! project:
//!   name: my-project
//!   goal: Build something great
//! "#;
//!
//! let faf = parse(content).unwrap();
//! println!("Project: {}", faf.project_name());
//! ```

pub mod binary;
mod compress;
mod discovery;
mod parser;
mod types;
mod validator;

#[cfg(feature = "axum")]
pub mod axum;

pub use binary::{FafbError, FafbHeader, Flags, Priority, SectionEntry, SectionTable, SectionType};
pub use compress::{CompressionLevel, compress, estimate_tokens};
pub use discovery::{FindError, find_and_parse, find_faf_file};
pub use parser::{FafError, FafFile, parse, parse_file, stringify};
pub use types::*;
pub use validator::{ValidationResult, validate};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
