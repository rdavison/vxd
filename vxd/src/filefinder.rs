//! File finding helpers.
//!
//! This module models simple file search behavior across a set of search paths.

use crate::types::VimResult;

/// Trait for file finding implementations.
pub trait FileFinder {
    /// Set the search paths.
    fn set_paths(&mut self, paths: Vec<String>) -> VimResult<()>;

    /// Find files matching a query.
    fn find_files(&self, query: &str) -> Vec<String>;
}

/// Search for files in `paths` matching `query`.
pub fn find_in_paths(paths: &[String], query: &str) -> Vec<String> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }

    let has_sep = query.contains('/') || query.contains('\\');
    let mut matches = Vec::new();

    for path in paths {
        if has_sep {
            if path.ends_with(query) || path == query {
                matches.push(path.clone());
            }
        } else if let Some(name) = path.rsplit(['/', '\\']).next() {
            if name == query || name.starts_with(query) {
                matches.push(path.clone());
            }
        }
    }

    matches.sort();
    matches.dedup();
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_in_paths_by_basename() {
        let paths = vec![
            "/src/main.rs".to_string(),
            "/src/lib.rs".to_string(),
            "/README.md".to_string(),
        ];

        let matches = find_in_paths(&paths, "lib.rs");
        assert_eq!(matches, vec!["/src/lib.rs".to_string()]);
    }

    #[test]
    fn test_find_in_paths_by_prefix() {
        let paths = vec![
            "/src/main.rs".to_string(),
            "/src/lib.rs".to_string(),
            "/src/lib_test.rs".to_string(),
        ];

        let matches = find_in_paths(&paths, "lib");
        assert_eq!(
            matches,
            vec!["/src/lib.rs".to_string(), "/src/lib_test.rs".to_string()]
        );
    }

    #[test]
    fn test_find_in_paths_with_separator() {
        let paths = vec![
            "/src/main.rs".to_string(),
            "/src/lib.rs".to_string(),
            "/docs/src/lib.rs".to_string(),
        ];

        let matches = find_in_paths(&paths, "src/lib.rs");
        assert_eq!(
            matches,
            vec!["/docs/src/lib.rs".to_string(), "/src/lib.rs".to_string()]
        );
    }
}
