//! Working directory implementation for the TUI.

use vxd::cwd::{validate_cwd, WorkingDirectory};
use vxd::types::VimResult;

/// Simple in-memory working directory tracker.
#[derive(Debug, Clone)]
pub struct TuiWorkingDirectory {
    cwd: String,
}

impl TuiWorkingDirectory {
    /// Create a new working directory tracker.
    pub fn new() -> Self {
        TuiWorkingDirectory {
            cwd: ".".to_string(),
        }
    }
}

impl Default for TuiWorkingDirectory {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkingDirectory for TuiWorkingDirectory {
    fn getcwd(&self) -> &str {
        &self.cwd
    }

    fn setcwd(&mut self, path: &str) -> VimResult<()> {
        validate_cwd(path)?;
        self.cwd = path.to_string();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cwd() {
        let wd = TuiWorkingDirectory::new();
        assert_eq!(wd.getcwd(), ".");
    }
}
