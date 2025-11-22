//! Stress tests for FAF Rust SDK
//! F1 Philosophy: When brakes must work flawlessly, so must our code.

use faf_sdk::{parse, validate, compress, CompressionLevel};

// =============================================================================
// SCORE PARSING EDGE CASES
// =============================================================================

#[test]
fn test_score_valid_85() {
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
fn test_score_valid_0() {
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
fn test_score_valid_100() {
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
fn test_score_with_space() {
    // "85 %" - space before percent
    let content = r#"
faf_version: 2.5.0
ai_score: "85 %"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    // Current implementation will fail to parse this
    assert_eq!(faf.score(), None);
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
    // Lenient: trim_end_matches removes all trailing %, so "85%%" → "85"
    assert_eq!(faf.score(), Some(85));
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
    // No percent sign - should still parse the number
    assert_eq!(faf.score(), Some(85));
}

#[test]
fn test_score_negative() {
    let content = r#"
faf_version: 2.5.0
ai_score: "-5%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    // Negative should fail to parse to u8
    assert_eq!(faf.score(), None);
}

#[test]
fn test_score_overflow() {
    let content = r#"
faf_version: 2.5.0
ai_score: "256%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    // 256 overflows u8
    assert_eq!(faf.score(), None);
}

#[test]
fn test_score_float() {
    let content = r#"
faf_version: 2.5.0
ai_score: "85.5%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    // Float won't parse to u8
    assert_eq!(faf.score(), None);
}

#[test]
fn test_score_text() {
    let content = r#"
faf_version: 2.5.0
ai_score: "HIGH"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    // Text should return None
    assert_eq!(faf.score(), None);
}

// =============================================================================
// UNICODE AND SPECIAL CHARACTERS
// =============================================================================

#[test]
fn test_unicode_project_name() {
    let content = r#"
faf_version: 2.5.0
project:
  name: "测试プロジェクト🚀"
  goal: "Build something 大きい"
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.project_name(), "测试プロジェクト🚀");
    assert_eq!(faf.goal(), Some("Build something 大きい"));
}

#[test]
fn test_emoji_in_fields() {
    let content = r#"
faf_version: 2.5.0
project:
  name: "🏎️ F1 Project"
instant_context:
  what_building: "🚀 Rocket launcher"
  tech_stack: "Rust 🦀, Python 🐍"
"#;
    let faf = parse(content).unwrap();
    assert!(faf.project_name().contains("🏎️"));
    assert!(faf.tech_stack().unwrap().contains("🦀"));
}

#[test]
fn test_special_characters() {
    let content = r#"
faf_version: 2.5.0
project:
  name: "test<>&\"'project"
  goal: "Handle \t tabs \n newlines"
"#;
    let faf = parse(content).unwrap();
    assert!(faf.project_name().contains("<>&"));
}

#[test]
fn test_multiline_strings() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: |
    This is a multiline
    goal that spans
    multiple lines
"#;
    let faf = parse(content).unwrap();
    assert!(faf.goal().unwrap().contains("multiline"));
    assert!(faf.goal().unwrap().contains("multiple lines"));
}

// =============================================================================
// YAML EDGE CASES
// =============================================================================

#[test]
fn test_yaml_anchors() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
defaults: &defaults
  testing: required
preferences:
  <<: *defaults
  documentation: inline
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.project_name(), "test");
    // Anchors should be resolved
}

#[test]
fn test_null_values() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: null
  main_language: ~
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.goal(), None);
}

#[test]
fn test_empty_string_vs_null() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: ""
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.goal(), Some(""));
}

#[test]
fn test_empty_arrays() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
instant_context:
  key_files: []
tags: []
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.key_files().len(), 0);
}

#[test]
fn test_boolean_coercion() {
    // YAML treats yes/no, on/off, true/false as booleans
    let content = r#"
faf_version: 2.5.0
project:
  name: "yes"
  goal: "true"
"#;
    let faf = parse(content).unwrap();
    // These should stay as strings
    assert_eq!(faf.project_name(), "yes");
}

#[test]
fn test_numeric_strings() {
    let content = r#"
faf_version: "2.5.0"
project:
  name: "123"
  version: "1.0.0"
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.project_name(), "123");
}

// =============================================================================
// VALIDATION EDGE CASES
// =============================================================================

#[test]
fn test_validate_empty_version() {
    let content = r#"
faf_version: ""
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    let result = validate(&faf);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.contains("faf_version")));
}

#[test]
fn test_validate_empty_name() {
    let content = r#"
faf_version: 2.5.0
project:
  name: ""
"#;
    let faf = parse(content).unwrap();
    let result = validate(&faf);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.contains("project.name")));
}

#[test]
fn test_validate_score_calculation() {
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
    // Should max out at 100
    assert_eq!(result.score, 100);
}

#[test]
fn test_validate_minimal_score() {
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

// =============================================================================
// COMPRESSION EDGE CASES
// =============================================================================

#[test]
fn test_compress_preserves_required() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: Testing
instant_context:
  what_building: App
  tech_stack: Rust
"#;
    let faf = parse(content).unwrap();

    // Minimal should keep version, name, goal, tech_stack
    let minimal = compress(&faf, CompressionLevel::Minimal);
    assert_eq!(minimal.faf_version, "2.5.0");
    assert_eq!(minimal.project.name, "test");
    assert_eq!(minimal.project.goal, Some("Testing".to_string()));
    assert!(minimal.instant_context.as_ref().unwrap().tech_stack.is_some());

    // But should drop what_building
    assert!(minimal.instant_context.as_ref().unwrap().what_building.is_none());
}

#[test]
fn test_compress_key_files_limit() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
instant_context:
  key_files:
    - a.rs
    - b.rs
    - c.rs
    - d.rs
    - e.rs
    - f.rs
    - g.rs
    - h.rs
    - i.rs
    - j.rs
"#;
    let faf = parse(content).unwrap();
    let standard = compress(&faf, CompressionLevel::Standard);

    // Standard limits to 5 key files
    assert_eq!(standard.instant_context.as_ref().unwrap().key_files.len(), 5);
}

#[test]
fn test_compress_full_identity() {
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

    // Full should be identical
    assert_eq!(full.project.name, faf.data.project.name);
    assert_eq!(full.project.goal, faf.data.project.goal);
    assert!(full.stack.is_some());
}

// =============================================================================
// LARGE INPUT TESTS
// =============================================================================

#[test]
fn test_many_key_files() {
    let mut files = Vec::new();
    for i in 0..1000 {
        files.push(format!("    - file{}.rs", i));
    }
    let files_yaml = files.join("\n");

    let content = format!(r#"
faf_version: 2.5.0
project:
  name: test
instant_context:
  key_files:
{}
"#, files_yaml);

    let faf = parse(&content).unwrap();
    assert_eq!(faf.key_files().len(), 1000);
}

#[test]
fn test_many_tags() {
    let mut tags = Vec::new();
    for i in 0..500 {
        tags.push(format!("  - tag{}", i));
    }
    let tags_yaml = tags.join("\n");

    let content = format!(r#"
faf_version: 2.5.0
project:
  name: test
tags:
{}
"#, tags_yaml);

    let faf = parse(&content).unwrap();
    assert_eq!(faf.data.tags.len(), 500);
}

#[test]
fn test_long_strings() {
    let long_name = "x".repeat(10000);
    let content = format!(r#"
faf_version: 2.5.0
project:
  name: "{}"
"#, long_name);

    let faf = parse(&content).unwrap();
    assert_eq!(faf.project_name().len(), 10000);
}

// =============================================================================
// ERROR HANDLING
// =============================================================================

#[test]
fn test_missing_project() {
    let content = r#"
faf_version: 2.5.0
"#;
    let result = parse(content);
    assert!(result.is_err());
}

#[test]
fn test_missing_project_name() {
    let content = r#"
faf_version: 2.5.0
project:
  goal: Testing
"#;
    let result = parse(content);
    assert!(result.is_err());
}

#[test]
fn test_wrong_type_for_key_files() {
    // key_files should be array, not string
    let content = r#"
faf_version: 2.5.0
project:
  name: test
instant_context:
  key_files: "main.rs"
"#;
    let result = parse(content);
    assert!(result.is_err());
}

#[test]
fn test_wrong_type_for_tags() {
    // tags should be array, not string
    let content = r#"
faf_version: 2.5.0
project:
  name: test
tags: "rust"
"#;
    let result = parse(content);
    assert!(result.is_err());
}

#[test]
fn test_invalid_yaml_unclosed_bracket() {
    let content = r#"
faf_version: 2.5.0
project:
  name: [unclosed
"#;
    let result = parse(content);
    assert!(result.is_err());
}

#[test]
fn test_invalid_yaml_bad_indentation() {
    let content = r#"
faf_version: 2.5.0
project:
name: test
"#;
    let result = parse(content);
    assert!(result.is_err());
}

#[test]
fn test_whitespace_only() {
    let result = parse("   \n\t\n   ");
    assert!(result.is_err());
}

#[test]
fn test_comments_only() {
    let content = r#"
# Just a comment
# Another comment
"#;
    let result = parse(content);
    assert!(result.is_err());
}

// =============================================================================
// ACCESSOR METHOD EDGE CASES
// =============================================================================

#[test]
fn test_is_high_quality_boundary() {
    // 70% should be high quality
    let content = r#"
faf_version: 2.5.0
ai_score: "70%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert!(faf.is_high_quality());

    // 69% should not
    let content = r#"
faf_version: 2.5.0
ai_score: "69%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert!(!faf.is_high_quality());
}

#[test]
fn test_accessors_with_missing_sections() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
"#;
    let faf = parse(content).unwrap();

    // All these should return None/empty gracefully
    assert_eq!(faf.tech_stack(), None);
    assert_eq!(faf.what_building(), None);
    assert_eq!(faf.key_files().len(), 0);
    assert_eq!(faf.goal(), None);
    assert_eq!(faf.score(), None);
    assert!(!faf.is_high_quality());
}
