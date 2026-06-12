//! Integration tests for faf-rust-sdk axum feature
//!
//! Run with: cargo test --features axum

#![cfg(feature = "axum")]

use std::fs;

use axum::Router;
use axum::body::Body;
use axum::routing::get;
use http::Request;
use tempfile::TempDir;
use tower::ServiceExt;

use faf_rust_sdk::axum::{FafContext, FafLayer};
use faf_rust_sdk::{CompressionLevel, parse};

const SAMPLE_FAF: &str = r#"
faf_version: 2.5.0
ai_score: "85%"
project:
  name: axum-test-app
  goal: Test Axum integration
instant_context:
  what_building: Test server
  tech_stack: Rust, Axum
  key_files:
    - src/main.rs
stack:
  backend: Rust
"#;

fn temp_faf() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("project.faf"), SAMPLE_FAF).unwrap();
    dir
}

// -----------------------------------------------------------------------
// Test 1: Layer injects context into handler
// -----------------------------------------------------------------------
#[tokio::test]
async fn layer_injects_context() {
    let dir = temp_faf();
    let layer = FafLayer::builder().dir(dir.path()).build();

    let app = Router::new()
        .route(
            "/",
            get(|faf: FafContext| async move { faf.project_name().to_string() }),
        )
        .layer(layer);

    let resp = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"axum-test-app");
}

// -----------------------------------------------------------------------
// Test 2: FafContext delegates (project_name, score, version, etc.)
// -----------------------------------------------------------------------
#[tokio::test]
async fn context_delegates() {
    let dir = temp_faf();
    let layer = FafLayer::builder().dir(dir.path()).build();
    let ctx = layer.context().clone();

    assert_eq!(ctx.project_name(), "axum-test-app");
    assert_eq!(ctx.score(), Some(85));
    assert_eq!(ctx.version(), "2.5.0");
    assert_eq!(ctx.tech_stack(), Some("Rust, Axum"));
    assert_eq!(ctx.goal(), Some("Test Axum integration"));
    assert_eq!(ctx.data().project.name, "axum-test-app");
}

// -----------------------------------------------------------------------
// Test 3: Compression available when configured
// -----------------------------------------------------------------------
#[tokio::test]
async fn compression_configured() {
    let dir = temp_faf();
    let layer = FafLayer::builder()
        .dir(dir.path())
        .compression(CompressionLevel::Minimal)
        .build();

    let ctx = layer.context().clone();
    let compressed = ctx.compressed().expect("compression should be set");
    assert_eq!(compressed.project.name, "axum-test-app");
    // Minimal strips stack
    assert!(compressed.stack.is_none());
}

// -----------------------------------------------------------------------
// Test 4: try_build fails gracefully with no .faf
// -----------------------------------------------------------------------
#[tokio::test]
async fn try_build_fails_gracefully() {
    let empty_dir = TempDir::new().unwrap();
    let result = FafLayer::builder().dir(empty_dir.path()).try_build();
    assert!(result.is_err());
}

// -----------------------------------------------------------------------
// Test 5: Rejection returns 500 without layer
// -----------------------------------------------------------------------
#[tokio::test]
async fn rejection_without_layer() {
    let app = Router::new().route(
        "/",
        get(|faf: FafContext| async move { faf.project_name().to_string() }),
    );

    let resp = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
}

// -----------------------------------------------------------------------
// Test 6: Validation accessible
// -----------------------------------------------------------------------
#[tokio::test]
async fn validation_accessible() {
    let dir = temp_faf();
    let layer = FafLayer::builder().dir(dir.path()).build();
    let ctx = layer.context().clone();

    let v = ctx.validation();
    assert!(v.valid);
    assert!(v.score > 0);
}

// -----------------------------------------------------------------------
// Test 7: Arc clone is cheap (pointer equality)
// -----------------------------------------------------------------------
#[tokio::test]
async fn arc_clone_is_cheap() {
    let dir = temp_faf();
    let layer = FafLayer::builder().dir(dir.path()).build();

    let a = layer.context().clone();
    let b = a.clone();
    assert!(a.ptr_eq(&b));
}

// -----------------------------------------------------------------------
// Test 8: from_file builds layer from pre-parsed FafFile
// -----------------------------------------------------------------------
#[tokio::test]
async fn from_file_works() {
    let faf = parse(SAMPLE_FAF).unwrap();
    let layer = FafLayer::from_file(faf);

    let app = Router::new()
        .route(
            "/",
            get(|faf: FafContext| async move { faf.project_name().to_string() }),
        )
        .layer(layer);

    let resp = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"axum-test-app");
}
