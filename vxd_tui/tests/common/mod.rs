//! Common test utilities for vxd_tui integration tests.
//!
//! This module provides a test harness similar to Neovim's functional testing framework,
//! allowing tests to simulate user input and verify buffer/cursor state.

use vxd::buffer::{Buffer, BufferManager};
use vxd::cursor::Cursor;
use vxd::modes::Mode;
use vxd::motions::CharFindMotion;
use vxd::types::LineNr;
use vxd_tui::editor::Editor;

/// Test harness that wraps an Editor with convenient test methods.
pub struct TestHarness {
    pub editor: Editor,
    pending_g: bool,
    pending_char_find: Option<PendingCharFind>,
}

#[allow(dead_code)]
impl TestHarness {
    /// Create a new test harness with a fresh editor.
    pub fn new() -> Self {
        TestHarness {
            editor: Editor::new(),
            pending_g: false,
            pending_char_find: None,
        }
    }

    /// Create a test harness with initial buffer content.
    pub fn with_lines(lines: &[&str]) -> Self {
        let mut harness = Self::new();
        harness.set_lines(lines);
        harness
    }

    /// Set buffer lines (replaces all content).
    pub fn set_lines(&mut self, lines: &[&str]) {
        let lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        self.editor
            .buffers
            .current_mut()
            .set_lines(0, -1, false, lines)
            .unwrap();
        self.editor.sync_cursor_with_buffer();
    }

    /// Get all buffer lines.
    pub fn get_lines(&self) -> Vec<String> {
        self.editor
            .buffers
            .current()
            .get_lines(0, -1, false)
            .unwrap_or_default()
    }

    /// Get buffer content as a single string.
    pub fn content(&self) -> String {
        self.get_lines().join("\n")
    }

    /// Get current cursor position as (line, col) - 1-indexed line, 0-indexed col.
    pub fn cursor(&self) -> (usize, usize) {
        (self.editor.cursor.line().0, self.editor.cursor.col())
    }

    /// Set cursor position (1-indexed line, 0-indexed col).
    pub fn set_cursor(&mut self, line: usize, col: usize) {
        let ctx = self.editor.cursor_context();
        self.editor.cursor.set_line(LineNr(line), &ctx).ok();
        self.editor.cursor.set_col(col, &ctx).ok();
        self.editor.cursor.update_curswant();
    }

    /// Set lines and adjust the cursor like nvim_buf_set_lines would.
    pub fn set_lines_range(
        &mut self,
        start: i64,
        end: i64,
        strict_indexing: bool,
        replacement: Vec<String>,
    ) {
        let cursor_line = self.editor.cursor.line().0;
        let cursor_col = self.editor.cursor.col();
        let line_count = self.editor.buffers.current().line_count();
        let (start_idx, end_idx) = normalize_line_range(start, end, line_count);
        let removed = end_idx.saturating_sub(start_idx);
        let added = replacement.len();
        let delta = added as i64 - removed as i64;

        self.editor
            .buffers
            .current_mut()
            .set_lines(start, end, strict_indexing, replacement)
            .unwrap();
        self.editor.sync_cursor_with_buffer();

        let cursor_zero = cursor_line.saturating_sub(1);
        let mut new_line = cursor_line as i64;
        if start_idx < cursor_zero {
            new_line = (cursor_line as i64 + delta).max(1);
        }
        let new_line = LineNr(new_line.max(1) as usize);
        let ctx = self.editor.cursor_context();
        let _ = self
            .editor
            .cursor
            .set_position(vxd::cursor::CursorPosition::new(new_line, cursor_col), &ctx);
        self.editor.cursor.update_curswant();
    }

    /// Get current mode.
    pub fn mode(&self) -> Mode {
        self.editor.mode()
    }

    /// Feed a sequence of keys to the editor.
    /// Supports Vim-style key notation like <Esc>, <CR>, <C-o>, etc.
    pub fn feed(&mut self, keys: &str) {
        let parsed = parse_keys(keys);
        for key in parsed {
            self.process_key(key);
        }
    }

    /// Process a single key.
    fn process_key(&mut self, key: Key) {
        match self.editor.mode() {
            Mode::Normal => self.process_normal_key(key),
            Mode::Insert | Mode::Replace => self.process_insert_key(key),
            Mode::Visual(_) => self.process_visual_key(key),
            _ => {}
        }
    }

    fn process_normal_key(&mut self, key: Key) {
        if self.pending_g && !matches!(key, Key::Char('g')) {
            self.pending_g = false;
        }
        if let Some(pending) = self.pending_char_find.take() {
            if let Key::Char(target) = key {
                let motion = match pending {
                    PendingCharFind::FindForward => CharFindMotion::FindForward(target),
                    PendingCharFind::FindBackward => CharFindMotion::FindBackward(target),
                    PendingCharFind::TillForward => CharFindMotion::TillForward(target),
                    PendingCharFind::TillBackward => CharFindMotion::TillBackward(target),
                };
                let _ = self.editor.find_char(motion);
            }
            return;
        }
        match key {
            Key::Char('i') => {
                let _ = self.editor.enter_insert();
            }
            Key::Char('a') => {
                let _ = self.editor.cursor_right(1);
                let _ = self.editor.enter_insert();
            }
            Key::Char('I') => {
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_col(0, &ctx);
                let _ = self.editor.enter_insert();
            }
            Key::Char('A') => {
                let line_len = self.editor.current_line().len();
                let _ = self.editor.enter_insert();
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_col(line_len, &ctx);
            }
            Key::Char('o') => {
                let line = self.editor.cursor.line().0 as i64;
                let _ = self.editor.buffers.current_mut().set_lines(
                    line,
                    line,
                    false,
                    vec!["".to_string()],
                );
                self.editor.sync_cursor_with_buffer();
                let _ = self.editor.cursor_down(1);
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_col(0, &ctx);
                let _ = self.editor.enter_insert();
            }
            Key::Char('O') => {
                let line = self.editor.cursor.line().0 as i64 - 1;
                let _ = self.editor.buffers.current_mut().set_lines(
                    line,
                    line,
                    false,
                    vec!["".to_string()],
                );
                self.editor.sync_cursor_with_buffer();
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_col(0, &ctx);
                let _ = self.editor.enter_insert();
            }
            Key::Char('h') | Key::Left => {
                let _ = self.editor.cursor_left(1);
            }
            Key::Char('j') | Key::Down => {
                let _ = self.editor.cursor_down(1);
            }
            Key::Char('k') | Key::Up => {
                let _ = self.editor.cursor_up(1);
            }
            Key::Char('l') | Key::Right => {
                let _ = self.editor.cursor_right(1);
            }
            Key::Char('0') => {
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_col(0, &ctx);
                self.editor.cursor.update_curswant();
            }
            Key::Char('$') => {
                let line_len = self.editor.current_line().len();
                let ctx = self.editor.cursor_context();
                let col = if line_len > 0 { line_len - 1 } else { 0 };
                let _ = self.editor.cursor.set_col(col, &ctx);
                self.editor.cursor.set_curswant_eol();
            }
            Key::Char('x') => {
                let _ = self.editor.delete_char();
            }
            Key::Char('G') => {
                let line_count = self.editor.buffers.current().line_count();
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_line(LineNr(line_count), &ctx);
            }
            Key::Char('g') => {
                if self.pending_g {
                    self.pending_g = false;
                    let ctx = self.editor.cursor_context();
                    let _ = self.editor.cursor.set_line(LineNr(1), &ctx);
                } else {
                    self.pending_g = true;
                }
            }
            Key::Char('f') => {
                self.pending_char_find = Some(PendingCharFind::FindForward);
            }
            Key::Char('F') => {
                self.pending_char_find = Some(PendingCharFind::FindBackward);
            }
            Key::Char('t') => {
                self.pending_char_find = Some(PendingCharFind::TillForward);
            }
            Key::Char('T') => {
                self.pending_char_find = Some(PendingCharFind::TillBackward);
            }
            Key::Char(';') => {
                let _ = self.editor.find_char(CharFindMotion::RepeatForward);
            }
            Key::Char(',') => {
                let _ = self.editor.find_char(CharFindMotion::RepeatBackward);
            }
            Key::Char('R') => {
                let _ = self.editor.enter_replace();
            }
            Key::Char('%') => {
                let _ = self.editor.match_bracket();
            }
            _ => {}
        }
    }

    fn process_insert_key(&mut self, key: Key) {
        match key {
            Key::Escape => {
                let _ = self.editor.escape();
            }
            Key::Char(c) => {
                let _ = self.editor.insert_char(c);
            }
            Key::Backspace => {
                let col = self.editor.cursor.col();
                if col > 0 {
                    let _ = self.editor.cursor_left(1);
                    let _ = self.editor.delete_char();
                }
            }
            Key::Enter => {
                let _ = self.editor.insert_newline();
            }
            Key::Left => {
                let _ = self.editor.cursor_left(1);
            }
            Key::Right => {
                let _ = self.editor.cursor_right(1);
            }
            Key::Up => {
                let _ = self.editor.cursor_up(1);
            }
            Key::Down => {
                let _ = self.editor.cursor_down(1);
            }
            _ => {}
        }
    }

    fn process_visual_key(&mut self, key: Key) {
        match key {
            Key::Escape => {
                let _ = self.editor.escape();
            }
            Key::Char('h') | Key::Left => {
                let _ = self.editor.cursor_left(1);
            }
            Key::Char('j') | Key::Down => {
                let _ = self.editor.cursor_down(1);
            }
            Key::Char('k') | Key::Up => {
                let _ = self.editor.cursor_up(1);
            }
            Key::Char('l') | Key::Right => {
                let _ = self.editor.cursor_right(1);
            }
            _ => {}
        }
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed key representation.
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Char(char),
    Escape,
    Enter,
    Backspace,
    Delete,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Ctrl(char),
    Alt(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingCharFind {
    FindForward,
    FindBackward,
    TillForward,
    TillBackward,
}

/// Parse a Vim-style key sequence into individual keys.
/// Supports: <Esc>, <CR>, <BS>, <Del>, <Tab>, <Left>, <Right>, <Up>, <Down>,
/// <Home>, <End>, <PageUp>, <PageDown>, <C-x>, <A-x>, <M-x>
pub fn parse_keys(input: &str) -> Vec<Key> {
    let mut keys = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            // Parse special key notation
            let mut special = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '>' {
                    chars.next();
                    break;
                }
                special.push(chars.next().unwrap());
            }

            let key = match special.to_lowercase().as_str() {
                "esc" | "escape" => Key::Escape,
                "cr" | "enter" | "return" => Key::Enter,
                "bs" | "backspace" => Key::Backspace,
                "del" | "delete" => Key::Delete,
                "tab" => Key::Tab,
                "left" => Key::Left,
                "right" => Key::Right,
                "up" => Key::Up,
                "down" => Key::Down,
                "home" => Key::Home,
                "end" => Key::End,
                "pageup" => Key::PageUp,
                "pagedown" => Key::PageDown,
                s if s.starts_with("c-") => {
                    let ch = s.chars().nth(2).unwrap_or('?');
                    Key::Ctrl(ch)
                }
                s if s.starts_with("a-") || s.starts_with("m-") => {
                    let ch = s.chars().nth(2).unwrap_or('?');
                    Key::Alt(ch)
                }
                _ => Key::Char('?'), // Unknown special key
            };
            keys.push(key);
        } else {
            keys.push(Key::Char(c));
        }
    }

    keys
}

fn normalize_line_range(start: i64, end: i64, line_count: usize) -> (usize, usize) {
    let len = line_count as i64;
    let start_idx = if start < 0 { len + start + 1 } else { start };
    let end_idx = if end == -1 { len } else if end < 0 { len + end + 1 } else { end };
    let start_idx = start_idx.clamp(0, len) as usize;
    let end_idx = end_idx.clamp(0, len) as usize;
    (start_idx, end_idx)
}

/// Assert that buffer content equals expected lines.
#[macro_export]
macro_rules! assert_lines {
    ($harness:expr, $($line:expr),* $(,)?) => {
        let expected: Vec<&str> = vec![$($line),*];
        let actual = $harness.get_lines();
        assert_eq!(actual, expected, "Buffer content mismatch");
    };
}

/// Assert cursor position (1-indexed line, 0-indexed col).
#[macro_export]
macro_rules! assert_cursor {
    ($harness:expr, $line:expr, $col:expr) => {
        let (actual_line, actual_col) = $harness.cursor();
        assert_eq!(
            (actual_line, actual_col),
            ($line, $col),
            "Cursor position mismatch: expected ({}, {}), got ({}, {})",
            $line,
            $col,
            actual_line,
            actual_col
        );
    };
}

/// Assert current mode.
#[macro_export]
macro_rules! assert_mode {
    ($harness:expr, $mode:pat) => {
        assert!(
            matches!($harness.mode(), $mode),
            "Mode mismatch: expected {}, got {:?}",
            stringify!($mode),
            $harness.mode()
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keys_simple() {
        let keys = parse_keys("abc");
        assert_eq!(keys, vec![Key::Char('a'), Key::Char('b'), Key::Char('c')]);
    }

    #[test]
    fn test_parse_keys_special() {
        let keys = parse_keys("<Esc>");
        assert_eq!(keys, vec![Key::Escape]);

        let keys = parse_keys("<CR>");
        assert_eq!(keys, vec![Key::Enter]);

        let keys = parse_keys("i<Esc>");
        assert_eq!(keys, vec![Key::Char('i'), Key::Escape]);
    }

    #[test]
    fn test_parse_keys_ctrl() {
        let keys = parse_keys("<C-o>");
        assert_eq!(keys, vec![Key::Ctrl('o')]);
    }

    #[test]
    fn test_harness_basic() {
        let mut h = TestHarness::new();
        h.set_lines(&["hello", "world"]);
        assert_eq!(h.get_lines(), vec!["hello", "world"]);
    }

    #[test]
    fn test_harness_feed() {
        let mut h = TestHarness::new();
        h.feed("ihello<Esc>");
        assert_eq!(h.get_lines(), vec!["hello"]);
        assert!(matches!(h.mode(), Mode::Normal));
    }
}
