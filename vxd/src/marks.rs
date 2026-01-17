//! Mark system (local and global marks).
//!
//! Marks are saved positions in buffers that can be jumped to later.
//! There are local marks (per-buffer), global marks (across buffers),
//! and special automatic marks.
//!
//! # Key Behavioral Contracts
//!
//! - Local marks (a-z) are per-buffer
//! - Global marks (A-Z, 0-9) can span files
//! - Special marks are set automatically
//! - Marks adjust when text is inserted/deleted above them

use crate::buffer::BufHandle;
use crate::cursor::CursorPosition;
use crate::types::*;

// ============================================================================
// Mark Types
// ============================================================================

/// A mark identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mark {
    /// Local mark (a-z) - buffer-local
    Local(char),
    /// Global mark (A-Z) - can reference other files
    Global(char),
    /// Numbered mark (0-9) - set from shada/viminfo
    Numbered(u8),
    /// Last position before jump ('' or ``)
    LastJump,
    /// Last position where insert mode was stopped ('^' or `^`)
    LastInsert,
    /// Last change position ('.' or `.)
    LastChange,
    /// Start of last visual selection ('<' or `<)
    VisualStart,
    /// End of last visual selection ('>' or `>)
    VisualEnd,
    /// Start of last yank/put ('[' or `[)
    ChangeStart,
    /// End of last yank/put (']' or `])
    ChangeEnd,
    /// Position when last exiting buffer ('" or `")
    LastExit,
    /// Start of last inserted text
    InsertStart,
    /// End of last inserted text
    InsertEnd,
    /// First character of line for sentence/paragraph
    Sentence,
    /// First non-blank character
    FirstNonBlank,
}

impl Mark {
    /// Parse a mark from a character
    pub fn from_char(c: char) -> Result<Self, VimError> {
        match c {
            'a'..='z' => Ok(Mark::Local(c)),
            'A'..='Z' => Ok(Mark::Global(c)),
            '0'..='9' => Ok(Mark::Numbered(c as u8 - b'0')),
            '\'' | '`' => Ok(Mark::LastJump),
            '^' => Ok(Mark::LastInsert),
            '.' => Ok(Mark::LastChange),
            '<' => Ok(Mark::VisualStart),
            '>' => Ok(Mark::VisualEnd),
            '[' => Ok(Mark::ChangeStart),
            ']' => Ok(Mark::ChangeEnd),
            '"' => Ok(Mark::LastExit),
            _ => Err(VimError::InvalidMark(c)),
        }
    }

    /// Get the character representation of this mark
    pub fn to_char(&self) -> char {
        match self {
            Mark::Local(c) => *c,
            Mark::Global(c) => *c,
            Mark::Numbered(n) => (b'0' + n) as char,
            Mark::LastJump => '\'',
            Mark::LastInsert => '^',
            Mark::LastChange => '.',
            Mark::VisualStart => '<',
            Mark::VisualEnd => '>',
            Mark::ChangeStart => '[',
            Mark::ChangeEnd => ']',
            Mark::LastExit => '"',
            Mark::InsertStart => '[',
            Mark::InsertEnd => ']',
            Mark::Sentence => '(',
            Mark::FirstNonBlank => '^',
        }
    }

    /// Check if this is a local (buffer-specific) mark
    pub fn is_local(&self) -> bool {
        matches!(self, Mark::Local(_))
    }

    /// Check if this is a global (cross-file) mark
    pub fn is_global(&self) -> bool {
        matches!(self, Mark::Global(_) | Mark::Numbered(_))
    }

    /// Check if this mark is read-only (set automatically)
    pub fn is_readonly(&self) -> bool {
        matches!(
            self,
            Mark::LastChange
                | Mark::VisualStart
                | Mark::VisualEnd
                | Mark::ChangeStart
                | Mark::ChangeEnd
                | Mark::LastExit
                | Mark::InsertStart
                | Mark::InsertEnd
                | Mark::Sentence
                | Mark::FirstNonBlank
        )
    }
}

// ============================================================================
// Mark Value
// ============================================================================

/// The value stored for a mark
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkValue {
    /// Buffer this mark is in (None for unset or current buffer context)
    pub buffer: Option<BufHandle>,
    /// Position in the buffer
    pub position: CursorPosition,
    /// File path (for global marks that reference files)
    pub file: Option<String>,
}

impl MarkValue {
    /// Create a mark value for the current buffer
    pub fn new(position: CursorPosition) -> Self {
        MarkValue {
            buffer: None,
            position,
            file: None,
        }
    }

    /// Create a mark value for a specific buffer
    pub fn in_buffer(buffer: BufHandle, position: CursorPosition) -> Self {
        MarkValue {
            buffer: Some(buffer),
            position,
            file: None,
        }
    }

    /// Create a mark value for a file (global mark)
    pub fn in_file(file: String, position: CursorPosition) -> Self {
        MarkValue {
            buffer: None,
            position,
            file: Some(file),
        }
    }
}

// ============================================================================
// Jump List
// ============================================================================

/// An entry in the jump list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpEntry {
    /// Buffer handle
    pub buffer: BufHandle,
    /// Position in buffer
    pub position: CursorPosition,
    /// File path (if available)
    pub file: Option<String>,
}

/// The jump list tracks locations jumped from
pub trait JumpList {
    /// Get the current position in the jump list
    fn position(&self) -> usize;

    /// Get the total number of entries
    fn len(&self) -> usize;

    /// Check if jump list is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an entry at a specific index
    fn get(&self, index: usize) -> Option<&JumpEntry>;

    /// Add a new jump entry (called before jumping)
    fn push(&mut self, entry: JumpEntry);

    /// Go to older entry (Ctrl-O)
    fn go_older(&mut self) -> Option<&JumpEntry>;

    /// Go to newer entry (Ctrl-I / Tab)
    fn go_newer(&mut self) -> Option<&JumpEntry>;

    /// Clear the jump list
    fn clear(&mut self);
}

// ============================================================================
// Change List
// ============================================================================

/// An entry in the change list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeEntry {
    /// Position of the change
    pub position: CursorPosition,
    /// Column (for virtual column restoration)
    pub col: usize,
}

/// The change list tracks locations where changes occurred
pub trait ChangeList {
    /// Get the current position in the change list
    fn position(&self) -> usize;

    /// Get the total number of entries
    fn len(&self) -> usize;

    /// Check if change list is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an entry at a specific index
    fn get(&self, index: usize) -> Option<&ChangeEntry>;

    /// Add a new change entry
    fn push(&mut self, entry: ChangeEntry);

    /// Go to older change (g;)
    fn go_older(&mut self) -> Option<&ChangeEntry>;

    /// Go to newer change (g,)
    fn go_newer(&mut self) -> Option<&ChangeEntry>;

    /// Clear the change list
    fn clear(&mut self);
}

// ============================================================================
// Mark Manager Trait
// ============================================================================

/// Manages marks for a buffer or globally
pub trait MarkManager {
    /// Get the value of a mark
    fn get(&self, mark: Mark) -> Option<&MarkValue>;

    /// Set a mark
    fn set(&mut self, mark: Mark, value: MarkValue) -> VimResult<()>;

    /// Delete a mark
    fn delete(&mut self, mark: Mark) -> VimResult<()>;

    /// List all set marks
    fn list(&self) -> Vec<(Mark, &MarkValue)>;

    /// Adjust marks after buffer modification
    ///
    /// Called when text is inserted or deleted to keep marks valid.
    fn adjust(&mut self, line: LineNr, col: usize, lines_added: i64, bytes_added: i64);

    /// Get the jump list
    fn jump_list(&self) -> &dyn JumpList;

    /// Get the mutable jump list
    fn jump_list_mut(&mut self) -> &mut dyn JumpList;

    /// Get the change list
    fn change_list(&self) -> &dyn ChangeList;

    /// Get the mutable change list
    fn change_list_mut(&mut self) -> &mut dyn ChangeList;

    /// Record a jump (before actually jumping)
    fn record_jump(&mut self, from: CursorPosition);

    /// Record a change location
    fn record_change(&mut self, at: CursorPosition);

    /// Set the last visual selection marks
    fn set_visual_marks(&mut self, start: CursorPosition, end: CursorPosition);

    /// Set the last change/yank marks
    fn set_change_marks(&mut self, start: CursorPosition, end: CursorPosition);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_from_char() {
        assert_eq!(Mark::from_char('a').unwrap(), Mark::Local('a'));
        assert_eq!(Mark::from_char('z').unwrap(), Mark::Local('z'));
        assert_eq!(Mark::from_char('A').unwrap(), Mark::Global('A'));
        assert_eq!(Mark::from_char('Z').unwrap(), Mark::Global('Z'));
        assert_eq!(Mark::from_char('0').unwrap(), Mark::Numbered(0));
        assert_eq!(Mark::from_char('.').unwrap(), Mark::LastChange);
        assert_eq!(Mark::from_char('<').unwrap(), Mark::VisualStart);
        assert_eq!(Mark::from_char('[').unwrap(), Mark::ChangeStart);
    }

    #[test]
    fn test_mark_locality() {
        assert!(Mark::Local('a').is_local());
        assert!(!Mark::Local('a').is_global());
        assert!(!Mark::Global('A').is_local());
        assert!(Mark::Global('A').is_global());
        assert!(Mark::Numbered(0).is_global());
    }

    /// Behavioral tests for mark implementations
    pub trait MarkBehaviorTests: MarkManager + Sized {
        /// Test: Local marks are buffer-specific
        fn test_local_marks_per_buffer(&mut self);

        /// Test: Global marks persist across buffers
        fn test_global_marks_cross_buffer(&mut self);

        /// Test: Marks adjust when lines inserted above
        fn test_marks_adjust_on_insert(&mut self);

        /// Test: Marks adjust when lines deleted above
        fn test_marks_adjust_on_delete(&mut self);

        /// Test: Jump list records positions before jump
        fn test_jump_list_records(&mut self);

        /// Test: Ctrl-O/Ctrl-I navigate jump list
        fn test_jump_list_navigation(&mut self);

        /// Test: Change list tracks edit locations
        fn test_change_list_tracks(&mut self);

        /// Test: Visual marks are set after visual selection
        fn test_visual_marks(&mut self);

        /// Test: Change marks are set after yank/put
        fn test_change_marks(&mut self);
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Mark Behavioral Tests
        //!
        //! Tests derived from Neovim's test suite:
        //! - test/functional/editor/mark_spec.lua
        //! - test/functional/legacy/025_jump_spec.lua
        //!
        //! ## Key Quirks
        //!
        //! 1. **Mark adjustment**: When lines are added above a mark, the mark
        //!    moves down. When lines are deleted, the mark moves up (or is
        //!    invalidated if its line is deleted).
        //!
        //! 2. **Jump list**: Certain commands add to the jump list (:tag, G, /,
        //!    etc.) while others don't (j, k, h, l).
        //!
        //! 3. **Change list**: The change list is per-buffer and limited in size.
        //!
        //! 4. **' vs `**: Single quote goes to first non-blank of mark's line,
        //!    backtick goes to exact column.
        //!
        //! 5. **Numbered marks**: 0-9 are special and set from shada on startup.
    }
}
