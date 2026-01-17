//! Editor - the main entry point combining all components.
//!
//! This module provides the `Editor` struct which combines buffer, cursor,
//! mode, register, and mark management into a cohesive editor.

use crate::buffer::TuiBufferManager;
use crate::cursor::TuiCursor;
use crate::marks::TuiMarkManager;
use crate::modes::TuiModeManager;
use crate::registers::TuiRegisterBank;

use vxd::buffer::{Buffer, BufferManager};
use vxd::cursor::{Cursor, CursorContext, CursorPosition, VirtualEdit};
use vxd::marks::MarkManager;
use vxd::modes::{Mode, ModeManager};
use vxd::motions::CharFindMotion;
use vxd::types::{LineNr, VimError, VimResult};

/// The main editor struct combining all components
#[derive(Debug)]
pub struct Editor {
    /// Buffer manager
    pub buffers: TuiBufferManager,
    /// Cursor for current window
    pub cursor: TuiCursor,
    /// Mode manager
    pub modes: TuiModeManager,
    /// Register bank
    pub registers: TuiRegisterBank,
    /// Mark manager
    pub marks: TuiMarkManager,
    last_char_find: Option<CharFindMotion>,
    current_insert: Option<String>,
}

impl Editor {
    /// Create a new editor instance
    pub fn new() -> Self {
        let mut editor = Editor {
            buffers: TuiBufferManager::new(),
            cursor: TuiCursor::new(),
            modes: TuiModeManager::new(),
            registers: TuiRegisterBank::new(),
            marks: TuiMarkManager::new(),
            last_char_find: None,
            current_insert: None,
        };

        // Sync cursor with initial buffer
        editor.sync_cursor_with_buffer();

        editor
    }

    /// Sync cursor line lengths with current buffer
    pub fn sync_cursor_with_buffer(&mut self) {
        let lines = self
            .buffers
            .current()
            .get_lines(0, -1, false)
            .unwrap_or_default();
        self.cursor.update_line_lengths(&lines);
        let pos = self.cursor.position();
        let ctx = self.cursor_context();
        let _ = self.cursor.set_position(pos, &ctx);
    }

    /// Get cursor context based on current mode
    pub fn cursor_context(&self) -> CursorContext {
        CursorContext {
            allow_past_eol: self.modes.mode().allows_cursor_past_eol(),
            virtualedit: VirtualEdit::None, // TODO: Get from options
            visual_selection: self.modes.mode().is_visual(),
        }
    }

    /// Get current line content
    pub fn current_line(&self) -> String {
        self.buffers
            .current()
            .get_line(self.cursor.line().0 as i64 - 1)
            .unwrap_or_default()
    }

    /// Get the current mode
    pub fn mode(&self) -> Mode {
        self.modes.mode()
    }

    /// Enter insert mode
    pub fn enter_insert(&mut self) -> VimResult<()> {
        self.modes
            .enter_insert()
            .map(|_| ())
            .map_err(|err| VimError::NotAllowedInMode(err.reason))?;
        self.current_insert = Some(String::new());
        Ok(())
    }

    /// Enter replace mode
    pub fn enter_replace(&mut self) -> VimResult<()> {
        self.modes
            .enter_replace()
            .map(|_| ())
            .map_err(|err| VimError::NotAllowedInMode(err.reason))?;
        self.current_insert = Some(String::new());
        Ok(())
    }

    /// Enter normal mode (escape)
    pub fn escape(&mut self) -> VimResult<()> {
        self.modes
            .escape_to_normal()
            .map(|_| ())
            .map_err(|err| VimError::NotAllowedInMode(err.reason))?;
        if let Some(inserted) = self.current_insert.take() {
            if !inserted.is_empty() {
                self.registers
                    .set_last_inserted(vxd::registers::RegisterContent::characterwise(inserted));
            }
        }
        // In normal mode, cursor can't be past EOL
        let ctx = self.cursor_context();
        self.cursor.check_cursor(&ctx);
        Ok(())
    }

    /// Move cursor down
    pub fn cursor_down(&mut self, count: usize) -> VimResult<()> {
        let ctx = self.cursor_context();
        let new_line =
            LineNr((self.cursor.line().0 + count).min(self.buffers.current().line_count()));
        let want_col = self.cursor.curswant().value();
        self.cursor
            .set_position(CursorPosition::new(new_line, want_col), &ctx)?;
        self.sync_cursor_with_buffer();
        Ok(())
    }

    /// Move cursor up
    pub fn cursor_up(&mut self, count: usize) -> VimResult<()> {
        let ctx = self.cursor_context();
        let new_line = LineNr(self.cursor.line().0.saturating_sub(count).max(1));
        let want_col = self.cursor.curswant().value();
        self.cursor
            .set_position(CursorPosition::new(new_line, want_col), &ctx)?;
        self.sync_cursor_with_buffer();
        Ok(())
    }

    /// Move cursor left
    pub fn cursor_left(&mut self, count: usize) -> VimResult<()> {
        let ctx = self.cursor_context();
        let new_col = self.cursor.col().saturating_sub(count);
        self.cursor.set_col(new_col, &ctx)?;
        self.cursor.update_curswant();
        Ok(())
    }

    /// Move cursor right
    pub fn cursor_right(&mut self, count: usize) -> VimResult<()> {
        let ctx = self.cursor_context();
        let new_col = self.cursor.col() + count;
        self.cursor.set_col(new_col, &ctx)?;
        self.cursor.update_curswant();
        Ok(())
    }

    /// Insert text at cursor position
    pub fn insert_char(&mut self, c: char) -> VimResult<()> {
        if !self.modes.mode().allows_insertion() {
            return Ok(()); // Ignore in non-insert modes
        }

        let line_idx = self.cursor.line().0 as i64 - 1;
        let col = self.cursor.col();

        // Get current line
        let current_line = self.current_line();

        let new_line = if self.modes.mode() == Mode::Replace && col < current_line.len() {
            let char_end = col
                + current_line[col..]
                    .chars()
                    .next()
                    .map(|ch| ch.len_utf8())
                    .unwrap_or(1);
            format!("{}{}{}", &current_line[..col], c, &current_line[char_end..])
        } else {
            let (before, after) = if col <= current_line.len() {
                (&current_line[..col], &current_line[col..])
            } else {
                (current_line.as_str(), "")
            };
            format!("{}{}{}", before, c, after)
        };
        self.buffers
            .current_mut()
            .set_lines(line_idx, line_idx + 1, false, vec![new_line])?;

        // Move cursor right
        self.sync_cursor_with_buffer();
        let ctx = self.cursor_context();
        self.cursor.set_col(col + c.len_utf8(), &ctx)?;

        // Record change
        self.marks.record_change(self.cursor.position());
        if let Some(inserted) = self.current_insert.as_mut() {
            inserted.push(c);
        }

        Ok(())
    }

    /// Insert a character from a nearby line (Ctrl-Y/Ctrl-E behavior).
    pub fn insert_from_adjacent_line(&mut self, line_offset: i64) -> VimResult<()> {
        if !self.modes.mode().allows_insertion() {
            return Ok(());
        }

        let line_count = self.buffers.current().line_count() as i64;
        let current_idx = self.cursor.line().0 as i64 - 1;
        let target_idx = current_idx + line_offset;
        if target_idx < 0 || target_idx >= line_count {
            return Ok(());
        }

        let source_line = self
            .buffers
            .current()
            .get_line(target_idx)
            .unwrap_or_default();
        let col = self.cursor.col();
        if col >= source_line.len() {
            return Ok(());
        }

        if let Some(ch) = source_line[col..].chars().next() {
            self.insert_char(ch)?;
        }

        Ok(())
    }

    /// Delete character at cursor position (like 'x' command)
    pub fn delete_char(&mut self) -> VimResult<()> {
        let line_idx = self.cursor.line().0 as i64 - 1;
        let col = self.cursor.col();
        let current_line = self.current_line();

        if col >= current_line.len() {
            return Ok(()); // Nothing to delete
        }

        // Find character boundary
        let char_end = col
            + current_line[col..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);

        // Build new line
        let new_line = format!("{}{}", &current_line[..col], &current_line[char_end..]);
        self.buffers
            .current_mut()
            .set_lines(line_idx, line_idx + 1, false, vec![new_line])?;

        self.sync_cursor_with_buffer();
        let ctx = self.cursor_context();
        self.cursor.check_cursor(&ctx);

        // Record change
        self.marks.record_change(self.cursor.position());

        Ok(())
    }

    /// Insert a new line (like pressing Enter in insert mode)
    pub fn insert_newline(&mut self) -> VimResult<()> {
        if !self.modes.mode().allows_insertion() {
            return Ok(());
        }

        let line_idx = self.cursor.line().0 as i64 - 1;
        let col = self.cursor.col();
        let current_line = self.current_line();

        // Split line at cursor
        let (before, after) = if col <= current_line.len() {
            (&current_line[..col], &current_line[col..])
        } else {
            (current_line.as_str(), "")
        };

        self.buffers.current_mut().set_lines(
            line_idx,
            line_idx + 1,
            false,
            vec![before.to_string(), after.to_string()],
        )?;

        // Move cursor to beginning of new line
        self.sync_cursor_with_buffer();
        let ctx = self.cursor_context();
        self.cursor.set_position(
            vxd::cursor::CursorPosition::new(LineNr(self.cursor.line().0 + 1), 0),
            &ctx,
        )?;
        if let Some(inserted) = self.current_insert.as_mut() {
            inserted.push('\n');
        }

        Ok(())
    }

    pub fn insert_text(&mut self, text: &str) -> VimResult<()> {
        for ch in text.chars() {
            if ch == '\n' {
                self.insert_newline()?;
            } else {
                self.insert_char(ch)?;
            }
        }
        Ok(())
    }

    pub fn find_char(&mut self, motion: CharFindMotion) -> VimResult<bool> {
        let actual_motion = match motion {
            CharFindMotion::RepeatForward => match self.last_char_find {
                Some(last) => last,
                None => return Ok(false),
            },
            CharFindMotion::RepeatBackward => match self.last_char_find {
                Some(last) => invert_char_find(last),
                None => return Ok(false),
            },
            other => other,
        };

        let line = self.current_line();
        let col = self.cursor.col();
        let target_col = match char_find_target(&line, col, actual_motion) {
            Some(idx) => idx,
            None => return Ok(false),
        };

        let ctx = self.cursor_context();
        self.cursor.set_col(target_col, &ctx)?;
        self.cursor.update_curswant();
        self.last_char_find = Some(actual_motion);
        Ok(true)
    }

    pub fn match_bracket(&mut self) -> VimResult<bool> {
        let line = self.current_line();
        let col = self.cursor.col();
        let ch = match char_at(&line, col) {
            Some(ch) => ch,
            None => return Ok(false),
        };
        let (open, close, forward) = match ch {
            '(' => ('(', ')', true),
            '[' => ('[', ']', true),
            '{' => ('{', '}', true),
            ')' => ('(', ')', false),
            ']' => ('[', ']', false),
            '}' => ('{', '}', false),
            _ => return Ok(false),
        };

        let target_col = if forward {
            find_matching_forward(&line, col, open, close)
        } else {
            find_matching_backward(&line, col, open, close)
        };

        if let Some(target) = target_col {
            let ctx = self.cursor_context();
            self.cursor.set_col(target, &ctx)?;
            self.cursor.update_curswant();
            return Ok(true);
        }

        Ok(false)
    }
}

fn invert_char_find(motion: CharFindMotion) -> CharFindMotion {
    match motion {
        CharFindMotion::FindForward(c) => CharFindMotion::FindBackward(c),
        CharFindMotion::FindBackward(c) => CharFindMotion::FindForward(c),
        CharFindMotion::TillForward(c) => CharFindMotion::TillBackward(c),
        CharFindMotion::TillBackward(c) => CharFindMotion::TillForward(c),
        other => other,
    }
}

fn char_find_target(line: &str, col: usize, motion: CharFindMotion) -> Option<usize> {
    match motion {
        CharFindMotion::FindForward(target) => {
            let start = next_char_index(line, col)?;
            find_forward(line, start, target)
        }
        CharFindMotion::FindBackward(target) => find_backward(line, col, target),
        CharFindMotion::TillForward(target) => {
            let start = next_char_index(line, col)?;
            let idx = find_forward(line, start, target)?;
            previous_char_index(line, idx)
        }
        CharFindMotion::TillBackward(target) => {
            let idx = find_backward(line, col, target)?;
            let after = next_char_index(line, idx)?;
            if after >= line.len() {
                None
            } else {
                Some(after)
            }
        }
        CharFindMotion::RepeatForward | CharFindMotion::RepeatBackward => None,
    }
}

fn find_forward(line: &str, start: usize, target: char) -> Option<usize> {
    for (idx, ch) in line.char_indices() {
        if idx < start {
            continue;
        }
        if ch == target {
            return Some(idx);
        }
    }
    None
}

fn find_backward(line: &str, before: usize, target: char) -> Option<usize> {
    let mut found = None;
    for (idx, ch) in line.char_indices() {
        if idx >= before {
            break;
        }
        if ch == target {
            found = Some(idx);
        }
    }
    found
}

fn next_char_index(line: &str, idx: usize) -> Option<usize> {
    let ch = line.get(idx..)?.chars().next()?;
    Some(idx + ch.len_utf8())
}

fn previous_char_index(line: &str, idx: usize) -> Option<usize> {
    let mut prev = None;
    for (pos, _) in line.char_indices() {
        if pos >= idx {
            break;
        }
        prev = Some(pos);
    }
    prev
}

fn char_at(line: &str, col: usize) -> Option<char> {
    line.get(col..)?.chars().next()
}

fn find_matching_forward(line: &str, col: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in line.char_indices() {
        if idx < col {
            continue;
        }
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

fn find_matching_backward(line: &str, col: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in line.char_indices() {
        if idx > col {
            break;
        }
        if ch == close {
            depth += 1;
        } else if ch == open {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = Editor::new();
        assert_eq!(editor.mode(), Mode::Normal);
        assert_eq!(editor.buffers.list().len(), 1);
    }

    #[test]
    fn test_mode_switching() {
        let mut editor = Editor::new();

        assert_eq!(editor.mode(), Mode::Normal);

        editor.enter_insert().unwrap();
        assert_eq!(editor.mode(), Mode::Insert);

        editor.escape().unwrap();
        assert_eq!(editor.mode(), Mode::Normal);
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = Editor::new();

        // Add some content
        editor
            .buffers
            .current_mut()
            .set_lines(
                0,
                -1,
                false,
                vec!["line 1".into(), "line 2".into(), "line 3".into()],
            )
            .unwrap();
        editor.sync_cursor_with_buffer();

        // Move down
        editor.cursor_down(1).unwrap();
        assert_eq!(editor.cursor.line(), LineNr(2));

        // Move down again
        editor.cursor_down(1).unwrap();
        assert_eq!(editor.cursor.line(), LineNr(3));

        // Move up
        editor.cursor_up(1).unwrap();
        assert_eq!(editor.cursor.line(), LineNr(2));

        // Move right
        editor.cursor_right(3).unwrap();
        assert_eq!(editor.cursor.col(), 3);

        // Move left
        editor.cursor_left(1).unwrap();
        assert_eq!(editor.cursor.col(), 2);
    }

    #[test]
    fn test_insert_char() {
        let mut editor = Editor::new();

        // Enter insert mode
        editor.enter_insert().unwrap();

        // Insert characters
        editor.insert_char('h').unwrap();
        editor.insert_char('i').unwrap();

        // Check buffer content
        let lines = editor.buffers.current().get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["hi"]);
    }

    #[test]
    fn test_delete_char() {
        let mut editor = Editor::new();

        // Set content
        editor
            .buffers
            .current_mut()
            .set_lines(0, -1, false, vec!["hello".into()])
            .unwrap();
        editor.sync_cursor_with_buffer();

        // Delete first char
        editor.delete_char().unwrap();

        let lines = editor.buffers.current().get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["ello"]);
    }

    #[test]
    fn test_insert_newline() {
        let mut editor = Editor::new();

        // Set content and position cursor in middle
        editor
            .buffers
            .current_mut()
            .set_lines(0, -1, false, vec!["hello world".into()])
            .unwrap();
        editor.sync_cursor_with_buffer();

        // Enter insert mode and position cursor
        editor.enter_insert().unwrap();
        let ctx = editor.cursor_context();
        editor.cursor.set_col(5, &ctx).unwrap();

        // Insert newline
        editor.insert_newline().unwrap();

        let lines = editor.buffers.current().get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["hello", " world"]);
        assert_eq!(editor.cursor.line(), LineNr(2));
    }
}
