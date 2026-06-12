//! Axum integration — FAF project context as middleware
//!
//! Add project DNA to every request with one line:
//!
//! ```rust,no_run
//! use axum::{Router, routing::get};
//! use faf_rust_sdk::axum::{FafLayer, FafContext};
//!
//! let app: Router = Router::new()
//!     .route("/", get(handler))
//!     .layer(FafLayer::new());
//!
//! async fn handler(faf: FafContext) -> String {
//!     format!("Project: {}", faf.project_name())
//! }
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::task::{Context, Poll};

use ::axum::extract::FromRequestParts;
use ::axum::response::IntoResponse;
use ::http::StatusCode;
use ::http::request::Parts;
use ::tower::{Layer, Service};

use crate::compress::{CompressionLevel, compress};
use crate::discovery::{FindError, find_and_parse};
use crate::parser::FafFile;
use crate::types::FafData;
use crate::validator::{ValidationResult, validate};

// ---------------------------------------------------------------------------
// FafContext — the extractor
// ---------------------------------------------------------------------------

/// FAF project context, extracted from requests.
///
/// Wraps an `Arc` — cloning is a single atomic increment, zero allocation.
#[derive(Clone, Debug)]
pub struct FafContext(Arc<FafContextInner>);

#[derive(Debug)]
struct FafContextInner {
    file: FafFile,
    validation: ValidationResult,
    compressed: Option<FafData>,
}

impl FafContext {
    /// Project name from `.faf`
    #[inline]
    pub fn project_name(&self) -> &str {
        self.0.file.project_name()
    }

    /// AI-readiness score (0–100)
    pub fn score(&self) -> Option<u8> {
        self.0.file.score()
    }

    /// FAF version string
    #[inline]
    pub fn version(&self) -> &str {
        self.0.file.version()
    }

    /// Tech stack
    pub fn tech_stack(&self) -> Option<&str> {
        self.0.file.tech_stack()
    }

    /// Project goal
    pub fn goal(&self) -> Option<&str> {
        self.0.file.goal()
    }

    /// Full parsed data
    pub fn data(&self) -> &FafData {
        &self.0.file.data
    }

    /// Validation result computed at startup
    pub fn validation(&self) -> &ValidationResult {
        &self.0.validation
    }

    /// Compressed data (if compression was configured)
    pub fn compressed(&self) -> Option<&FafData> {
        self.0.compressed.as_ref()
    }

    /// Access the underlying `FafFile`
    pub fn file(&self) -> &FafFile {
        &self.0.file
    }

    /// Returns `true` when two `FafContext` values point to the same Arc
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<S> FromRequestParts<S> for FafContext
where
    S: Send + Sync,
{
    type Rejection = FafContextRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<FafContext>()
            .cloned()
            .ok_or(FafContextRejection)
    }
}

// ---------------------------------------------------------------------------
// Rejection
// ---------------------------------------------------------------------------

/// Returned when `FafLayer` is not installed on the router.
#[derive(Debug)]
pub struct FafContextRejection;

impl IntoResponse for FafContextRejection {
    fn into_response(self) -> ::axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FafLayer not installed — add .layer(FafLayer::new()) to your Router",
        )
            .into_response()
    }
}

impl std::fmt::Display for FafContextRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FafLayer not installed")
    }
}

// ---------------------------------------------------------------------------
// FafLayer — Tower Layer
// ---------------------------------------------------------------------------

/// Tower layer that injects [`FafContext`] into every request.
///
/// Parses the `.faf` file **once** at startup; per-request cost is a single
/// `Arc::clone`.
#[derive(Clone, Debug)]
pub struct FafLayer {
    context: FafContext,
}

impl Default for FafLayer {
    /// Equivalent to [`FafLayer::new()`].
    fn default() -> Self {
        Self::new()
    }
}

impl FafLayer {
    /// Discover and parse `.faf` from the current directory (walks parents).
    ///
    /// # Panics
    ///
    /// Panics if no `.faf` file is found. Use [`FafLayer::builder`] with
    /// [`FafLayerBuilder::try_build`] for graceful error handling.
    pub fn new() -> Self {
        Self::builder()
            .try_build()
            .expect("FafLayer::new() — no .faf file found. Use FafLayer::builder().dir(...).try_build() for graceful handling.")
    }

    /// Start building with options.
    pub fn builder() -> FafLayerBuilder {
        FafLayerBuilder {
            dir: None,
            compression: None,
            validate: true,
        }
    }

    /// Build from an already-parsed `FafFile`.
    pub fn from_file(file: FafFile) -> Self {
        let validation = validate(&file);
        Self {
            context: FafContext(Arc::new(FafContextInner {
                file,
                validation,
                compressed: None,
            })),
        }
    }

    /// Get a reference to the shared context.
    pub fn context(&self) -> &FafContext {
        &self.context
    }
}

impl<S> Layer<S> for FafLayer {
    type Service = FafService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FafService {
            inner,
            context: self.context.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// FafLayerBuilder
// ---------------------------------------------------------------------------

/// Builder for [`FafLayer`] with options for directory, compression, and
/// validation.
pub struct FafLayerBuilder {
    dir: Option<PathBuf>,
    compression: Option<CompressionLevel>,
    validate: bool,
}

impl FafLayerBuilder {
    /// Set the directory to search for `.faf` (default: cwd).
    pub fn dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Pre-compute a compressed version at startup.
    pub fn compression(mut self, level: CompressionLevel) -> Self {
        self.compression = Some(level);
        self
    }

    /// Whether to validate at startup (default: `true`).
    pub fn validate(mut self, yes: bool) -> Self {
        self.validate = yes;
        self
    }

    /// Build, returning an error if no `.faf` is found or it fails to parse.
    pub fn try_build(self) -> Result<FafLayer, FindError> {
        let file = match &self.dir {
            Some(d) => find_and_parse(Some(d))?,
            None => find_and_parse::<PathBuf>(None)?,
        };

        let validation = if self.validate {
            validate(&file)
        } else {
            ValidationResult {
                valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                score: 0,
            }
        };

        let compressed = self.compression.map(|level| compress(&file, level));

        Ok(FafLayer {
            context: FafContext(Arc::new(FafContextInner {
                file,
                validation,
                compressed,
            })),
        })
    }

    /// Build, panicking on failure. Prefer [`try_build`](Self::try_build).
    pub fn build(self) -> FafLayer {
        self.try_build().expect("FafLayerBuilder::build() failed")
    }
}

// ---------------------------------------------------------------------------
// FafService — Tower Service (internal)
// ---------------------------------------------------------------------------

/// Middleware service that injects [`FafContext`] as a request extension.
#[derive(Clone, Debug)]
pub struct FafService<S> {
    inner: S,
    context: FafContext,
}

impl<S, B> Service<::http::Request<B>> for FafService<S>
where
    S: Service<::http::Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: ::http::Request<B>) -> Self::Future {
        req.extensions_mut().insert(self.context.clone());
        self.inner.call(req)
    }
}
