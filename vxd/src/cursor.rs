//! Cursor movement and positioning.
//!
//! This module defines cursor behavior in Vim. The cursor has complex
//! constraints that vary by mode, and maintains memory of desired
//! column positions.
//!
//! # Key Behavioral Contracts
//!
//! - Line is always clamped to [1, line_count]
//! - Column constraints vary by mode (normal < insert < virtualedit)
//! - Cursor remembers desired column (`curswant`) for vertical movement
//! - Cannot position in middle of multibyte characters
//! - Virtual column differs from byte column (tabs, wide chars)

use crate::types::*;

// ============================================================================
// Cursor Position Types
// ============================================================================

/// Cursor position in a buffer.
///
/// This matches Vim's internal `pos_T` structure.
/// - `line`: 1-indexed line number
/// - `col`: 0-indexed byte offset into the line
/// - `coladd`: Additional virtual column offset (virtualedit mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CursorPosition {
    /// Line number (1-indexed, MINLNUM to MAXLNUM)
    pub line: LineNr,
    /// Column (0-indexed byte offset)
    pub col: usize,
    /// Virtual column addition (for virtualedit mode)
    pub coladd: usize,
}

impl CursorPosition {
    /// Minimum line number
    pub const MINLNUM: usize = 1;
    /// Maximum line number
    pub const MAXLNUM: usize = 0x7fffffff;
    /// Minimum column
    pub const MINCOL: usize = 0;
    /// Maximum column (used for "end of line" memory)
    pub const MAXCOL: usize = 0x7fffffff;

    /// Create a new cursor position
    pub fn new(line: LineNr, col: usize) -> Self {
        CursorPosition {
            line,
            col,
            coladd: 0,
        }
    }

    /// Create a cursor position with virtual column offset
    pub fn with_coladd(line: LineNr, col: usize, coladd: usize) -> Self {
        CursorPosition { line, col, coladd }
    }

    /// Origin position (line 1, col 0)
    pub const ORIGIN: CursorPosition = CursorPosition {
        line: LineNr(1),
        col: 0,
        coladd: 0,
    };
}

/// The "wanted" or "desired" column for vertical movement.
///
/// When moving vertically (j/k), Vim remembers the desired column
/// and tries to return to it. This is set by horizontal movements
/// and preserved by vertical movements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorWant {
    /// A specific column position
    Column(usize),
    /// End of line (set by `$` command)
    EndOfLine,
}

impl CursorWant {
    /// Get the column value, with MAXCOL for EndOfLine
    pub fn value(&self) -> usize {
        match self {
            CursorWant::Column(c) => *c,
            CursorWant::EndOfLine => CursorPosition::MAXCOL,
        }
    }
}

impl Default for CursorWant {
    fn default() -> Self {
        CursorWant::Column(0)
    }
}

// ============================================================================
// Cursor Shape Types
// ============================================================================

/// Shape of the cursor for display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorShape {
    /// Block cursor (covers entire cell)
    Block,
    /// Horizontal bar (percentage of cell height from bottom)
    Horizontal(u8),
    /// Vertical bar (percentage of cell width from left)
    Vertical(u8),
}

impl Default for CursorShape {
    fn default() -> Self {
        CursorShape::Block
    }
}

/// Cursor blink timing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CursorBlink {
    /// Delay before cursor starts blinking (ms), 0 = no delay
    pub blinkwait: u32,
    /// Time cursor is shown (ms), 0 = no blink
    pub blinkon: u32,
    /// Time cursor is hidden (ms), 0 = no blink
    pub blinkoff: u32,
}

impl CursorBlink {
    /// No blinking
    pub const NONE: CursorBlink = CursorBlink {
        blinkwait: 0,
        blinkon: 0,
        blinkoff: 0,
    };

    /// Default terminal blinking
    pub const TERMINAL: CursorBlink = CursorBlink {
        blinkwait: 0,
        blinkon: 500,
        blinkoff: 500,
    };
}

/// Complete cursor style information for a mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CursorStyle {
    /// Shape of the cursor
    pub shape: CursorShape,
    /// Blink timing
    pub blink: CursorBlink,
    /// Highlight group name (e.g., "Cursor", "lCursor")
    pub attr_id: u32,
}

impl Default for CursorStyle {
    fn default() -> Self {
        CursorStyle {
            shape: CursorShape::Block,
            blink: CursorBlink::NONE,
            attr_id: 0,
        }
    }
}

// ============================================================================
// Cursor Mode Context
// ============================================================================

/// Context that affects cursor positioning rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CursorContext {
    /// Current mode allows positioning past EOL
    pub allow_past_eol: bool,
    /// Virtualedit mode is enabled
    pub virtualedit: VirtualEdit,
    /// In visual mode with selection != 'old'
    pub visual_selection: bool,
}

/// Virtual edit mode setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VirtualEdit {
    /// No virtual editing
    #[default]
    None,
    /// Allow virtual editing in block visual mode
    Block,
    /// Allow virtual editing in insert mode
    Insert,
    /// Allow cursor one position past EOL
    OneMore,
    /// Allow virtual editing everywhere
    All,
}

impl VirtualEdit {
    /// Check if virtualedit allows positioning past EOL
    pub fn allows_past_eol(&self) -> bool {
        matches!(self, VirtualEdit::OneMore | VirtualEdit::All)
    }

    /// Check if virtualedit allows arbitrary positioning
    pub fn allows_anywhere(&self) -> bool {
        matches!(self, VirtualEdit::All)
    }
}

// ============================================================================
// Cursor Trait
// ============================================================================

/// The Cursor trait defines Vim's expected cursor behaviors.
///
/// # Position Semantics
///
/// - Line numbers are 1-indexed (1 to line_count)
/// - Column is 0-indexed byte offset
/// - Virtual column accounts for tabs and wide characters
///
/// # Mode-Dependent Constraints
///
/// | Mode | Can be past EOL? | Notes |
/// |------|------------------|-------|
/// | Normal | No | Clamped to last char |
/// | Insert | Yes (one past) | For appending |
/// | Visual | Depends | If selection != 'old' |
/// | Replace | No | Like normal mode |
/// | Virtual Edit | Yes | Uses coladd |
pub trait Cursor {
    // ========================================================================
    // Position Access
    // ========================================================================

    /// Get the current cursor position
    fn position(&self) -> CursorPosition;

    /// Get just the line number
    fn line(&self) -> LineNr {
        self.position().line
    }

    /// Get just the column (0-indexed byte offset)
    fn col(&self) -> usize {
        self.position().col
    }

    /// Get the virtual column addition
    fn coladd(&self) -> usize {
        self.position().coladd
    }

    /// Get the desired/wanted column for vertical movement
    fn curswant(&self) -> CursorWant;

    // ========================================================================
    // Position Setting
    // ========================================================================

    /// Set the cursor position.
    ///
    /// The position will be validated and clamped according to the
    /// current mode and virtualedit settings.
    ///
    /// # Arguments
    /// - `pos`: Target position
    /// - `ctx`: Context affecting validation rules
    fn set_position(&mut self, pos: CursorPosition, ctx: &CursorContext) -> VimResult<()>;

    /// Set just the line, preserving column if possible
    fn set_line(&mut self, line: LineNr, ctx: &CursorContext) -> VimResult<()>;

    /// Set just the column
    fn set_col(&mut self, col: usize, ctx: &CursorContext) -> VimResult<()>;

    /// Set the wanted column (for vertical movement memory)
    fn set_curswant(&mut self, want: CursorWant);

    /// Update curswant to current column (called by horizontal movements)
    fn update_curswant(&mut self) {
        self.set_curswant(CursorWant::Column(self.col()));
    }

    /// Set curswant to end of line (called by `$` command)
    fn set_curswant_eol(&mut self) {
        self.set_curswant(CursorWant::EndOfLine);
    }

    // ========================================================================
    // Virtual Column
    // ========================================================================

    /// Get the virtual (display) column.
    ///
    /// This accounts for tabs expanding to tabstop width and
    /// multibyte characters having different display widths.
    fn virtcol(&self) -> usize;

    /// Get the virtual column at a specific position
    fn virtcol_at(&self, pos: CursorPosition) -> usize;

    /// Convert virtual column to byte column for a given line
    fn virtcol_to_col(&self, line: LineNr, vcol: usize) -> usize;

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate and clamp cursor position to buffer bounds.
    ///
    /// Called after buffer modifications to ensure cursor is valid.
    fn check_cursor(&mut self, ctx: &CursorContext);

    /// Validate just the line number
    fn check_cursor_lnum(&mut self, max_line: LineNr);

    /// Validate just the column for current line
    fn check_cursor_col(&mut self, line_len: usize, ctx: &CursorContext);

    // ========================================================================
    // Cursor Style
    // ========================================================================

    /// Get the cursor style for the current mode
    fn style(&self) -> CursorStyle;
}

// ============================================================================
// Cursor Manager Trait
// ============================================================================

/// Manages cursor state for a window
pub trait CursorManager: Cursor {
    /// Save the current cursor position (for restoration)
    fn save(&self) -> CursorPosition;

    /// Restore a previously saved cursor position
    fn restore(&mut self, pos: CursorPosition, ctx: &CursorContext);

    /// Move cursor to beginning of line (column 0)
    fn move_to_bol(&mut self, ctx: &CursorContext) -> VimResult<()> {
        self.set_col(0, ctx)
    }

    /// Move cursor to first non-blank character
    fn move_to_first_nonblank(&mut self, line_content: &str, ctx: &CursorContext) -> VimResult<()> {
        let col = line_content.find(|c: char| !c.is_whitespace()).unwrap_or(0);
        self.set_col(col, ctx)
    }

    /// Move cursor to end of line
    fn move_to_eol(&mut self, line_len: usize, ctx: &CursorContext) -> VimResult<()> {
        let col = if ctx.allow_past_eol || ctx.virtualedit.allows_past_eol() {
            line_len
        } else if line_len > 0 {
            line_len - 1
        } else {
            0
        };
        self.set_col(col, ctx)?;
        self.set_curswant_eol();
        Ok(())
    }

    /// Adjust cursor after text change
    ///
    /// # Arguments
    /// - `change_line`: Line where change occurred (1-indexed)
    /// - `change_col`: Column where change started (0-indexed)
    /// - `deleted_bytes`: Number of bytes deleted
    /// - `added_bytes`: Number of bytes added
    fn adjust_for_change(
        &mut self,
        change_line: LineNr,
        change_col: usize,
        deleted_bytes: usize,
        added_bytes: usize,
        ctx: &CursorContext,
    );
}

// ============================================================================
// Default Cursor Styles by Mode
// ============================================================================

/// Get the default cursor style for a mode name
pub fn default_cursor_style(mode: &str) -> CursorStyle {
    match mode {
        "normal" | "n" => CursorStyle {
            shape: CursorShape::Block,
            blink: CursorBlink::NONE,
            attr_id: 0,
        },
        "visual" | "v" => CursorStyle {
            shape: CursorShape::Block,
            blink: CursorBlink::NONE,
            attr_id: 0,
        },
        "insert" | "i" => CursorStyle {
            shape: CursorShape::Vertical(25),
            blink: CursorBlink::NONE,
            attr_id: 0,
        },
        "replace" | "r" => CursorStyle {
            shape: CursorShape::Horizontal(20),
            blink: CursorBlink::NONE,
            attr_id: 0,
        },
        "cmdline_normal" | "c" => CursorStyle {
            shape: CursorShape::Block,
            blink: CursorBlink::NONE,
            attr_id: 0,
        },
        "cmdline_insert" | "ci" => CursorStyle {
            shape: CursorShape::Vertical(25),
            blink: CursorBlink::NONE,
            attr_id: 0,
        },
        "cmdline_replace" | "cr" => CursorStyle {
            shape: CursorShape::Horizontal(20),
            blink: CursorBlink::NONE,
            attr_id: 0,
        },
        "operator" | "o" => CursorStyle {
            shape: CursorShape::Horizontal(50),
            blink: CursorBlink::NONE,
            attr_id: 0,
        },
        "visual_select" | "ve" => CursorStyle {
            shape: CursorShape::Vertical(25),
            blink: CursorBlink::NONE,
            attr_id: 0,
        },
        "terminal" | "t" => CursorStyle {
            shape: CursorShape::Block,
            blink: CursorBlink::TERMINAL,
            attr_id: 0, // Would be HL_ATTR(HLF_TERM) in Vim
        },
        _ => CursorStyle::default(),
    }
}

// ============================================================================
// Behavioral Tests (portable)
// ============================================================================

/// Portable behavior checks derived from Neovim cursor tests.
pub mod behavior {
    use super::*;

    /// Behavioral tests for cursor implementations.
    pub trait CursorBehaviorTests: Cursor + Sized {
        /// Get the line length for testing column clamping.
        fn test_line_len(&self) -> usize;

        // ====================================================================
        // Position Clamping Tests
        // ====================================================================

        /// Test: Cursor line is clamped to valid range.
        fn test_line_clamping(&mut self, max_line: LineNr) {
            let ctx = CursorContext::default();

            // Line 0 should clamp to 1
            let pos = CursorPosition::new(LineNr(0), 0);
            self.set_position(pos, &ctx).ok();
            assert!(self.line().0 >= 1, "Line should be at least 1");

            // Line past max should clamp to max
            let pos = CursorPosition::new(LineNr(max_line.0 + 100), 0);
            self.set_position(pos, &ctx).ok();
            assert!(
                self.line().0 <= max_line.0,
                "Line should be at most max_line"
            );
        }

        /// Test: Normal mode column clamping (can't be past EOL).
        fn test_normal_mode_col_clamping(&mut self) {
            let ctx = CursorContext {
                allow_past_eol: false,
                virtualedit: VirtualEdit::None,
                visual_selection: false,
            };

            let line_len = self.test_line_len();
            if line_len == 0 {
                return; // Skip for empty lines
            }

            // Try to set column past end of line
            let pos = CursorPosition::new(self.line(), line_len + 10);
            self.set_position(pos, &ctx).ok();

            // Should be clamped to last character
            assert!(
                self.col() < line_len,
                "Normal mode: col should be < line_len"
            );
        }

        /// Test: Insert mode allows one past EOL.
        fn test_insert_mode_past_eol(&mut self) {
            let ctx = CursorContext {
                allow_past_eol: true,
                virtualedit: VirtualEdit::None,
                visual_selection: false,
            };

            let line_len = self.test_line_len();
            let pos = CursorPosition::new(self.line(), line_len);
            self.set_position(pos, &ctx).ok();

            // Should be allowed at line_len (one past last char)
            assert_eq!(
                self.col(),
                line_len,
                "Insert mode: col should be allowed at line_len"
            );
        }

        /// Test: Virtualedit allows coladd.
        fn test_virtualedit_coladd(&mut self) {
            let ctx = CursorContext {
                allow_past_eol: false,
                virtualedit: VirtualEdit::All,
                visual_selection: false,
            };

            let line_len = self.test_line_len();
            let pos = CursorPosition::with_coladd(self.line(), line_len, 5);
            self.set_position(pos, &ctx).ok();

            // With virtualedit, coladd should be preserved
            // (actual behavior depends on implementation)
        }

        // ====================================================================
        // Curswant Tests
        // ====================================================================

        /// Test: Curswant is preserved across vertical movement.
        fn test_curswant_preserved(&mut self) {
            // Set a specific column
            let ctx = CursorContext::default();
            let line_len = self.test_line_len();
            let target = if line_len == 0 {
                0
            } else {
                10.min(line_len.saturating_sub(1))
            };
            self.set_col(target, &ctx).ok();
            self.update_curswant();

            // Curswant should remember the column
            assert_eq!(
                self.curswant(),
                CursorWant::Column(target),
                "Curswant should remember column"
            );
        }

        /// Test: $ sets curswant to MAXCOL.
        fn test_dollar_sets_maxcol(&mut self) {
            self.set_curswant_eol();

            assert_eq!(
                self.curswant(),
                CursorWant::EndOfLine,
                "$ should set curswant to EndOfLine"
            );

            assert_eq!(
                self.curswant().value(),
                CursorPosition::MAXCOL,
                "EndOfLine value should be MAXCOL"
            );
        }

        // ====================================================================
        // Empty Line Tests
        // ====================================================================

        /// Test: Cursor on empty line has col 0.
        fn test_empty_line_col_zero(&mut self) {
            // On an empty line, column should be 0
            // This test assumes we can set up an empty line
            let ctx = CursorContext::default();
            self.check_cursor_col(0, &ctx);

            assert_eq!(self.col(), 0, "Empty line should have col 0");
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
    // Behavioral Documentation Tests
    // ========================================================================

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Cursor Behavioral Tests
        //!
        //! These tests document expected Vim cursor behavior from:
        //! - test/functional/api/window_spec.lua (nvim_win_get_cursor, nvim_win_set_cursor)
        //! - test/functional/ui/cursor_spec.lua (cursor shapes, blinking)
        //! - test/functional/editor/cursor_spec.lua (cursor movement edge cases)
        //!
        //! ## Key Quirks
        //!
        //! 1. **0-indexed API, 1-indexed display**: API uses 0-indexed columns,
        //!    but line numbers are 1-indexed. Display shows 1-indexed for both.
        //!
        //! 2. **Normal mode clamping**: Cursor is clamped to [0, line_len-1] in
        //!    normal mode, but [0, line_len] in insert mode.
        //!
        //! 3. **Curswant memory**: Vertical movements (j/k) preserve the desired
        //!    column. Horizontal movements update it.
        //!
        //! 4. **$ special case**: The `$` command sets curswant to MAXCOL, making
        //!    vertical movements go to end of each line.
        //!
        //! 5. **Multibyte alignment**: Cursor cannot land in middle of a multibyte
        //!    character; it's adjusted to the character boundary.
        //!
        //! 6. **Virtualedit**: When enabled, allows cursor to be positioned past
        //!    EOL using `coladd` field.
    }
}
