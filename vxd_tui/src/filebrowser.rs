//! File browser implementation for the TUI.

use vxd::filebrowser::{sort_entries, validate_dir, BrowseSort, FileBrowser, FileEntry};
use vxd::types::VimResult;

/// In-memory file browser.
#[derive(Debug, Default)]
pub struct TuiFileBrowser {
    dir: String,
    entries: Vec<FileEntry>,
}

impl TuiFileBrowser {
    /// Create a new file browser.
    pub fn new() -> Self {
        TuiFileBrowser {
            dir: ".".to_string(),
            entries: Vec::new(),
        }
    }

    /// Set entries for the current directory (testing helper).
    pub fn set_entries(&mut self, entries: Vec<FileEntry>) {
        self.entries = entries;
    }
}

impl FileBrowser for TuiFileBrowser {
    fn set_dir(&mut self, dir: &str) -> VimResult<()> {
        validate_dir(dir)?;
        self.dir = dir.to_string();
        Ok(())
    }

    fn dir(&self) -> &str {
        &self.dir
    }

    fn list(&self, sort: BrowseSort) -> Vec<FileEntry> {
        sort_entries(self.entries.clone(), sort)
    }
}
