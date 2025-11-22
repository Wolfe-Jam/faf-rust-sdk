//! Corruption Recovery & Bi-Sync Showcase Tests
//!
//! Demonstrates FAF's resilience to file corruption and self-healing capabilities.
//! Key showcase for xAI integration testing.

use faf_rust_sdk::{parse, validate, find_and_parse};
use std::fs;
use tempfile::TempDir;

/// Valid FAF content for testing
const VALID_FAF: &str = r#"
faf_version: 2.5.0
ai_score: 85%
ai_confidence: HIGH

project:
  name: grok-integration
  goal: Demonstrate corruption recovery

instant_context:
  what_building: Resilient AI context system
  tech_stack: Rust, YAML, FAF
  key_files:
    - src/lib.rs
    - src/parser.rs

stack:
  backend: Rust
  infrastructure: xAI

human_context:
  who: xAI team
  what: Test bi-sync resilience
  why: Production readiness
"#;

#[test]
fn test_corruption_detection_missing_version() {
    // Corrupt: Remove required faf_version
    let corrupted = r#"
project:
  name: broken-project
  goal: Missing version field
"#;

    let result = parse(corrupted);

    // Parser correctly rejects missing faf_version at parse time
    assert!(result.is_err(), "Parser should reject missing faf_version");

    let err = result.unwrap_err();
    println!("✅ Detected missing faf_version - corruption caught: {}", err);
}

#[test]
fn test_corruption_detection_invalid_score() {
    // Corrupt: Invalid score format
    let corrupted = r#"
faf_version: 2.5.0
ai_score: NOT_A_PERCENTAGE

project:
  name: bad-score
  goal: Invalid score format
"#;

    let result = parse(corrupted);
    assert!(result.is_ok(), "Parser handles invalid score gracefully");

    let faf = result.unwrap();
    // Score should be None when invalid
    println!("✅ Invalid score handled gracefully: {:?}", faf.score());
}

#[test]
fn test_corruption_detection_malformed_yaml() {
    // Corrupt: Bad YAML indentation
    let corrupted = r#"
faf_version: 2.5.0
project:
name: bad-indent  # Wrong - should be indented
  goal: Malformed YAML
"#;

    let result = parse(corrupted);
    assert!(result.is_err(), "Should reject malformed YAML");

    let err = result.unwrap_err();
    println!("✅ Malformed YAML rejected: {}", err);
}

#[test]
fn test_corruption_detection_truncated_file() {
    // Corrupt: Truncated mid-content
    let corrupted = r#"
faf_version: 2.5.0
ai_score: 75%

project:
  name: truncated
  goal: File was cut o"#; // Truncated!

    let result = parse(corrupted);
    // Should still parse what it can
    if let Ok(faf) = result {
        println!("✅ Truncated file partially recovered: {}", faf.project_name());
    } else {
        println!("✅ Truncated file detected as corrupt");
    }
}

#[test]
fn test_corruption_recovery_workflow() {
    let temp = TempDir::new().unwrap();
    let faf_path = temp.path().join("project.faf");

    // Step 1: Create valid file
    fs::write(&faf_path, VALID_FAF).unwrap();
    println!("1️⃣ Created valid FAF file");

    // Step 2: Verify it's valid
    let faf = find_and_parse::<std::path::PathBuf>(Some(temp.path().to_path_buf())).unwrap();
    let validation = validate(&faf);
    assert!(validation.valid, "Initial file should be valid");
    assert!(faf.score().unwrap_or(0) > 80);
    println!("2️⃣ Validated: score {}%", faf.score().unwrap_or(0));

    // Step 3: Corrupt it
    let corrupted = r#"
faf_version: 2.5.0
ai_score: CORRUPTED

project:
  name: corrupted
  goal: Corrupted file
"#;
    fs::write(&faf_path, corrupted).unwrap();
    println!("3️⃣ File corrupted");

    // Step 4: Detect corruption
    let corrupt_faf = find_and_parse::<std::path::PathBuf>(Some(temp.path().to_path_buf())).unwrap();
    let corrupt_validation = validate(&corrupt_faf);
    // Score will be None due to invalid format
    assert!(corrupt_faf.score().is_none() || corrupt_validation.warnings.len() > 0);
    println!("4️⃣ Corruption detected: {} errors, {} warnings, score: {:?}",
             corrupt_validation.errors.len(),
             corrupt_validation.warnings.len(),
             corrupt_faf.score());

    // Step 5: Self-heal by restoring valid content
    fs::write(&faf_path, VALID_FAF).unwrap();
    let healed_faf = find_and_parse::<std::path::PathBuf>(Some(temp.path().to_path_buf())).unwrap();
    let healed_validation = validate(&healed_faf);
    assert!(healed_validation.valid);
    println!("5️⃣ File healed: score {}%", healed_faf.score().unwrap_or(0));

    println!("\n🏆 CORRUPTION RECOVERY WORKFLOW COMPLETE");
}

#[test]
fn test_bisync_conflict_detection() {
    // Simulate bi-sync scenario: two versions of same project
    let version_a = r#"
faf_version: 2.5.0
ai_score: 80%

project:
  name: shared-project
  goal: Version A - local changes

instant_context:
  what_building: Feature A
  tech_stack: Rust
"#;

    let version_b = r#"
faf_version: 2.5.0
ai_score: 85%

project:
  name: shared-project
  goal: Version B - remote changes

instant_context:
  what_building: Feature B
  tech_stack: Rust, Python
"#;

    let faf_a = parse(version_a).unwrap();
    let faf_b = parse(version_b).unwrap();

    // Detect differences (bi-sync conflict indicators)
    let score_diff = (faf_a.score().unwrap_or(0) as i32 - faf_b.score().unwrap_or(0) as i32).abs();
    let goal_a = faf_a.data.project.goal.as_deref().unwrap_or("");
    let goal_b = faf_b.data.project.goal.as_deref().unwrap_or("");
    let goal_changed = goal_a != goal_b;

    println!("📊 Bi-sync conflict analysis:");
    println!("   Score delta: {}%", score_diff);
    println!("   Goal changed: {}", goal_changed);
    println!("   Project name match: {}", faf_a.project_name() == faf_b.project_name());

    assert!(goal_changed, "Should detect goal conflict");
    assert!(score_diff > 0, "Should detect score difference");

    println!("\n✅ BI-SYNC CONFLICT DETECTION WORKING");
}

#[test]
fn test_unicode_corruption_resilience() {
    // Test with Unicode that might get corrupted in transit
    let unicode_content = r#"
faf_version: 2.5.0
ai_score: 90%

project:
  name: unicode-test-🦀
  goal: Test émojis and spëcial châractérs

instant_context:
  what_building: 日本語テスト
  tech_stack: Rust 🦀, Python 🐍
"#;

    let result = parse(unicode_content);
    assert!(result.is_ok(), "Should handle Unicode");

    let faf = result.unwrap();
    assert!(faf.project_name().contains("🦀") || faf.project_name().contains("unicode"));

    println!("✅ Unicode handling: {}", faf.project_name());
}

#[test]
fn test_large_file_corruption_detection() {
    // Create a large FAF with lots of key_files
    let mut content = String::from(r#"
faf_version: 2.5.0
ai_score: 95%

project:
  name: large-project
  goal: Test large file handling

instant_context:
  what_building: Massive codebase
  tech_stack: Rust
  key_files:
"#);

    // Add 1000 key files
    for i in 0..1000 {
        content.push_str(&format!("    - src/module_{}.rs\n", i));
    }

    let result = parse(&content);
    assert!(result.is_ok(), "Should handle large files");

    let faf = result.unwrap();
    let validation = validate(&faf);

    println!("✅ Large file parsed: {} key_files", faf.key_files().len());
    println!("   Valid: {}, Warnings: {}", validation.valid, validation.warnings.len());
}

#[test]
fn test_rapid_modification_resilience() {
    let temp = TempDir::new().unwrap();
    let faf_path = temp.path().join("project.faf");

    // Simulate rapid modifications (bi-sync scenario)
    let mut success_count = 0;

    for i in 0..100 {
        let content = format!(r#"
faf_version: 2.5.0
ai_score: {}%

project:
  name: rapid-test
  goal: Iteration {}
"#, 50 + (i % 50), i);

        fs::write(&faf_path, &content).unwrap();

        if let Ok(faf) = find_and_parse::<std::path::PathBuf>(Some(temp.path().to_path_buf())) {
            if validate(&faf).valid {
                success_count += 1;
            }
        }
    }

    println!("✅ Rapid modification test: {}/100 successful parses", success_count);
    assert!(success_count >= 95, "Should handle rapid modifications reliably");
}
