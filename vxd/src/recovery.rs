//! Crash recovery helpers.
//!
//! This module models simple recovery metadata and discovery.

use crate::types::VimError;

/// Metadata describing a recovery artifact (e.g., swap file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryArtifact {
    /// Path to the artifact.
    pub path: String,
    /// Associated original file path.
    pub original: String,
}

/// Derive a swap file path for a given file.
pub fn swap_path(path: &str) -> Result<String, VimError> {
    if path.trim().is_empty() {
        return Err(VimError::Error(1, "Empty path".to_string()));
    }
    Ok(format!("{}.swp", path))
}

/// Parse a swap path back into its original file path.
pub fn original_from_swap(path: &str) -> Option<String> {
    if let Some(stripped) = path.strip_suffix(".swp") {
        if stripped.is_empty() {
            return None;
        }
        Some(stripped.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_path_appends_suffix() {
        assert_eq!(swap_path("file.txt").unwrap(), "file.txt.swp");
    }

    #[test]
    fn test_swap_path_rejects_empty() {
        let err = swap_path("").unwrap_err();
        assert_eq!(err, VimError::Error(1, "Empty path".to_string()));
    }

    #[test]
    fn test_original_from_swap() {
        assert_eq!(original_from_swap("file.txt.swp"), Some("file.txt".to_string()));
        assert_eq!(original_from_swap("file.txt"), None);
    }
}
