//! Visual mode selections.
//!
//! Visual mode allows selecting text before applying operators.
//! There are three types: characterwise, linewise, and blockwise.

use crate::cursor::CursorPosition;
use crate::modes::VisualMode;
use crate::types::*;

// ============================================================================
// Selection Types
// ============================================================================

/// A visual selection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualSelection {
    /// Selection start (anchor)
    pub start: CursorPosition,
    /// Selection end (cursor)
    pub end: CursorPosition,
    /// Selection type
    pub mode: VisualMode,
}

impl VisualSelection {
    /// Create a new selection at a single position
    pub fn new(pos: CursorPosition, mode: VisualMode) -> Self {
        VisualSelection {
            start: pos,
            end: pos,
            mode,
        }
    }

    /// Get the normalized selection (start <= end)
    pub fn normalized(&self) -> (CursorPosition, CursorPosition) {
        if self.start.line.0 < self.end.line.0
            || (self.start.line == self.end.line && self.start.col <= self.end.col)
        {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Get the line range of the selection
    pub fn line_range(&self) -> LineRange {
        let (start, end) = self.normalized();
        LineRange::new(start.line, end.line)
    }

    /// Check if selection spans multiple lines
    pub fn is_multiline(&self) -> bool {
        self.start.line != self.end.line
    }

    /// Convert to linewise if needed
    pub fn as_linewise(&self) -> VisualSelection {
        VisualSelection {
            start: CursorPosition::new(self.start.line, 0),
            end: CursorPosition::new(self.end.line, 0),
            mode: VisualMode::Line,
        }
    }
}

/// Block selection info (for blockwise visual)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSelection {
    /// Start line
    pub start_line: LineNr,
    /// End line
    pub end_line: LineNr,
    /// Start column (virtual column)
    pub start_vcol: usize,
    /// End column (virtual column)
    pub end_vcol: usize,
}

impl BlockSelection {
    /// Get number of lines in block
    pub fn height(&self) -> usize {
        self.end_line.0 - self.start_line.0 + 1
    }

    /// Get width of block
    pub fn width(&self) -> usize {
        if self.end_vcol >= self.start_vcol {
            self.end_vcol - self.start_vcol + 1
        } else {
            self.start_vcol - self.end_vcol + 1
        }
    }
}

// ============================================================================
// Visual Selection Trait
// ============================================================================

/// Trait for managing visual selections
pub trait VisualSelectionManager {
    /// Check if visual mode is active
    fn is_active(&self) -> bool;

    /// Get the current selection
    fn selection(&self) -> Option<&VisualSelection>;

    /// Start a visual selection
    fn start(&mut self, pos: CursorPosition, mode: VisualMode);

    /// Update the selection end (cursor moved)
    fn update(&mut self, pos: CursorPosition);

    /// Change the selection mode (v, V, Ctrl-V toggle)
    fn change_mode(&mut self, mode: VisualMode);

    /// Clear the selection (exit visual mode)
    fn clear(&mut self);

    /// Swap start and end (o command)
    fn swap_ends(&mut self);

    /// Get selected text
    fn get_text(&self, get_line: impl Fn(LineNr) -> Option<String>) -> Vec<String>;

    /// Check if a position is within the selection
    fn contains(&self, pos: CursorPosition) -> bool;

    /// Get block selection info (for blockwise mode)
    fn block_info(&self) -> Option<BlockSelection>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_normalized() {
        let sel = VisualSelection {
            start: CursorPosition::new(LineNr(5), 10),
            end: CursorPosition::new(LineNr(3), 5),
            mode: VisualMode::Char,
        };

        let (start, end) = sel.normalized();
        assert_eq!(start.line, LineNr(3));
        assert_eq!(end.line, LineNr(5));
    }

    #[test]
    fn test_block_dimensions() {
        let block = BlockSelection {
            start_line: LineNr(1),
            end_line: LineNr(5),
            start_vcol: 10,
            end_vcol: 20,
        };

        assert_eq!(block.height(), 5);
        assert_eq!(block.width(), 11);
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Visual Mode Behavioral Tests
        //!
        //! ## Key Quirks
        //!
        //! 1. **Selection direction**: Selection has an anchor (start) and cursor
        //!    (end). The `o` command swaps them.
        //!
        //! 2. **Mode toggling**: `v` from visual goes to normal. `V` from visual
        //!    switches to linewise.
        //!
        //! 3. **Reselect**: `gv` reselects the last visual selection.
        //!
        //! 4. **Block mode**: In block mode, short lines are extended with virtual
        //!    space if 'virtualedit' allows.
        //!
        //! 5. **Inclusive end**: The character at the end position is included
        //!    (unless `selection=exclusive`).
    }
}
