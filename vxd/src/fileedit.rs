//! Multi-file editing helpers (argument list navigation).
//!
//! This module models basic file navigation across an argument list.

use crate::types::VimResult;

/// Trait for multi-file editing/navigation.
pub trait FileEditor {
    /// Edit (open) a file, adding it to the argument list if needed.
    fn edit(&mut self, name: &str) -> VimResult<()>;

    /// Get the current file name.
    fn current_file(&self) -> Option<&str>;

    /// Get the argument list (file names).
    fn arglist(&self) -> &[String];

    /// Move to the next file in the argument list.
    fn next_file(&mut self) -> VimResult<()>;

    /// Move to the previous file in the argument list.
    fn prev_file(&mut self) -> VimResult<()>;
}
