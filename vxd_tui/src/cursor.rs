//! Cursor implementation.
//!
//! This module provides a concrete implementation of cursor management.

use vxd::cursor::{Cursor, CursorContext, CursorManager, CursorPosition, CursorStyle, CursorWant};
use vxd::types::{LineNr, VimResult};

/// Concrete cursor implementation
#[derive(Debug, Clone)]
pub struct TuiCursor {
    /// Current position
    position: CursorPosition,
    /// Desired/wanted column for vertical movement
    curswant: CursorWant,
    /// Current cursor style
    style: CursorStyle,
    /// Line length getter (for validation)
    line_lengths: Vec<usize>,
}

impl TuiCursor {
    /// Create a new cursor at the origin
    pub fn new() -> Self {
        TuiCursor {
            position: CursorPosition::ORIGIN,
            curswant: CursorWant::default(),
            style: CursorStyle::default(),
            line_lengths: vec![0], // At least one empty line
        }
    }

    /// Set line lengths for validation
    pub fn set_line_lengths(&mut self, lengths: Vec<usize>) {
        self.line_lengths = lengths;
        if self.line_lengths.is_empty() {
            self.line_lengths.push(0);
        }
    }

    /// Update line lengths from buffer
    pub fn update_line_lengths(&mut self, lines: &[String]) {
        self.line_lengths = lines.iter().map(|l| l.len()).collect();
        if self.line_lengths.is_empty() {
            self.line_lengths.push(0);
        }
    }

    /// Get line length at given line number
    fn line_len(&self, line: LineNr) -> usize {
        let idx = line.to_zero_indexed();
        self.line_lengths.get(idx).copied().unwrap_or(0)
    }

    /// Get total number of lines
    fn line_count(&self) -> usize {
        self.line_lengths.len()
    }
}

impl Default for TuiCursor {
    fn default() -> Self {
        Self::new()
    }
}

impl Cursor for TuiCursor {
    fn position(&self) -> CursorPosition {
        self.position
    }

    fn curswant(&self) -> CursorWant {
        self.curswant
    }

    fn set_position(&mut self, pos: CursorPosition, ctx: &CursorContext) -> VimResult<()> {
        // Clamp line to valid range
        let max_line = self.line_count().max(1);
        let line = LineNr(pos.line.0.clamp(1, max_line));

        // Get line length for column clamping
        let line_len = self.line_len(line);

        // Clamp column based on context
        let col = if ctx.allow_past_eol || ctx.virtualedit.allows_past_eol() {
            // Can be at line_len (one past last char)
            pos.col.min(line_len)
        } else {
            // Must be before last char
            if line_len == 0 {
                0
            } else {
                pos.col.min(line_len.saturating_sub(1))
            }
        };

        // Handle coladd for virtualedit
        let coladd = if ctx.virtualedit.allows_anywhere() {
            pos.coladd
        } else {
            0
        };

        self.position = CursorPosition { line, col, coladd };

        Ok(())
    }

    fn set_line(&mut self, line: LineNr, ctx: &CursorContext) -> VimResult<()> {
        let pos = CursorPosition::new(line, self.position.col);
        self.set_position(pos, ctx)
    }

    fn set_col(&mut self, col: usize, ctx: &CursorContext) -> VimResult<()> {
        let pos = CursorPosition::new(self.position.line, col);
        self.set_position(pos, ctx)
    }

    fn set_curswant(&mut self, want: CursorWant) {
        self.curswant = want;
    }

    fn virtcol(&self) -> usize {
        // For now, treat virtual column as same as byte column
        // A full implementation would account for tabs and wide chars
        self.position.col + self.position.coladd
    }

    fn virtcol_at(&self, pos: CursorPosition) -> usize {
        pos.col + pos.coladd
    }

    fn virtcol_to_col(&self, _line: LineNr, vcol: usize) -> usize {
        // Simple implementation: virtual col = byte col
        vcol
    }

    fn check_cursor(&mut self, ctx: &CursorContext) {
        let max_line = self.line_count().max(1);
        self.check_cursor_lnum(LineNr(max_line));
        let line_len = self.line_len(self.position.line);
        self.check_cursor_col(line_len, ctx);
    }

    fn check_cursor_lnum(&mut self, max_line: LineNr) {
        if self.position.line.0 < 1 {
            self.position.line = LineNr(1);
        } else if self.position.line.0 > max_line.0 {
            self.position.line = max_line;
        }
    }

    fn check_cursor_col(&mut self, line_len: usize, ctx: &CursorContext) {
        if line_len == 0 {
            self.position.col = 0;
        } else if ctx.allow_past_eol || ctx.virtualedit.allows_past_eol() {
            if self.position.col > line_len {
                self.position.col = line_len;
            }
        } else {
            if self.position.col >= line_len {
                self.position.col = line_len.saturating_sub(1);
            }
        }
    }

    fn style(&self) -> CursorStyle {
        self.style
    }
}

impl CursorManager for TuiCursor {
    fn save(&self) -> CursorPosition {
        self.position
    }

    fn restore(&mut self, pos: CursorPosition, ctx: &CursorContext) {
        let _ = self.set_position(pos, ctx);
    }

    fn adjust_for_change(
        &mut self,
        change_line: LineNr,
        change_col: usize,
        deleted_bytes: usize,
        added_bytes: usize,
        ctx: &CursorContext,
    ) {
        // If change is on a different line, no adjustment needed
        if self.position.line != change_line {
            return;
        }

        // If cursor is before the change, no adjustment needed
        if self.position.col < change_col {
            return;
        }

        // Adjust column based on the change
        if self.position.col >= change_col + deleted_bytes {
            // Cursor is after the deleted region
            self.position.col = self.position.col - deleted_bytes + added_bytes;
        } else {
            // Cursor is within the deleted region - move to change_col
            self.position.col = change_col + added_bytes;
        }

        // Validate the new position
        self.check_cursor(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vxd::cursor::behavior::CursorBehaviorTests;
    use vxd::cursor::VirtualEdit;

    impl CursorBehaviorTests for TuiCursor {
        fn test_line_len(&self) -> usize {
            self.line_len(self.position.line)
        }
    }

    fn with_cursor(mut f: impl FnMut(&mut TuiCursor)) {
        let mut cursor = TuiCursor::new();
        cursor.set_line_lengths(vec![5]);
        f(&mut cursor);
    }

    fn normal_ctx() -> CursorContext {
        CursorContext {
            allow_past_eol: false,
            virtualedit: VirtualEdit::None,
            visual_selection: false,
        }
    }

    fn insert_ctx() -> CursorContext {
        CursorContext {
            allow_past_eol: true,
            virtualedit: VirtualEdit::None,
            visual_selection: false,
        }
    }

    #[test]
    fn test_line_clamping() {
        let mut cursor = TuiCursor::new();
        cursor.set_line_lengths(vec![5, 5, 5]); // 3 lines

        let ctx = normal_ctx();

        // Line 0 should clamp to 1
        cursor
            .set_position(CursorPosition::new(LineNr(0), 0), &ctx)
            .ok();
        assert!(cursor.line().0 >= 1);

        // Line past max should clamp
        cursor
            .set_position(CursorPosition::new(LineNr(100), 0), &ctx)
            .ok();
        assert_eq!(cursor.line().0, 3);
    }

    #[test]
    fn test_normal_mode_col_clamping() {
        let mut cursor = TuiCursor::new();
        cursor.set_line_lengths(vec![5]); // "hello" = 5 chars

        let ctx = normal_ctx();

        // Try to set column past end
        cursor
            .set_position(CursorPosition::new(LineNr(1), 10), &ctx)
            .ok();

        // Should be clamped to last char (index 4)
        assert_eq!(cursor.col(), 4);
    }

    #[test]
    fn test_insert_mode_past_eol() {
        let mut cursor = TuiCursor::new();
        cursor.set_line_lengths(vec![5]); // "hello" = 5 chars

        let ctx = insert_ctx();

        // Set column at end (append position)
        cursor
            .set_position(CursorPosition::new(LineNr(1), 5), &ctx)
            .ok();

        // Should be allowed at 5 (one past last char)
        assert_eq!(cursor.col(), 5);
    }

    #[test]
    fn test_curswant_preserved() {
        let mut cursor = TuiCursor::new();
        cursor.set_line_lengths(vec![20]);

        let ctx = normal_ctx();
        cursor.set_col(10, &ctx).ok();
        cursor.update_curswant();

        assert_eq!(cursor.curswant(), CursorWant::Column(10));
    }

    #[test]
    fn test_dollar_sets_maxcol() {
        let mut cursor = TuiCursor::new();
        cursor.set_curswant_eol();

        assert_eq!(cursor.curswant(), CursorWant::EndOfLine);
        assert_eq!(cursor.curswant().value(), CursorPosition::MAXCOL);
    }

    #[test]
    fn test_empty_line_col_zero() {
        let mut cursor = TuiCursor::new();
        cursor.set_line_lengths(vec![0]); // Empty line

        let ctx = normal_ctx();
        cursor.check_cursor_col(0, &ctx);

        assert_eq!(cursor.col(), 0);
    }

    #[test]
    fn test_cursor_behavior_contracts() {
        with_cursor(|cursor| cursor.test_line_clamping(LineNr(1)));
        with_cursor(|cursor| cursor.test_normal_mode_col_clamping());
        with_cursor(|cursor| cursor.test_insert_mode_past_eol());
        with_cursor(|cursor| cursor.test_virtualedit_coladd());
        with_cursor(|cursor| cursor.test_curswant_preserved());
        with_cursor(|cursor| cursor.test_dollar_sets_maxcol());
        with_cursor(|cursor| cursor.test_empty_line_col_zero());
    }
}
