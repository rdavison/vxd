//! Normal mode tests ported from Neovim's test/functional/editor/mode_normal_spec.lua
//!
//! These tests verify normal mode behavior including:
//! - Mode transitions
//! - Basic commands (x, delete, etc.)
//! - Movement commands

#![allow(non_snake_case)]

mod common;

use common::TestHarness;
use vxd::buffer::{Buffer, BufferManager};
use vxd::modes::Mode;

/// Test: starts in normal mode
/// Source: basic Vim behavior
#[test]
fn test_starts_in_normal_mode() {
    let h = TestHarness::new();
    assert!(matches!(h.mode(), Mode::Normal));
}

/// Test: 'x' deletes character under cursor
/// Source: basic Vim behavior
#[test]
fn test_x_deletes_char() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 0);

    h.feed("x");
    assert_eq!(h.get_lines(), vec!["ello"]);
}

/// Test: 'x' at end of line
/// Source: Vim behavior
#[test]
fn test_x_at_end_of_line() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 4); // On 'o'

    h.feed("x");
    assert_eq!(h.get_lines(), vec!["hell"]);
}

/// Test: 'x' on empty line does nothing
/// Source: Vim behavior
#[test]
fn test_x_on_empty_line() {
    let mut h = TestHarness::new();
    h.set_lines(&[""]);
    h.set_cursor(1, 0);

    h.feed("x");
    assert_eq!(h.get_lines(), vec![""]);
}

/// Test: multiple 'x' commands
/// Source: basic Vim behavior
#[test]
fn test_multiple_x() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 0);

    h.feed("xxx");
    assert_eq!(h.get_lines(), vec!["lo"]);
}

/// Test: 'i' enters insert mode
/// Source: mode_normal_spec.lua
#[test]
fn test_i_enters_insert() {
    let mut h = TestHarness::new();

    assert!(matches!(h.mode(), Mode::Normal));
    h.feed("i");
    assert!(matches!(h.mode(), Mode::Insert));
}

/// Test: transition from normal to insert and back
/// Source: basic Vim behavior
#[test]
fn test_mode_transition_cycle() {
    let mut h = TestHarness::new();

    assert!(matches!(h.mode(), Mode::Normal));

    h.feed("i");
    assert!(matches!(h.mode(), Mode::Insert));

    h.feed("<Esc>");
    assert!(matches!(h.mode(), Mode::Normal));

    h.feed("a");
    assert!(matches!(h.mode(), Mode::Insert));

    h.feed("<Esc>");
    assert!(matches!(h.mode(), Mode::Normal));
}

/// Test: 'j' moves down
/// Source: basic Vim behavior
#[test]
fn test_j_moves_down() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);
    h.set_cursor(1, 0);

    h.feed("j");
    let (line, _) = h.cursor();
    assert_eq!(line, 2);
}

/// Test: 'k' moves up
/// Source: basic Vim behavior
#[test]
fn test_k_moves_up() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);
    h.set_cursor(2, 0);

    h.feed("k");
    let (line, _) = h.cursor();
    assert_eq!(line, 1);
}

/// Test: 'h' moves left
/// Source: basic Vim behavior
#[test]
fn test_h_moves_left() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 3);

    h.feed("h");
    let (_, col) = h.cursor();
    assert_eq!(col, 2);
}

/// Test: 'l' moves right
/// Source: basic Vim behavior
#[test]
fn test_l_moves_right() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 1);

    h.feed("l");
    let (_, col) = h.cursor();
    assert_eq!(col, 2);
}

/// Test: '0' moves to line start
/// Source: basic Vim behavior
#[test]
fn test_zero_moves_to_start() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello world"]);
    h.set_cursor(1, 5);

    h.feed("0");
    let (_, col) = h.cursor();
    assert_eq!(col, 0);
}

/// Test: '$' moves to line end
/// Source: basic Vim behavior
#[test]
fn test_dollar_moves_to_end() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 0);

    h.feed("$");
    let (_, col) = h.cursor();
    assert_eq!(col, 4); // Last char
}

/// Test: 'G' moves to last line
/// Source: basic Vim behavior
#[test]
fn test_G_moves_to_last_line() {
    let mut h = TestHarness::new();
    h.set_lines(&["a", "b", "c", "d"]);
    h.set_cursor(1, 0);

    h.feed("G");
    let (line, _) = h.cursor();
    assert_eq!(line, 4);
}

/// Test: escape does nothing in normal mode
/// Source: Vim behavior
#[test]
fn test_escape_in_normal_mode() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 2);

    let before = h.cursor();
    h.feed("<Esc>");
    let after = h.cursor();

    // Escape in normal mode shouldn't change anything
    assert_eq!(before, after);
    assert!(matches!(h.mode(), Mode::Normal));
}

/// Test: editing leaves buffer modified
/// Source: basic Vim behavior
#[test]
fn test_editing_modifies_buffer() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);

    h.feed("x");
    assert!(h.editor.buffers.current().is_modified());
}

/// Test: insert then normal mode edit
/// Source: combined behavior
#[test]
fn test_insert_then_normal_edit() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);

    h.feed("Aworld<Esc>");
    assert_eq!(h.get_lines(), vec!["helloworld"]);

    h.feed("0x");
    assert_eq!(h.get_lines(), vec!["elloworld"]);
}

/// Test: 'o' opens line below and enters insert
/// Source: basic Vim behavior
#[test]
fn test_o_opens_line_below() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2"]);
    h.set_cursor(1, 0);

    h.feed("o");
    assert!(matches!(h.mode(), Mode::Insert));

    h.feed("new<Esc>");
    assert_eq!(h.get_lines(), vec!["line1", "new", "line2"]);
}

/// Test: 'O' opens line above and enters insert
/// Source: basic Vim behavior
#[test]
fn test_O_opens_line_above() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2"]);
    h.set_cursor(2, 0);

    h.feed("O");
    assert!(matches!(h.mode(), Mode::Insert));

    h.feed("new<Esc>");
    assert_eq!(h.get_lines(), vec!["line1", "new", "line2"]);
}

/// Test: 'A' appends at end of line
/// Source: basic Vim behavior
#[test]
fn test_A_appends_at_end() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello"]);
    h.set_cursor(1, 0);

    h.feed("A world<Esc>");
    assert_eq!(h.get_lines(), vec!["hello world"]);
}

/// Test: 'I' inserts at beginning of line
/// Source: basic Vim behavior
#[test]
fn test_I_inserts_at_beginning() {
    let mut h = TestHarness::new();
    h.set_lines(&["world"]);
    h.set_cursor(1, 3);

    h.feed("Ihello <Esc>");
    assert_eq!(h.get_lines(), vec!["hello world"]);
}

/// Test: cursor stays in bounds after delete
/// Source: Vim behavior
#[test]
fn test_cursor_bounds_after_delete() {
    let mut h = TestHarness::new();
    h.set_lines(&["ab"]);
    h.set_cursor(1, 1); // On 'b'

    h.feed("x"); // Delete 'b'
                 // Now only 'a' remains, cursor should be on 'a'
    let (_, col) = h.cursor();
    assert_eq!(col, 0);
}
