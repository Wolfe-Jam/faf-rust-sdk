//! FAF file discovery - find project.faf in directory tree

use std::env;
use std::path::{Path, PathBuf};

/// Maximum directories to traverse upward
const MAX_DEPTH: usize = 10;

/// FAF file names to search for (in priority order)
const FAF_FILES: &[&str] = &["project.faf", ".faf"];

/// Find FAF file starting from given directory, walking up to parents
///
/// Searches for `project.faf` (modern) or `.faf` (legacy) in the starting
/// directory and up to 10 parent directories.
///
/// # Example
///
/// ```rust,no_run
/// use faf_rust_sdk::find_faf_file;
/// use std::path::PathBuf;
///
/// // Search from current directory
/// if let Some(path) = find_faf_file::<PathBuf>(None) {
///     println!("Found FAF at: {}", path.display());
/// }
///
/// // Search from specific directory
/// if let Some(path) = find_faf_file(Some("/path/to/project")) {
///     println!("Found FAF at: {}", path.display());
/// }
/// ```
pub fn find_faf_file<P: AsRef<Path>>(start_dir: Option<P>) -> Option<PathBuf> {
    let start = match start_dir {
        Some(p) => p.as_ref().to_path_buf(),
        None => env::current_dir().ok()?,
    };

    let mut current = start.as_path();
    let mut depth = 0;

    while depth < MAX_DEPTH {
        // Check for FAF files in priority order
        for &filename in FAF_FILES {
            let candidate = current.join(filename);
            if candidate.is_file() {
                return Some(candidate);
            }
        }

        // Move to parent directory
        match current.parent() {
            Some(parent) if parent != current => {
                current = parent;
                depth += 1;
            }
            _ => break,
        }
    }

    None
}

/// Find and parse FAF file in one call
///
/// Convenience function that combines `find_faf_file` and `parse_file`.
///
/// # Example
///
/// ```rust,no_run
/// use faf_rust_sdk::find_and_parse;
/// use std::path::PathBuf;
///
/// match find_and_parse::<PathBuf>(None) {
///     Ok(faf) => println!("Project: {}", faf.project_name()),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn find_and_parse<P: AsRef<Path>>(
    start_dir: Option<P>,
) -> Result<crate::parser::FafFile, FindError> {
    let path = find_faf_file(start_dir).ok_or(FindError::NotFound)?;
    crate::parser::parse_file(&path).map_err(FindError::ParseError)
}

/// Errors from find operations
#[derive(Debug)]
pub enum FindError {
    /// No FAF file found in directory tree
    NotFound,
    /// FAF file found but failed to parse
    ParseError(crate::parser::FafError),
}

impl std::fmt::Display for FindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindError::NotFound => write!(f, "No FAF file found in directory tree"),
            FindError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for FindError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_in_current_dir() {
        let dir = TempDir::new().unwrap();
        let faf_path = dir.path().join("project.faf");
        fs::write(&faf_path, "faf_version: 2.5.0\nproject:\n  name: test").unwrap();

        let found = find_faf_file(Some(dir.path()));
        assert!(found.is_some());
        assert_eq!(found.unwrap(), faf_path);
    }

    #[test]
    fn test_find_in_parent() {
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
    fn test_find_legacy_faf() {
        let dir = TempDir::new().unwrap();
        let faf_path = dir.path().join(".faf");
        fs::write(&faf_path, "faf_version: 2.5.0\nproject:\n  name: test").unwrap();

        let found = find_faf_file(Some(dir.path()));
        assert!(found.is_some());
        assert_eq!(found.unwrap(), faf_path);
    }

    #[test]
    fn test_modern_takes_priority() {
        let dir = TempDir::new().unwrap();

        // Create both files
        let modern = dir.path().join("project.faf");
        let legacy = dir.path().join(".faf");
        fs::write(&modern, "faf_version: 2.5.0\nproject:\n  name: modern").unwrap();
        fs::write(&legacy, "faf_version: 2.5.0\nproject:\n  name: legacy").unwrap();

        let found = find_faf_file(Some(dir.path()));
        assert!(found.is_some());
        // Modern should be found first
        assert_eq!(found.unwrap(), modern);
    }

    #[test]
    fn test_not_found() {
        let dir = TempDir::new().unwrap();
        let found = find_faf_file(Some(dir.path()));
        assert!(found.is_none());
    }

    #[test]
    fn test_find_and_parse() {
        let dir = TempDir::new().unwrap();
        let faf_path = dir.path().join("project.faf");
        fs::write(&faf_path, "faf_version: 2.5.0\nproject:\n  name: parsed-test").unwrap();

        let result = find_and_parse(Some(dir.path()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().project_name(), "parsed-test");
    }

    #[test]
    fn test_find_and_parse_not_found() {
        let dir = TempDir::new().unwrap();
        let result = find_and_parse(Some(dir.path()));
        assert!(matches!(result, Err(FindError::NotFound)));
    }

    #[test]
    fn test_depth_limit() {
        let base = TempDir::new().unwrap();

        // Create deeply nested directory (deeper than MAX_DEPTH)
        let mut deep = base.path().to_path_buf();
        for i in 0..15 {
            deep = deep.join(format!("level{}", i));
        }
        fs::create_dir_all(&deep).unwrap();

        // Put FAF at base
        let faf_path = base.path().join("project.faf");
        fs::write(&faf_path, "faf_version: 2.5.0\nproject:\n  name: test").unwrap();

        // Should NOT find it (too deep)
        let found = find_faf_file(Some(&deep));
        assert!(found.is_none());
    }
}
