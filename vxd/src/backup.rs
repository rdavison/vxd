//! Backup file handling.
//!
//! This module models simple backup file path generation.

use crate::types::VimError;

/// Generate a backup file path by appending a suffix.
pub fn backup_path(path: &str, suffix: &str) -> Result<String, VimError> {
    if path.trim().is_empty() {
        return Err(VimError::Error(1, "Empty path".to_string()));
    }
    Ok(format!("{}{}", path, suffix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_path_appends_suffix() {
        assert_eq!(backup_path("file.txt", "~").unwrap(), "file.txt~");
    }

    #[test]
    fn test_backup_path_rejects_empty() {
        let err = backup_path("", "~").unwrap_err();
        assert_eq!(err, VimError::Error(1, "Empty path".to_string()));
    }
}
