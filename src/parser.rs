//! Core FAF parser - optimized for inference workloads

use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::types::FafData;

/// FAF parsing errors
#[derive(Error, Debug)]
pub enum FafError {
    #[error("Empty content")]
    EmptyContent,

    #[error("Invalid YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Parsed FAF file with convenient accessors
#[derive(Debug, Clone)]
pub struct FafFile {
    /// Parsed and typed data
    pub data: FafData,
    /// Original file path (if loaded from file)
    pub path: Option<String>,
}

impl FafFile {
    /// Get project name
    #[inline]
    pub fn project_name(&self) -> &str {
        &self.data.project.name
    }

    /// Get AI score as integer (0-100)
    pub fn score(&self) -> Option<u8> {
        self.data.ai_score.as_ref().and_then(|s| {
            s.trim_end_matches('%')
                .parse()
                .ok()
        })
    }

    /// Get FAF version
    #[inline]
    pub fn version(&self) -> &str {
        &self.data.faf_version
    }

    /// Get tech stack string
    pub fn tech_stack(&self) -> Option<&str> {
        self.data
            .instant_context
            .as_ref()
            .and_then(|ic| ic.tech_stack.as_deref())
    }

    /// Get what building
    pub fn what_building(&self) -> Option<&str> {
        self.data
            .instant_context
            .as_ref()
            .and_then(|ic| ic.what_building.as_deref())
    }

    /// Get key files
    pub fn key_files(&self) -> &[String] {
        self.data
            .instant_context
            .as_ref()
            .map(|ic| ic.key_files.as_slice())
            .unwrap_or(&[])
    }

    /// Get project goal
    pub fn goal(&self) -> Option<&str> {
        self.data.project.goal.as_deref()
    }

    /// Check if score indicates high quality (>= 70%)
    pub fn is_high_quality(&self) -> bool {
        self.score().map(|s| s >= 70).unwrap_or(false)
    }
}

/// Parse FAF content from string
///
/// # Example
///
/// ```rust
/// use faf_rust_sdk::parse;
///
/// let content = r#"
/// faf_version: 2.5.0
/// project:
///   name: test
/// "#;
///
/// let faf = parse(content).unwrap();
/// assert_eq!(faf.project_name(), "test");
/// ```
pub fn parse(content: &str) -> Result<FafFile, FafError> {
    let content = content.trim();
    if content.is_empty() {
        return Err(FafError::EmptyContent);
    }

    let data: FafData = serde_yaml::from_str(content)?;

    Ok(FafFile { data, path: None })
}

/// Parse FAF from file path
///
/// # Example
///
/// ```rust,no_run
/// use faf_rust_sdk::parse_file;
///
/// let faf = parse_file("project.faf").unwrap();
/// println!("Project: {}", faf.project_name());
/// ```
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<FafFile, FafError> {
    let path_str = path.as_ref().to_string_lossy().to_string();
    let content = fs::read_to_string(&path)?;

    let mut faf = parse(&content)?;
    faf.path = Some(path_str);

    Ok(faf)
}

/// Serialize FAF back to YAML string
pub fn stringify(faf: &FafFile) -> Result<String, FafError> {
    Ok(serde_yaml::to_string(&faf.data)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal() {
        let content = r#"
faf_version: 2.5.0
project:
  name: test-project
"#;
        let faf = parse(content).unwrap();
        assert_eq!(faf.project_name(), "test-project");
        assert_eq!(faf.version(), "2.5.0");
    }

    #[test]
    fn test_parse_with_score() {
        let content = r#"
faf_version: 2.5.0
ai_score: "85%"
project:
  name: test
"#;
        let faf = parse(content).unwrap();
        assert_eq!(faf.score(), Some(85));
    }

    #[test]
    fn test_parse_full() {
        let content = r#"
faf_version: 2.5.0
ai_score: "90%"
project:
  name: full-test
  goal: Test everything
instant_context:
  what_building: Test app
  tech_stack: Rust, Python
  key_files:
    - src/main.rs
    - src/lib.rs
stack:
  backend: Rust
  database: PostgreSQL
"#;
        let faf = parse(content).unwrap();
        assert_eq!(faf.project_name(), "full-test");
        assert_eq!(faf.tech_stack(), Some("Rust, Python"));
        assert_eq!(faf.key_files().len(), 2);
        assert!(faf.is_high_quality());
    }

    #[test]
    fn test_empty_content() {
        let result = parse("");
        assert!(matches!(result, Err(FafError::EmptyContent)));
    }

    #[test]
    fn test_invalid_yaml() {
        let result = parse("invalid: [unclosed");
        assert!(matches!(result, Err(FafError::YamlError(_))));
    }
}
