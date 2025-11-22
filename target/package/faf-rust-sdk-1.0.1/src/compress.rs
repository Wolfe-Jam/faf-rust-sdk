//! FAF compression for token optimization

use crate::types::*;
use crate::parser::FafFile;

/// Compression levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// Minimal: ~150 tokens
    Minimal = 1,
    /// Standard: ~400 tokens
    Standard = 2,
    /// Full: ~800 tokens
    Full = 3,
}

/// Compress FAF to specified level
///
/// # Example
///
/// ```rust
/// use faf_sdk::{parse, compress, CompressionLevel};
///
/// let content = r#"
/// faf_version: 2.5.0
/// project:
///   name: test
///   goal: Testing
/// instant_context:
///   tech_stack: Rust
///   what_building: Test app
///   key_files:
///     - main.rs
/// stack:
///   backend: Rust
/// human_context:
///   who: Devs
/// "#;
///
/// let faf = parse(content).unwrap();
/// let compressed = compress(&faf, CompressionLevel::Minimal);
/// // Minimal only keeps project + tech_stack
/// ```
pub fn compress(faf: &FafFile, level: CompressionLevel) -> FafData {
    match level {
        CompressionLevel::Minimal => compress_minimal(faf),
        CompressionLevel::Standard => compress_standard(faf),
        CompressionLevel::Full => faf.data.clone(),
    }
}

fn compress_minimal(faf: &FafFile) -> FafData {
    FafData {
        faf_version: faf.data.faf_version.clone(),
        project: Project {
            name: faf.data.project.name.clone(),
            goal: faf.data.project.goal.clone(),
            main_language: None,
            approach: None,
            version: None,
            license: None,
        },
        ai_score: None,
        ai_confidence: None,
        ai_tldr: None,
        instant_context: faf.data.instant_context.as_ref().map(|ic| InstantContext {
            what_building: None,
            tech_stack: ic.tech_stack.clone(),
            deployment: None,
            key_files: Vec::new(),
            commands: Default::default(),
        }),
        context_quality: None,
        stack: None,
        human_context: None,
        preferences: None,
        state: None,
        tags: Vec::new(),
    }
}

fn compress_standard(faf: &FafFile) -> FafData {
    FafData {
        faf_version: faf.data.faf_version.clone(),
        project: faf.data.project.clone(),
        ai_score: faf.data.ai_score.clone(),
        ai_confidence: None,
        ai_tldr: None,
        instant_context: faf.data.instant_context.as_ref().map(|ic| InstantContext {
            what_building: ic.what_building.clone(),
            tech_stack: ic.tech_stack.clone(),
            deployment: None,
            key_files: ic.key_files.iter().take(5).cloned().collect(),
            commands: Default::default(),
        }),
        context_quality: None,
        stack: faf.data.stack.clone(),
        human_context: None,
        preferences: None,
        state: None,
        tags: Vec::new(),
    }
}

/// Get estimated token count for compression level
pub fn estimate_tokens(level: CompressionLevel) -> usize {
    match level {
        CompressionLevel::Minimal => 150,
        CompressionLevel::Standard => 400,
        CompressionLevel::Full => 800,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_compress_minimal() {
        let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: Testing
instant_context:
  what_building: App
  tech_stack: Rust
  key_files:
    - a.rs
    - b.rs
stack:
  backend: Rust
human_context:
  who: Devs
"#;
        let faf = parse(content).unwrap();
        let compressed = compress(&faf, CompressionLevel::Minimal);

        assert_eq!(compressed.project.name, "test");
        assert!(compressed.instant_context.as_ref().unwrap().tech_stack.is_some());
        assert!(compressed.stack.is_none());
        assert!(compressed.human_context.is_none());
    }

    #[test]
    fn test_compress_standard() {
        let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: Testing
instant_context:
  what_building: App
  tech_stack: Rust
  key_files:
    - a.rs
    - b.rs
    - c.rs
    - d.rs
    - e.rs
    - f.rs
    - g.rs
stack:
  backend: Rust
human_context:
  who: Devs
"#;
        let faf = parse(content).unwrap();
        let compressed = compress(&faf, CompressionLevel::Standard);

        assert!(compressed.stack.is_some());
        // Key files limited to 5
        assert_eq!(compressed.instant_context.as_ref().unwrap().key_files.len(), 5);
        // Human context still excluded
        assert!(compressed.human_context.is_none());
    }
}
