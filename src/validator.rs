//! FAF validation

use crate::parser::FafFile;

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// True if no errors
    pub valid: bool,
    /// Critical errors
    pub errors: Vec<String>,
    /// Non-critical warnings
    pub warnings: Vec<String>,
    /// Completeness score (0-100)
    pub score: u8,
}

/// Validate FAF file structure
///
/// # Example
///
/// ```rust
/// use faf_rust_sdk::{parse, validate};
///
/// let content = r#"
/// faf_version: 2.5.0
/// project:
///   name: test
/// "#;
///
/// let faf = parse(content).unwrap();
/// let result = validate(&faf);
/// assert!(result.valid);
/// ```
pub fn validate(faf: &FafFile) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Required fields
    if faf.data.faf_version.is_empty() {
        errors.push("Missing faf_version".to_string());
    }

    if faf.data.project.name.is_empty() {
        errors.push("Missing project.name".to_string());
    }

    // Recommended sections
    if faf.data.instant_context.is_none() {
        warnings.push("Missing instant_context section".to_string());
    } else {
        let ic = faf.data.instant_context.as_ref().unwrap();
        if ic.what_building.is_none() {
            warnings.push("Missing instant_context.what_building".to_string());
        }
        if ic.tech_stack.is_none() {
            warnings.push("Missing instant_context.tech_stack".to_string());
        }
    }

    if faf.data.stack.is_none() {
        warnings.push("Missing stack section".to_string());
    }

    if faf.data.human_context.is_none() {
        warnings.push("Missing human_context section".to_string());
    }

    // Calculate score
    let score = calculate_score(faf);

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        score,
    }
}

fn calculate_score(faf: &FafFile) -> u8 {
    let mut score: u8 = 0;

    // Required fields (30 points)
    if !faf.data.faf_version.is_empty() {
        score += 10;
    }
    if !faf.data.project.name.is_empty() {
        score += 10;
    }
    if faf.data.project.goal.is_some() {
        score += 10;
    }

    // Instant context (30 points)
    if let Some(ic) = &faf.data.instant_context {
        if ic.what_building.is_some() {
            score += 10;
        }
        if ic.tech_stack.is_some() {
            score += 10;
        }
        if !ic.key_files.is_empty() {
            score += 10;
        }
    }

    // Stack (15 points)
    if faf.data.stack.is_some() {
        score += 15;
    }

    // Human context (15 points)
    if faf.data.human_context.is_some() {
        score += 15;
    }

    // Extras (10 points)
    if !faf.data.tags.is_empty() {
        score += 5;
    }
    if faf.data.state.is_some() {
        score += 5;
    }

    score.min(100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_validate_minimal() {
        let content = r#"
faf_version: 2.5.0
project:
  name: test
"#;
        let faf = parse(content).unwrap();
        let result = validate(&faf);
        assert!(result.valid);
        assert!(result.score >= 20);
    }

    #[test]
    fn test_validate_full() {
        let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: Testing
instant_context:
  what_building: Test
  tech_stack: Rust
  key_files:
    - main.rs
stack:
  backend: Rust
human_context:
  who: Developers
tags:
  - rust
state:
  phase: dev
"#;
        let faf = parse(content).unwrap();
        let result = validate(&faf);
        assert!(result.valid);
        assert!(result.score >= 90);
    }
}
