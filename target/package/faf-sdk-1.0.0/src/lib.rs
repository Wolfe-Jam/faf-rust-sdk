//! FAF Rust SDK - Foundational AI-context Format
//!
//! Fast, zero-copy parser for FAF files optimized for inference workloads.
//!
//! # Example
//!
//! ```rust
//! use faf_sdk::{parse, FafFile};
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

mod parser;
mod types;
mod validator;
mod compress;

pub use parser::{parse, parse_file, FafFile, FafError};
pub use types::*;
pub use validator::{validate, ValidationResult};
pub use compress::{compress, CompressionLevel};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
