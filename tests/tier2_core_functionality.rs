//! WJTTC Tier 2: ENGINE - Core Functionality Tests
//!
//! Core parsing, validation, compression, and discovery.
//! The engine that drives everything.

use faf_rust_sdk::{
    CompressionLevel, compress, estimate_tokens, find_and_parse, find_faf_file, parse, stringify,
    validate,
};
use std::fs;
use tempfile::TempDir;

// =============================================================================
// CORE PARSING (Tests 1-3)
// =============================================================================

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
fn test_parse_score() {
    let content = r#"
faf_version: 2.5.0
ai_score: "85%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.score(), Some(85));
}

// =============================================================================
// SCORE VALUES (Tests 4-8)
// =============================================================================

#[test]
fn test_score_0() {
    let content = r#"
faf_version: 2.5.0
ai_score: "0%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.score(), Some(0));
}

#[test]
fn test_score_85() {
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
fn test_score_100() {
    let content = r#"
faf_version: 2.5.0
ai_score: "100%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.score(), Some(100));
}

#[test]
fn test_score_no_percent() {
    let content = r#"
faf_version: 2.5.0
ai_score: "85"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.score(), Some(85));
}

#[test]
fn test_score_double_percent() {
    let content = r#"
faf_version: 2.5.0
ai_score: "85%%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    // Lenient: trim_end_matches removes all trailing %
    assert_eq!(faf.score(), Some(85));
}

// =============================================================================
// VALIDATION SCORING (Tests 9-10)
// =============================================================================

#[test]
fn test_validation_scoring_minimal() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    let result = validate(&faf);
    assert!(result.valid);
    // Only faf_version (10) + project.name (10) = 20
    assert_eq!(result.score, 20);
}

#[test]
fn test_validation_scoring_full() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: Testing
instant_context:
  what_building: App
  tech_stack: Rust
  key_files:
    - main.rs
stack:
  backend: Rust
human_context:
  who: Devs
tags:
  - rust
state:
  phase: dev
"#;
    let faf = parse(content).unwrap();
    let result = validate(&faf);
    assert!(result.valid);
    assert_eq!(result.score, 100);
}

// =============================================================================
// COMPRESSION (Tests 11-13)
// =============================================================================

#[test]
fn test_compression_minimal() {
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
    assert!(
        compressed
            .instant_context
            .as_ref()
            .unwrap()
            .tech_stack
            .is_some()
    );
    assert!(compressed.stack.is_none());
    assert!(compressed.human_context.is_none());
}

#[test]
fn test_compression_standard() {
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
    assert_eq!(
        compressed.instant_context.as_ref().unwrap().key_files.len(),
        5
    );
    assert!(compressed.human_context.is_none());
}

#[test]
fn test_compression_full() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: Testing
stack:
  backend: Rust
"#;
    let faf = parse(content).unwrap();
    let full = compress(&faf, CompressionLevel::Full);

    assert_eq!(full.project.name, faf.data.project.name);
    assert_eq!(full.project.goal, faf.data.project.goal);
    assert!(full.stack.is_some());
}

// =============================================================================
// DISCOVERY (Tests 14-17)
// =============================================================================

#[test]
fn test_discovery_current_dir() {
    let dir = TempDir::new().unwrap();
    let faf_path = dir.path().join("project.faf");
    fs::write(&faf_path, "faf_version: 2.5.0\nproject:\n  name: test").unwrap();

    let found = find_faf_file(Some(dir.path()));
    assert!(found.is_some());
    assert_eq!(found.unwrap(), faf_path);
}

#[test]
fn test_discovery_parent() {
    let parent = TempDir::new().unwrap();
    let child = parent.path().join("subdir");
    fs::create_dir(&child).unwrap();

    let faf_path = parent.path().join("project.faf");
    fs::write(&faf_path, "faf_version: 2.5.0\nproject:\n  name: test").unwrap();

    let found = find_faf_file(Some(&child));
    assert!(found.is_some());
    assert_eq!(found.unwrap(), faf_path);
}

#[test]
fn test_discovery_roundtrip() {
    let dir = TempDir::new().unwrap();
    let faf_path = dir.path().join("project.faf");
    fs::write(
        &faf_path,
        "faf_version: 2.5.0\nproject:\n  name: parsed-test",
    )
    .unwrap();

    let result = find_and_parse(Some(dir.path()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().project_name(), "parsed-test");
}

#[test]
fn test_discovery_not_found() {
    let dir = TempDir::new().unwrap();
    let found = find_faf_file(Some(dir.path()));
    assert!(found.is_none());
}

// =============================================================================
// QUALITY THRESHOLD (Tests 18-19)
// =============================================================================

#[test]
fn test_quality_threshold_at_70() {
    let content = r#"
faf_version: 2.5.0
ai_score: "70%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert!(faf.is_high_quality(), "70% should be high quality");
}

#[test]
fn test_quality_threshold_below_70() {
    let content = r#"
faf_version: 2.5.0
ai_score: "69%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert!(!faf.is_high_quality(), "69% should NOT be high quality");
}

// =============================================================================
// INVALID SCORE GRACEFUL (Test 20)
// =============================================================================

#[test]
fn test_invalid_score_graceful() {
    let content = r#"
faf_version: 2.5.0
ai_score: NOT_A_PERCENTAGE

project:
  name: bad-score
  goal: Invalid score format
"#;
    let result = parse(content);
    assert!(result.is_ok(), "Parser handles invalid score gracefully");
    let faf = result.unwrap();
    assert!(faf.score().is_none());
}

// =============================================================================
// NEW: STRINGIFY ROUND-TRIP (Test 21)
// =============================================================================

#[test]
fn test_stringify_round_trip() {
    let content = r#"
faf_version: 2.5.0
project:
  name: roundtrip-test
  goal: Verify stringify produces parseable output
"#;
    let faf = parse(content).unwrap();
    let yaml_output = stringify(&faf).unwrap();

    // Parse the stringified output - should produce same data
    let reparsed = parse(&yaml_output).unwrap();
    assert_eq!(faf.project_name(), reparsed.project_name());
    assert_eq!(faf.version(), reparsed.version());
    assert_eq!(faf.goal(), reparsed.goal());
}

// =============================================================================
// NEW: ESTIMATE TOKENS (Test 22)
// =============================================================================

#[test]
fn test_estimate_tokens_values() {
    let minimal = estimate_tokens(CompressionLevel::Minimal);
    let standard = estimate_tokens(CompressionLevel::Standard);
    let full = estimate_tokens(CompressionLevel::Full);

    assert_eq!(minimal, 150);
    assert_eq!(standard, 400);
    assert_eq!(full, 800);

    // Verify ordering: minimal < standard < full
    assert!(minimal < standard);
    assert!(standard < full);
}
