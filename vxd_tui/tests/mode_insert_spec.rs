//! Insert mode tests ported from Neovim's test/functional/editor/mode_insert_spec.lua
//!
//! These tests verify insert mode behavior including:
//! - Entering and exiting insert mode
//! - Character insertion
//! - Backspace and delete
//! - Newline insertion
//! - Ctrl-O for single normal mode command

#![allow(non_snake_case)]

mod common;

use common::TestHarness;
use vxd::modes::Mode;

/// Test: entering insert mode with 'i'
/// Source: mode_insert_spec.lua (basic insert mode entry)
#[test]
fn test_enter_insert_mode_i() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);

    assert!(matches!(h.mode(), Mode::Normal));

    h.feed("i");
    assert!(matches!(h.mode(), Mode::Insert));
}

/// Test: exiting insert mode with Escape
/// Source: mode_insert_spec.lua (basic insert mode exit)
#[test]
fn test_exit_insert_mode_escape() {
    let mut h = TestHarness::new();

    h.feed("i");
    assert!(matches!(h.mode(), Mode::Insert));

    h.feed("<Esc>");
    assert!(matches!(h.mode(), Mode::Normal));
}

/// Test: inserting characters
/// Source: mode_insert_spec.lua (character insertion)
#[test]
fn test_insert_characters() {
    let mut h = TestHarness::new();

    h.feed("ihello<Esc>");
    assert_eq!(h.get_lines(), vec!["hello"]);
}

/// Test: inserting at cursor position
/// Source: mode_insert_spec.lua line 45-48 (CTRL-@)
#[test]
fn test_insert_at_cursor() {
    let mut h = TestHarness::new();
    h.set_lines(&["world"]);
    h.set_cursor(1, 0);

    h.feed("ihello <Esc>");
    assert_eq!(h.get_lines(), vec!["hello world"]);
}

/// Test: append with 'a'
/// Source: basic Vim behavior
#[test]
fn test_append_with_a() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 2); // Cursor on 'l'

    h.feed("aX<Esc>");
    // 'a' moves cursor right then inserts
    let content = h.content();
    assert!(content.contains("X"));
}

/// Test: insert at beginning of line with 'I'
/// Source: basic Vim behavior
#[test]
fn test_insert_at_line_beginning_I() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 3); // Middle of line

    h.feed("Istart <Esc>");
    assert_eq!(h.get_lines(), vec!["start hello"]);
}

/// Test: append at end of line with 'A'
/// Source: basic Vim behavior
#[test]
fn test_append_at_line_end_A() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 0);

    h.feed("A world<Esc>");
    assert_eq!(h.get_lines(), vec!["hello world"]);
}

/// Test: open line below with 'o'
/// Source: basic Vim behavior
#[test]
fn test_open_line_below_o() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line3"]);
    h.set_cursor(1, 0);

    h.feed("oline2<Esc>");
    assert_eq!(h.get_lines(), vec!["line1", "line2", "line3"]);
}

/// Test: open line above with 'O'
/// Source: basic Vim behavior
#[test]
fn test_open_line_above_O() {
    let mut h = TestHarness::new();
    h.set_lines(&["line2", "line3"]);
    h.set_cursor(1, 0);

    h.feed("Oline1<Esc>");
    assert_eq!(h.get_lines(), vec!["line1", "line2", "line3"]);
}

/// Test: backspace deletes character
/// Source: mode_insert_spec.lua (backspace behavior)
#[test]
fn test_backspace_deletes_char() {
    let mut h = TestHarness::new();

    h.feed("ihello<BS><Esc>");
    assert_eq!(h.get_lines(), vec!["hell"]);
}

/// Test: multiple backspaces
/// Source: mode_insert_spec.lua
#[test]
fn test_multiple_backspaces() {
    let mut h = TestHarness::new();

    h.feed("ihello<BS><BS><BS><Esc>");
    assert_eq!(h.get_lines(), vec!["he"]);
}

/// Test: backspace at beginning of line doesn't delete
/// Source: Vim behavior (or joins with previous line depending on settings)
#[test]
fn test_backspace_at_line_beginning() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 0);

    h.feed("i<BS><Esc>");
    // At column 0, backspace should do nothing or join with prev line
    // In our simple implementation, it does nothing
    assert_eq!(h.get_lines(), vec!["hello"]);
}

/// Test: Enter creates new line
/// Source: mode_insert_spec.lua (newline insertion)
#[test]
fn test_enter_creates_newline() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello world"]);
    h.set_cursor(1, 5);

    h.feed("i<CR><Esc>");
    assert_eq!(h.get_lines(), vec!["hello", " world"]);
}

/// Test: Enter at end of line
/// Source: mode_insert_spec.lua
#[test]
fn test_enter_at_end_of_line() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);

    h.feed("A<CR>world<Esc>");
    assert_eq!(h.get_lines(), vec!["hello", "world"]);
}

/// Test: Enter at beginning of line
/// Source: mode_insert_spec.lua
#[test]
fn test_enter_at_beginning_of_line() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 0);

    h.feed("i<CR><Esc>");
    assert_eq!(h.get_lines(), vec!["", "hello"]);
}

/// Test: cursor movement in insert mode with arrow keys
/// Source: mode_insert_spec.lua (arrow key movement)
#[test]
fn test_arrow_keys_in_insert_mode() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello", "world"]);
    h.set_cursor(1, 2);

    h.feed("i");
    assert!(matches!(h.mode(), Mode::Insert));

    // Move right
    h.feed("<Right>");
    let (_, col) = h.cursor();
    assert_eq!(col, 3);

    // Move left
    h.feed("<Left>");
    let (_, col) = h.cursor();
    assert_eq!(col, 2);

    h.feed("<Esc>");
}

/// Test: cursor movement in insert mode with up/down
/// Source: mode_insert_spec.lua
#[test]
fn test_up_down_in_insert_mode() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello", "world", "test"]);
    h.set_cursor(2, 2);

    h.feed("i");

    // Move up
    h.feed("<Up>");
    let (line, _) = h.cursor();
    assert_eq!(line, 1);

    // Move down twice
    h.feed("<Down><Down>");
    let (line, _) = h.cursor();
    assert_eq!(line, 3);

    h.feed("<Esc>");
}

/// Test: typing replaces nothing in insert mode (not replace mode)
/// Source: basic Vim behavior
#[test]
fn test_insert_mode_does_not_replace() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 2);

    h.feed("iX<Esc>");
    // Should insert X, not replace l
    assert_eq!(h.get_lines(), vec!["heXllo"]);
}

/// Test: cursor position after exiting insert mode
/// Source: Vim behavior - cursor moves back one position on Esc
#[test]
fn test_cursor_after_escape() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);

    h.feed("A"); // Append at end
    let (_, col) = h.cursor();
    let line_len = h.editor.current_line().len();
    assert_eq!(col, line_len); // Past the 'o'

    h.feed("<Esc>");
    // In normal mode, cursor should be on last char, not past it
    let (_, col) = h.cursor();
    assert!(col < line_len || line_len == 0);
}

/// Test: multiple inserts accumulate
/// Source: mode_insert_spec.lua
#[test]
fn test_multiple_inserts() {
    let mut h = TestHarness::new();

    h.feed("ihello<Esc>");
    h.feed("A world<Esc>");
    h.feed("I>>> <Esc>");

    assert_eq!(h.get_lines(), vec![">>> hello world"]);
}

/// Test: insert empty string does nothing
/// Source: edge case
#[test]
fn test_insert_then_immediate_escape() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);

    h.feed("i<Esc>");
    assert_eq!(h.get_lines(), vec!["hello"]);
}

/// Test: insert special characters
/// Source: mode_insert_spec.lua (multi-byte text)
#[test]
fn test_insert_special_characters() {
    let mut h = TestHarness::new();

    h.feed("i!@#$%<Esc>");
    assert_eq!(h.get_lines(), vec!["!@#$%"]);
}

/// Test: insert unicode characters
/// Source: mode_insert_spec.lua line 60 (multi-byte text test)
#[test]
fn test_insert_unicode() {
    let mut h = TestHarness::new();

    h.feed("iåäö<Esc>");
    assert_eq!(h.get_lines(), vec!["åäö"]);
}

/// Test: insert with count (not implemented in simple version)
/// Source: Vim behavior - 3ihello<Esc> inserts "hellohellohello"
/// Note: This may not be implemented yet
#[test]
#[ignore = "count prefix not yet implemented"]
fn test_insert_with_count() {
    let mut h = TestHarness::new();

    h.feed("3ihello<Esc>");
    assert_eq!(h.get_lines(), vec!["hellohellohello"]);
}

/// Test: 'o' at end of buffer
/// Source: edge case
#[test]
fn test_open_line_at_end_of_buffer() {
    let mut h = TestHarness::new();
    h.set_lines(&["only line"]);
    h.set_cursor(1, 0);

    h.feed("onew line<Esc>");
    assert_eq!(h.get_lines(), vec!["only line", "new line"]);
}

/// Test: 'O' at beginning of buffer
/// Source: edge case
#[test]
fn test_open_line_at_beginning_of_buffer() {
    let mut h = TestHarness::new();
    h.set_lines(&["only line"]);
    h.set_cursor(1, 0);

    h.feed("Onew first<Esc>");
    assert_eq!(h.get_lines(), vec!["new first", "only line"]);
}

/// Test: repeated Enter creates multiple blank lines
/// Source: basic Vim behavior
#[test]
fn test_multiple_enters() {
    let mut h = TestHarness::new();
    h.set_lines(&["start"]);

    h.feed("A<CR><CR><CR>end<Esc>");
    assert_eq!(h.get_lines(), vec!["start", "", "", "end"]);
}

/// Test: insert mode preserves undo points
/// Source: mode_insert_spec.lua (undo behavior with insert)
/// Note: This requires undo implementation
#[test]
#[ignore = "undo not yet fully implemented"]
fn test_insert_creates_undo_point() {
    let mut h = TestHarness::new();
    h.set_lines(&["original"]);

    h.feed("Atext<Esc>");
    // Would need undo command: h.feed("u");
    // assert_eq!(h.get_lines(), vec!["original"]);
}
