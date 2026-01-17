//! Buffer operations and text manipulation.
//!
//! This module defines the `Buffer` trait which captures Vim's expected
//! behavior for buffer operations. Buffers are the fundamental unit of
//! text storage in Vim.
//!
//! # Key Behavioral Contracts
//!
//! - Buffers always have at least one line (even if empty: `[""]`)
//! - Line numbers are 1-indexed in Vim's user interface, but the API uses 0-indexed
//! - Negative indices are relative to the end (-1 = last line)
//! - Unloaded buffers return empty content but don't error
//! - Cursor positions are preserved across edits when possible

use crate::types::*;

// ============================================================================
// Buffer Handle Type
// ============================================================================

/// Handle to a buffer (the buffer number in Vim)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufHandle(pub usize);

impl BufHandle {
    /// The special "current buffer" handle (0 in nvim API)
    pub const CURRENT: BufHandle = BufHandle(0);
}

// ============================================================================
// Buffer Types
// ============================================================================

/// The type of a buffer, as set by 'buftype' option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BufferType {
    /// Normal buffer backed by a file
    #[default]
    Normal,
    /// Buffer without a file, will be abandoned
    Nofile,
    /// Scratch buffer (nofile + nomodified)
    Scratch,
    /// Buffer for quickfix window
    Quickfix,
    /// Buffer for help window
    Help,
    /// Buffer for terminal emulator
    Terminal,
    /// Buffer for command-line window
    Prompt,
    /// Buffer for popup window
    Popup,
    /// Acwrite - buffer triggers autocmd on write
    Acwrite,
}

/// What happens when a buffer becomes hidden
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BufHidden {
    /// Use global 'hidden' setting
    #[default]
    UseGlobal,
    /// Hide the buffer
    Hide,
    /// Unload the buffer
    Unload,
    /// Delete the buffer
    Delete,
    /// Wipe the buffer
    Wipe,
}

/// Buffer deletion mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufDeleteMode {
    /// Unlist buffer (remove from buffer list, keep content)
    Unlist,
    /// Unload buffer (remove content from memory)
    Unload,
    /// Wipe buffer (completely remove, invalid after)
    Wipe,
}

// ============================================================================
// Buffer State
// ============================================================================

/// Represents the load state of a buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferLoadState {
    /// Buffer is loaded, content in memory
    Loaded,
    /// Buffer exists but content not in memory
    Unloaded,
    /// Buffer has been wiped (invalid)
    Wiped,
}

/// Information about a buffer's state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferInfo {
    /// Buffer handle/number
    pub handle: BufHandle,
    /// Buffer name (file path or display name)
    pub name: String,
    /// Number of lines
    pub line_count: usize,
    /// Whether buffer has unsaved changes
    pub modified: bool,
    /// Whether buffer can be modified
    pub modifiable: bool,
    /// Whether buffer is read-only
    pub readonly: bool,
    /// Buffer type
    pub buftype: BufferType,
    /// Hidden behavior
    pub bufhidden: BufHidden,
    /// Load state
    pub load_state: BufferLoadState,
    /// Whether buffer is listed
    pub listed: bool,
    /// Change tick (version number, increments on each change)
    pub changedtick: u64,
}

// ============================================================================
// Buffer Change Event
// ============================================================================

/// Information about a buffer change (for watchers/events)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferChange {
    /// First line changed (0-indexed)
    pub first_line: usize,
    /// Last line changed (0-indexed, exclusive)
    pub last_line: usize,
    /// New last line after change (0-indexed, exclusive)
    pub new_last_line: usize,
    /// The new changedtick value
    pub changedtick: u64,
}

// ============================================================================
// Buffer Trait
// ============================================================================

/// The core Buffer trait defining Vim's expected buffer behaviors.
///
/// # Indexing
///
/// This trait uses 0-based indexing with support for negative indices:
/// - `0` = first line
/// - `-1` = last line
/// - `-2` = second to last line, etc.
///
/// # Strict Indexing
///
/// When `strict_indexing` is true, out-of-bounds access returns an error.
/// When false, out-of-bounds is silently handled (returns empty or clips to bounds).
///
/// # Invariants
///
/// - A buffer always has at least one line (may be empty string)
/// - Deleting all lines results in a single empty line
/// - Unloaded buffers return empty content but are not errors
pub trait Buffer {
    // ========================================================================
    // Identity & Validity
    // ========================================================================

    /// Get the buffer handle/number
    fn handle(&self) -> BufHandle;

    /// Get the buffer name (file path or display name)
    fn name(&self) -> &str;

    /// Set the buffer name
    fn set_name(&mut self, name: &str) -> VimResult<()>;

    /// Check if buffer is still valid (not wiped)
    fn is_valid(&self) -> bool;

    // ========================================================================
    // Content Access
    // ========================================================================

    /// Get the number of lines in the buffer.
    ///
    /// Returns 0 for unloaded buffers, at least 1 for loaded buffers.
    fn line_count(&self) -> usize;

    /// Get lines from the buffer.
    ///
    /// # Arguments
    /// - `start`: Start line (0-indexed, negative for relative to end)
    /// - `end`: End line (0-indexed, exclusive, -1 means end of buffer)
    /// - `strict_indexing`: If true, error on out-of-bounds; if false, clip/return empty
    ///
    /// # Returns
    /// Vector of lines (without trailing newlines)
    ///
    /// # Vim Quirks
    /// - Unloaded buffers always return empty vec, never error
    /// - Empty range (start == end) returns empty vec
    fn get_lines(&self, start: i64, end: i64, strict_indexing: bool) -> VimResult<Vec<String>>;

    /// Get a single line from the buffer.
    ///
    /// Convenience method for getting a single line.
    fn get_line(&self, line: i64) -> VimResult<String> {
        let lines = self.get_lines(line, line + 1, false)?;
        Ok(lines.into_iter().next().unwrap_or_default())
    }

    /// Set lines in the buffer.
    ///
    /// # Arguments
    /// - `start`: Start line (0-indexed, negative for relative to end)
    /// - `end`: End line (0-indexed, exclusive, -1 means end of buffer)
    /// - `strict_indexing`: If true, error on out-of-bounds
    /// - `replacement`: Lines to replace with (empty = delete lines)
    ///
    /// # Vim Quirks
    /// - Cannot reduce buffer to 0 lines; deleting all results in single empty line
    /// - Each string in replacement is a single line (cannot contain newlines)
    /// - Fails if buffer is not modifiable
    fn set_lines(
        &mut self,
        start: i64,
        end: i64,
        strict_indexing: bool,
        replacement: Vec<String>,
    ) -> VimResult<()>;

    /// Set text at character granularity.
    ///
    /// # Arguments
    /// - `start_row`: Start line (0-indexed)
    /// - `start_col`: Start column (0-indexed, byte offset)
    /// - `end_row`: End line (0-indexed)
    /// - `end_col`: End column (0-indexed, byte offset)
    /// - `replacement`: Lines to replace with
    ///
    /// # Vim Quirks
    /// - Can join/split lines
    /// - Updates cursor positions
    fn set_text(
        &mut self,
        start_row: i64,
        start_col: i64,
        end_row: i64,
        end_col: i64,
        replacement: Vec<String>,
    ) -> VimResult<()>;

    /// Append lines after the specified line.
    ///
    /// # Arguments
    /// - `line`: Line number to append after (0 = before first line)
    /// - `lines`: Lines to append
    fn append(&mut self, line: i64, lines: Vec<String>) -> VimResult<()> {
        self.set_lines(line, line, false, lines)
    }

    // ========================================================================
    // Buffer State
    // ========================================================================

    /// Check if buffer has unsaved modifications
    fn is_modified(&self) -> bool;

    /// Set the modified flag
    fn set_modified(&mut self, modified: bool) -> VimResult<()>;

    /// Check if buffer is modifiable
    fn is_modifiable(&self) -> bool;

    /// Set whether buffer is modifiable
    fn set_modifiable(&mut self, modifiable: bool) -> VimResult<()>;

    /// Check if buffer is read-only
    fn is_readonly(&self) -> bool;

    /// Set whether buffer is read-only
    fn set_readonly(&mut self, readonly: bool) -> VimResult<()>;

    /// Get the buffer type
    fn buftype(&self) -> BufferType;

    /// Set the buffer type
    fn set_buftype(&mut self, buftype: BufferType) -> VimResult<()>;

    /// Get the buffer hidden behavior
    fn bufhidden(&self) -> BufHidden;

    /// Set the buffer hidden behavior
    fn set_bufhidden(&mut self, bufhidden: BufHidden) -> VimResult<()>;

    /// Check if buffer is listed (appears in :ls)
    fn is_listed(&self) -> bool;

    /// Set whether buffer is listed
    fn set_listed(&mut self, listed: bool) -> VimResult<()>;

    /// Get the load state of the buffer
    fn load_state(&self) -> BufferLoadState;

    /// Get the changedtick (version number)
    fn changedtick(&self) -> u64;

    // ========================================================================
    // Buffer Lifecycle
    // ========================================================================

    /// Unload the buffer (remove content from memory, keep buffer valid)
    fn unload(&mut self) -> VimResult<()>;

    /// Delete the buffer (unlist and unload)
    fn delete(&mut self, force: bool) -> VimResult<()>;

    /// Wipe the buffer (completely remove, makes buffer invalid)
    fn wipe(&mut self, force: bool) -> VimResult<()>;

    // ========================================================================
    // Buffer Info
    // ========================================================================

    /// Get comprehensive buffer information
    fn info(&self) -> BufferInfo {
        BufferInfo {
            handle: self.handle(),
            name: self.name().to_string(),
            line_count: self.line_count(),
            modified: self.is_modified(),
            modifiable: self.is_modifiable(),
            readonly: self.is_readonly(),
            buftype: self.buftype(),
            bufhidden: self.bufhidden(),
            load_state: self.load_state(),
            listed: self.is_listed(),
            changedtick: self.changedtick(),
        }
    }
}

// ============================================================================
// Buffer Manager Trait
// ============================================================================

/// Manages multiple buffers
pub trait BufferManager {
    /// The buffer type this manager produces
    type Buf: Buffer;

    /// Create a new empty buffer
    fn create(&mut self) -> VimResult<BufHandle>;

    /// Create a new buffer with a name
    fn create_named(&mut self, name: &str) -> VimResult<BufHandle>;

    /// Get a buffer by handle
    fn get(&self, handle: BufHandle) -> Option<&Self::Buf>;

    /// Get a mutable buffer by handle
    fn get_mut(&mut self, handle: BufHandle) -> Option<&mut Self::Buf>;

    /// Get the current buffer
    fn current(&self) -> &Self::Buf;

    /// Get the current buffer mutably
    fn current_mut(&mut self) -> &mut Self::Buf;

    /// Set the current buffer
    fn set_current(&mut self, handle: BufHandle) -> VimResult<()>;

    /// List all valid buffer handles
    fn list(&self) -> Vec<BufHandle>;

    /// List all listed (visible in :ls) buffer handles
    fn list_listed(&self) -> Vec<BufHandle>;

    /// Delete a buffer
    fn delete(&mut self, handle: BufHandle, mode: BufDeleteMode, force: bool) -> VimResult<()>;

    /// Get buffer by name (file path)
    fn get_by_name(&self, name: &str) -> Option<BufHandle>;
}

// ============================================================================
// Behavioral Tests (portable)
// ============================================================================

/// Portable behavior checks derived from Neovim's buffer tests.
pub mod behavior {
    use super::*;

    /// Behavioral tests for buffer implementations.
    ///
    /// These checks document expected Vim behavior and can be run by any
    /// concrete `Buffer` implementation.
    pub trait BufferBehaviorTests: Buffer + Sized {
        // ====================================================================
        // Line Count Behavior
        // ====================================================================

        /// Test: New buffer has exactly 1 line (empty line)
        fn test_new_buffer_has_one_line(&self) {
            assert_eq!(self.line_count(), 1, "New buffer should have 1 line");
        }

        /// Test: Cannot reduce buffer below 1 line
        fn test_cannot_have_zero_lines(&mut self) {
            // Set some content
            self.set_lines(0, -1, false, vec!["line1".into(), "line2".into()])
                .unwrap();
            assert_eq!(self.line_count(), 2);

            // Delete all lines - should result in 1 empty line
            self.set_lines(0, -1, false, vec![]).unwrap();
            assert_eq!(
                self.line_count(),
                1,
                "Buffer should have 1 line after deleting all"
            );

            let lines = self.get_lines(0, -1, false).unwrap();
            assert_eq!(lines, vec![""], "Remaining line should be empty string");
        }

        // ====================================================================
        // get_lines Behavior
        // ====================================================================

        /// Test: Empty range returns empty vec
        fn test_empty_range_returns_empty(&self) {
            let lines = self.get_lines(0, 0, false).unwrap();
            assert!(lines.is_empty(), "Empty range should return empty vec");
        }

        /// Test: Negative index -1 means end of buffer
        fn test_negative_index_end(&mut self) {
            self.set_lines(0, -1, false, vec!["a".into(), "b".into(), "c".into()])
                .unwrap();

            let lines = self.get_lines(0, -1, false).unwrap();
            assert_eq!(lines, vec!["a", "b", "c"]);
        }

        /// Test: Negative start index
        fn test_negative_start_index(&mut self) {
            self.set_lines(0, -1, false, vec!["a".into(), "b".into(), "c".into()])
                .unwrap();

            // -1 = past end, so -1 to -1 should get nothing (empty range)
            // -2 to -1 should get last line
            let lines = self.get_lines(-2, -1, false).unwrap();
            assert_eq!(lines, vec!["c"]);

            let lines = self.get_lines(-1, -1, false).unwrap();
            assert!(lines.is_empty());
        }

        /// Test: Out of bounds with strict_indexing=false returns empty/clipped
        fn test_out_of_bounds_non_strict(&self) {
            let lines = self.get_lines(100, 200, false).unwrap();
            assert!(
                lines.is_empty(),
                "Out of bounds non-strict should return empty"
            );
        }

        /// Test: Out of bounds with strict_indexing=true returns error
        fn test_out_of_bounds_strict(&self) {
            let result = self.get_lines(100, 200, true);
            assert!(result.is_err(), "Out of bounds strict should error");
        }

        // ====================================================================
        // set_lines Behavior
        // ====================================================================

        /// Test: Insert lines at beginning
        fn test_insert_at_beginning(&mut self) {
            self.set_lines(0, -1, false, vec!["original".into()])
                .unwrap();
            self.set_lines(0, 0, false, vec!["inserted".into()])
                .unwrap();

            let lines = self.get_lines(0, -1, false).unwrap();
            assert_eq!(lines, vec!["inserted", "original"]);
        }

        /// Test: Insert lines at end
        fn test_insert_at_end(&mut self) {
            self.set_lines(0, -1, false, vec!["original".into()])
                .unwrap();
            self.set_lines(-1, -1, false, vec!["appended".into()])
                .unwrap();

            let lines = self.get_lines(0, -1, false).unwrap();
            assert_eq!(lines, vec!["original", "appended"]);
        }

        /// Test: Replace lines
        fn test_replace_lines(&mut self) {
            self.set_lines(0, -1, false, vec!["a".into(), "b".into(), "c".into()])
                .unwrap();
            self.set_lines(1, 2, false, vec!["B".into()]).unwrap();

            let lines = self.get_lines(0, -1, false).unwrap();
            assert_eq!(lines, vec!["a", "B", "c"]);
        }

        /// Test: Delete lines
        fn test_delete_lines(&mut self) {
            self.set_lines(0, -1, false, vec!["a".into(), "b".into(), "c".into()])
                .unwrap();
            self.set_lines(1, 2, false, vec![]).unwrap();

            let lines = self.get_lines(0, -1, false).unwrap();
            assert_eq!(lines, vec!["a", "c"]);
        }

        // ====================================================================
        // Modified State Behavior
        // ====================================================================

        /// Test: New buffer is not modified
        fn test_new_buffer_not_modified(&self) {
            assert!(!self.is_modified(), "New buffer should not be modified");
        }

        /// Test: Setting lines marks buffer as modified
        fn test_set_lines_marks_modified(&mut self) {
            self.set_lines(0, -1, false, vec!["content".into()])
                .unwrap();
            assert!(
                self.is_modified(),
                "Buffer should be modified after set_lines"
            );
        }

        /// Test: Can manually clear modified flag
        fn test_clear_modified_flag(&mut self) {
            self.set_lines(0, -1, false, vec!["content".into()])
                .unwrap();
            self.set_modified(false).unwrap();
            assert!(!self.is_modified(), "Modified flag should be clearable");
        }

        // ====================================================================
        // Modifiable Behavior
        // ====================================================================

        /// Test: Non-modifiable buffer rejects changes
        fn test_non_modifiable_rejects_changes(&mut self) {
            self.set_modifiable(false).unwrap();
            let result = self.set_lines(0, -1, false, vec!["content".into()]);
            assert!(
                result.is_err(),
                "Non-modifiable buffer should reject changes"
            );
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Behavioral Test Cases (for documentation)
    // ========================================================================

    /// These are the key behavioral test cases derived from Neovim's test suite.
    /// Any implementation of Buffer should pass these tests.
    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Buffer Behavioral Tests
        //!
        //! These tests document the expected Vim behavior for buffer operations.
        //! They are derived from Neovim's test suite:
        //! - test/functional/api/buffer_spec.lua
        //! - test/functional/vimscript/buf_spec.lua
        //!
        //! ## Key Quirks
        //!
        //! 1. **Minimum Line Count**: A buffer always has at least 1 line.
        //!    Deleting all lines results in a single empty line `[""]`.
        //!
        //! 2. **Negative Indices**: `-1` means end of buffer, `-2` second to last, etc.
        //!
        //! 3. **Exclusive End**: End index is exclusive (like Python slicing).
        //!
        //! 4. **Unloaded Buffers**: Return empty content, never error on read.
        //!
        //! 5. **Strict Indexing**: Only errors on out-of-bounds when strict=true.
        //!
        //! 6. **Changedtick**: Increments on every modification.
    }
}
