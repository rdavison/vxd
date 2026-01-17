//! Undo/redo tree.
//!
//! Vim maintains an undo tree (not just a linear history), allowing
//! navigation to any previous state.

use crate::cursor::CursorPosition;
use crate::types::*;
use std::time::SystemTime;

// ============================================================================
// Undo Types
// ============================================================================

/// A single change in the undo history
#[derive(Debug, Clone)]
pub struct UndoChange {
    /// Start line of change
    pub start_line: LineNr,
    /// End line of change (before)
    pub end_line: LineNr,
    /// Lines that were there before
    pub old_lines: Vec<String>,
    /// Lines that replaced them
    pub new_lines: Vec<String>,
    /// Cursor position before change
    pub cursor_before: CursorPosition,
    /// Cursor position after change
    pub cursor_after: CursorPosition,
}

/// An undo entry (can contain multiple changes)
#[derive(Debug, Clone)]
pub struct UndoEntry {
    /// Unique sequence number
    pub seq: usize,
    /// Changes in this entry
    pub changes: Vec<UndoChange>,
    /// Time of this entry
    pub time: SystemTime,
    /// Whether buffer was modified before this entry
    pub modified_before: bool,
}

/// A node in the undo tree
#[derive(Debug, Clone)]
pub struct UndoNode {
    /// Entry at this node
    pub entry: UndoEntry,
    /// Parent node sequence number
    pub parent: Option<usize>,
    /// Child nodes (branches)
    pub children: Vec<usize>,
    /// Alternate branch (for :earlier/:later navigation)
    pub alt: Option<usize>,
}

/// Undo tree state
#[derive(Debug, Clone, Default)]
pub struct UndoTreeState {
    /// Current position in tree (sequence number)
    pub current: usize,
    /// Total number of entries
    pub entry_count: usize,
    /// Current save point (for 'modified' state)
    pub save_point: usize,
    /// Whether tree has been synced
    pub synced: bool,
}

// ============================================================================
// Undo Tree Trait
// ============================================================================

/// Trait for undo tree operations
pub trait UndoTree {
    /// Get the current state
    fn state(&self) -> &UndoTreeState;

    /// Begin a new undo block (groups changes)
    fn begin_block(&mut self);

    /// End the current undo block
    fn end_block(&mut self);

    /// Add a change to the current block
    fn add_change(&mut self, change: UndoChange);

    /// Undo the last change (or block)
    fn undo(&mut self) -> VimResult<Option<&UndoEntry>>;

    /// Redo the last undone change
    fn redo(&mut self) -> VimResult<Option<&UndoEntry>>;

    /// Go to a specific entry by sequence number
    fn go_to(&mut self, seq: usize) -> VimResult<()>;

    /// Go to state at a specific time
    fn go_to_time(&mut self, time: SystemTime) -> VimResult<()>;

    /// Get entry by sequence number
    fn get_entry(&self, seq: usize) -> Option<&UndoEntry>;

    /// Get the undo tree for visualization
    fn tree(&self) -> Vec<&UndoNode>;

    /// Clear the undo history
    fn clear(&mut self);

    /// Set the save point (current becomes unmodified state)
    fn set_save_point(&mut self);

    /// Check if at save point
    fn at_save_point(&self) -> bool {
        self.state().current == self.state().save_point
    }

    /// Get number of changes that can be undone
    fn undo_count(&self) -> usize;

    /// Get number of changes that can be redone
    fn redo_count(&self) -> usize;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Undo Behavioral Tests
        //!
        //! ## Key Quirks
        //!
        //! 1. **Undo tree**: Unlike most editors, Vim never loses undo history.
        //!    Undo then making changes creates a new branch.
        //!
        //! 2. **Undo blocks**: Changes are grouped into blocks. Normal mode
        //!    commands create blocks; insert mode is one block.
        //!
        //! 3. **Time navigation**: `:earlier` and `:later` can navigate by time.
        //!
        //! 4. **Persistent undo**: With 'undofile', undo history survives restart.
        //!
        //! 5. **Join behavior**: `u` after `J` undoes the join as one operation.
    }
}
