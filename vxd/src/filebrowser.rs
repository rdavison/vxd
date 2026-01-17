//! File browser model.
//!
//! This module models simple directory listings and sorting.

use crate::types::{VimError, VimResult};

/// Sort order for file listings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowseSort {
    /// Sort by name.
    Name,
    /// Sort by time.
    Time,
    /// Sort by size.
    Size,
}

/// File entry metadata for browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// File name (basename).
    pub name: String,
    /// Whether entry is a directory.
    pub is_dir: bool,
    /// File size in bytes.
    pub size: u64,
    /// Timestamp (opaque integer).
    pub mtime: i64,
}

/// File browser trait.
pub trait FileBrowser {
    /// Set the current directory.
    fn set_dir(&mut self, dir: &str) -> VimResult<()>;

    /// Get the current directory.
    fn dir(&self) -> &str;

    /// List entries in the current directory.
    fn list(&self, sort: BrowseSort) -> Vec<FileEntry>;
}

/// Sort entries according to the requested ordering.
pub fn sort_entries(mut entries: Vec<FileEntry>, sort: BrowseSort) -> Vec<FileEntry> {
    entries.sort_by(|a, b| match sort {
        BrowseSort::Name => a.name.cmp(&b.name),
        BrowseSort::Time => a.mtime.cmp(&b.mtime),
        BrowseSort::Size => a.size.cmp(&b.size),
    });
    entries
}

/// Validate that a directory path is non-empty.
pub fn validate_dir(dir: &str) -> VimResult<()> {
    if dir.trim().is_empty() {
        return Err(VimError::Error(1, "Empty directory".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_entries_by_name() {
        let entries = vec![
            FileEntry {
                name: "b.txt".to_string(),
                is_dir: false,
                size: 2,
                mtime: 2,
            },
            FileEntry {
                name: "a.txt".to_string(),
                is_dir: false,
                size: 1,
                mtime: 1,
            },
        ];

        let sorted = sort_entries(entries, BrowseSort::Name);
        assert_eq!(sorted[0].name, "a.txt");
        assert_eq!(sorted[1].name, "b.txt");
    }

    #[test]
    fn test_validate_dir_rejects_empty() {
        let err = validate_dir("").unwrap_err();
        assert_eq!(err, VimError::Error(1, "Empty directory".to_string()));
    }
}
