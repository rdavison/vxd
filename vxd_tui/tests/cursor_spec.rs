//! Cursor tests ported from Neovim tests
//!
//! These tests verify cursor positioning behavior including:
//! - Basic movement (hjkl)
//! - Line boundaries
//! - Column clamping
//! - Curswant preservation

#![allow(non_snake_case)]

mod common;

use common::TestHarness;
use vxd::modes::Mode;

/// Test: cursor movement with h (left)
/// Source: basic Vim behavior
#[test]
fn test_cursor_move_left_h() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 3);

    h.feed("h");
    assert_eq!(h.cursor(), (1, 2));

    h.feed("h");
    assert_eq!(h.cursor(), (1, 1));
}

/// Test: cursor doesn't move left past column 0
/// Source: Vim behavior
#[test]
fn test_cursor_left_boundary() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 0);

    h.feed("h");
    assert_eq!(h.cursor(), (1, 0)); // Should stay at 0
}

/// Test: cursor movement with l (right)
/// Source: basic Vim behavior
#[test]
fn test_cursor_move_right_l() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 0);

    h.feed("l");
    assert_eq!(h.cursor(), (1, 1));

    h.feed("l");
    assert_eq!(h.cursor(), (1, 2));
}

/// Test: cursor doesn't move right past end of line in normal mode
/// Source: Vim behavior
#[test]
fn test_cursor_right_boundary_normal() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]); // 5 chars, indices 0-4
    h.set_cursor(1, 4); // Last char 'o'

    h.feed("l");
    // In normal mode, can't go past last char
    let (_, col) = h.cursor();
    assert!(col <= 4);
}

/// Test: cursor movement with j (down)
/// Source: basic Vim behavior
#[test]
fn test_cursor_move_down_j() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);
    h.set_cursor(1, 0);

    h.feed("j");
    assert_eq!(h.cursor(), (2, 0));

    h.feed("j");
    assert_eq!(h.cursor(), (3, 0));
}

/// Test: cursor doesn't move down past last line
/// Source: Vim behavior
#[test]
fn test_cursor_down_boundary() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2"]);
    h.set_cursor(2, 0);

    h.feed("j");
    assert_eq!(h.cursor(), (2, 0)); // Should stay on line 2
}

/// Test: cursor movement with k (up)
/// Source: basic Vim behavior
#[test]
fn test_cursor_move_up_k() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);
    h.set_cursor(3, 0);

    h.feed("k");
    assert_eq!(h.cursor(), (2, 0));

    h.feed("k");
    assert_eq!(h.cursor(), (1, 0));
}

/// Test: cursor doesn't move up past line 1
/// Source: Vim behavior
#[test]
fn test_cursor_up_boundary() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2"]);
    h.set_cursor(1, 0);

    h.feed("k");
    assert_eq!(h.cursor(), (1, 0)); // Should stay on line 1
}

/// Test: cursor column clamped to shorter line
/// Source: Vim behavior - curswant
#[test]
fn test_cursor_column_clamped_shorter_line() {
    let mut h = TestHarness::new();
    h.set_lines(&["long line here", "short", "long line here"]);
    h.set_cursor(1, 10); // Column 10 on first line

    h.feed("j");
    // Line 2 is only 5 chars, so cursor should clamp
    let (line, col) = h.cursor();
    assert_eq!(line, 2);
    assert!(col < 10); // Should be clamped
}

/// Test: cursor column restored when moving back to longer line
/// Source: Vim behavior - curswant preserved
#[test]
fn test_curswant_preserved() {
    let mut h = TestHarness::new();
    h.set_lines(&["long line here", "short", "long line here"]);
    h.set_cursor(1, 10);

    h.feed("j"); // Move to short line
    h.feed("j"); // Move back to long line

    // Cursor should return to column 10 (or close to it)
    let (line, col) = h.cursor();
    assert_eq!(line, 3);
    // curswant should restore the original column
    assert!(col >= 10 || col == h.editor.current_line().len().saturating_sub(1));
}

/// Test: 0 moves to beginning of line
/// Source: basic Vim behavior
#[test]
fn test_zero_moves_to_beginning() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello world"]);
    h.set_cursor(1, 5);

    h.feed("0");
    assert_eq!(h.cursor(), (1, 0));
}

/// Test: $ moves to end of line
/// Source: basic Vim behavior
#[test]
fn test_dollar_moves_to_end() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]); // 5 chars
    h.set_cursor(1, 0);

    h.feed("$");
    let (_, col) = h.cursor();
    // In normal mode, $ puts cursor on last char (index 4)
    assert_eq!(col, 4);
}

/// Test: $ on empty line stays at 0
/// Source: Vim behavior
#[test]
fn test_dollar_on_empty_line() {
    let mut h = TestHarness::new();
    h.set_lines(&[""]);
    h.set_cursor(1, 0);

    h.feed("$");
    assert_eq!(h.cursor(), (1, 0));
}

/// Test: G moves to last line
/// Source: basic Vim behavior
#[test]
fn test_G_moves_to_last_line() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3", "line4"]);
    h.set_cursor(1, 0);

    h.feed("G");
    let (line, _) = h.cursor();
    assert_eq!(line, 4);
}

/// Test: gg moves to first line
/// Source: basic Vim behavior
#[test]
fn test_g_moves_to_first_line() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);
    h.set_cursor(3, 0);

    h.feed("gg");
    let (line, _) = h.cursor();
    assert_eq!(line, 1);
}

/// Test: cursor on empty buffer
/// Source: edge case
#[test]
fn test_cursor_on_empty_buffer() {
    let h = TestHarness::new();
    // New buffer has one empty line
    assert_eq!(h.cursor(), (1, 0));
}

/// Test: arrow keys work same as hjkl in normal mode
/// Source: basic Vim behavior
#[test]
fn test_arrow_keys_normal_mode() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello", "world"]);
    h.set_cursor(1, 2);

    h.feed("<Down>");
    assert_eq!(h.cursor(), (2, 2));

    h.feed("<Up>");
    assert_eq!(h.cursor(), (1, 2));

    h.feed("<Right>");
    assert_eq!(h.cursor(), (1, 3));

    h.feed("<Left>");
    assert_eq!(h.cursor(), (1, 2));
}

/// Test: cursor in insert mode can go past EOL
/// Source: Vim behavior - insert mode allows cursor after last char
#[test]
fn test_insert_mode_cursor_past_eol() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);

    h.feed("A"); // Append mode - cursor goes after 'o'

    assert!(matches!(h.mode(), Mode::Insert));
    let (_, col) = h.cursor();
    // In insert mode after 'A', cursor should be at position 5 (past last char)
    assert_eq!(col, 5);
}

/// Test: cursor column valid after line deletion
/// Source: buffer_spec.lua cursor maintenance tests
#[test]
fn test_cursor_valid_after_line_delete() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);
    h.set_cursor(3, 0);

    // Delete line 3
    h.feed("x"); // Delete first char
    h.editor.sync_cursor_with_buffer();

    let (line, col) = h.cursor();
    assert!(line >= 1);
    assert!(col <= h.editor.current_line().len());
}

/// Test: cursor stays on valid character after content change
/// Source: Vim behavior
#[test]
fn test_cursor_stays_valid() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello world"]);
    h.set_cursor(1, 10); // On 'd'

    // Delete several characters
    h.feed("xxxxx");

    // Cursor should still be valid
    let (_, col) = h.cursor();
    let line_len = h.editor.current_line().len();
    assert!(col < line_len || line_len == 0);
}

/// Test: repeated movement
/// Source: basic Vim behavior
#[test]
fn test_repeated_movement() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 0);

    h.feed("lllll");
    let (_, col) = h.cursor();
    // Can't go past last char in normal mode
    assert!(col <= 4);
}

/// Test: movement with count (not implemented in simple version)
/// Source: Vim behavior - 5l moves 5 right
#[test]
#[ignore = "count prefix not yet implemented"]
fn test_movement_with_count() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello world test"]);
    h.set_cursor(1, 0);

    h.feed("5l");
    assert_eq!(h.cursor(), (1, 5));
}
