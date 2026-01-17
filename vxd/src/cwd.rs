//! Working directory handling.
//!
//! This module models the editor's notion of the current working directory.

use crate::types::{VimError, VimResult};

/// Trait for managing the current working directory.
pub trait WorkingDirectory {
    /// Get the current working directory.
    fn getcwd(&self) -> &str;

    /// Set the current working directory.
    fn setcwd(&mut self, path: &str) -> VimResult<()>;
}

/// Validate a path for use as a working directory.
pub fn validate_cwd(path: &str) -> VimResult<()> {
    if path.trim().is_empty() {
        return Err(VimError::Error(1, "Empty path".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cwd_rejects_empty() {
        let err = validate_cwd("").unwrap_err();
        assert_eq!(err, VimError::Error(1, "Empty path".to_string()));
    }
}
