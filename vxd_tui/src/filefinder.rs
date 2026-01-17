//! File finding implementation for the TUI.

use vxd::filefinder::{find_in_paths, FileFinder};
use vxd::types::VimResult;

/// Simple in-memory file finder.
#[derive(Debug, Default, Clone)]
pub struct TuiFileFinder {
    paths: Vec<String>,
}

impl TuiFileFinder {
    /// Create a new file finder with no paths.
    pub fn new() -> Self {
        TuiFileFinder { paths: Vec::new() }
    }
}

impl FileFinder for TuiFileFinder {
    fn set_paths(&mut self, paths: Vec<String>) -> VimResult<()> {
        self.paths = paths;
        Ok(())
    }

    fn find_files(&self, query: &str) -> Vec<String> {
        find_in_paths(&self.paths, query)
    }
}
