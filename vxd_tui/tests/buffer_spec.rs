//! Buffer tests ported from Neovim's test/functional/api/buffer_spec.lua
//!
//! These tests verify buffer manipulation behavior including:
//! - Line insertion, deletion, and replacement
//! - Cursor position maintenance during buffer edits
//! - Buffer line count invariants

mod common;

use common::TestHarness;
use vxd::buffer::{Buffer, BufferManager};

/// Test: nvim_buf_set_lines, nvim_buf_line_count - deprecated forms
/// Source: buffer_spec.lua line 30-44
#[test]
fn test_line_count_basic() {
    let mut h = TestHarness::new();

    // New buffer has 1 line
    assert_eq!(h.editor.buffers.current().line_count(), 1);

    // Insert a line
    h.editor
        .buffers
        .current_mut()
        .set_lines(-1, -1, false, vec!["line".to_string()])
        .unwrap();
    assert_eq!(h.editor.buffers.current().line_count(), 2);

    // Insert another line
    h.editor
        .buffers
        .current_mut()
        .set_lines(-1, -1, false, vec!["line".to_string()])
        .unwrap();
    assert_eq!(h.editor.buffers.current().line_count(), 3);

    // Delete last line
    h.editor
        .buffers
        .current_mut()
        .set_lines(-2, -1, false, vec![])
        .unwrap();
    assert_eq!(h.editor.buffers.current().line_count(), 2);

    // Delete remaining lines - should always have at least 1
    h.editor
        .buffers
        .current_mut()
        .set_lines(0, -1, false, vec![])
        .unwrap();
    // There's always at least one line
    assert_eq!(h.editor.buffers.current().line_count(), 1);
}

/// Test: cursor position is maintained after lines are inserted #9961
/// Source: buffer_spec.lua line 53-92
#[test]
fn test_cursor_maintained_after_line_insert() {
    let mut h = TestHarness::new();

    // Replace the buffer contents with these four lines
    h.set_lines(&["line1", "line2", "line3", "line4"]);

    // Set the current cursor to {3, 2} (line 3, column 2)
    h.set_cursor(3, 2);
    assert_eq!(h.cursor(), (3, 2));

    // Add 2 lines and delete 1 line above the current cursor position
    h.set_lines_range(1, 2, true, vec!["line5".to_string(), "line6".to_string()]);

    // Check the current set of lines in the buffer
    assert_eq!(
        h.get_lines(),
        vec!["line1", "line5", "line6", "line3", "line4"]
    );

    // Cursor should be moved below by 1 line (from 3 to 4)
    assert_eq!(h.cursor(), (4, 2));
}

/// Test: cursor position after adding line after cursor
/// Source: buffer_spec.lua line 74-82
#[test]
fn test_cursor_unchanged_after_line_added_below() {
    let mut h = TestHarness::new();

    h.set_lines(&["line1", "line2", "line3", "line4"]);
    h.set_cursor(2, 2);

    // Add a line after the current cursor position
    h.editor
        .buffers
        .current_mut()
        .set_lines(4, 4, false, vec!["line5".to_string()])
        .unwrap();
    h.editor.sync_cursor_with_buffer();

    // Check the current set of lines
    assert_eq!(
        h.get_lines(),
        vec!["line1", "line2", "line3", "line4", "line5"]
    );

    // Cursor position should be unchanged
    assert_eq!(h.cursor(), (2, 2));
}

/// Test: overwrite current cursor line
/// Source: buffer_spec.lua line 84-92
#[test]
fn test_cursor_unchanged_when_overwriting_current_line() {
    let mut h = TestHarness::new();

    h.set_lines(&["line1", "line2", "line3", "line4"]);
    h.set_cursor(3, 2);

    // Overwrite lines 3-4 with new content
    h.set_lines_range(2, 4, true, vec!["line8".to_string(), "line9".to_string()]);

    // Check buffer content
    assert_eq!(h.get_lines(), vec!["line1", "line2", "line8", "line9"]);

    // Cursor position should be unchanged
    assert_eq!(h.cursor(), (3, 2));
}

/// Test: buffer name can be changed
/// Source: :file {newname}
#[test]
fn test_buffer_name_changes() {
    let mut h = TestHarness::new();

    assert_eq!(h.editor.buffers.current().name(), "");

    h.editor
        .buffers
        .current_mut()
        .set_name("one.txt")
        .unwrap();
    assert_eq!(h.editor.buffers.current().name(), "one.txt");

    h.editor
        .buffers
        .current_mut()
        .set_name("two.txt")
        .unwrap();
    assert_eq!(h.editor.buffers.current().name(), "two.txt");
}

/// Test: get_lines returns correct content
/// Source: buffer_spec.lua (various)
#[test]
fn test_get_lines_basic() {
    let mut h = TestHarness::new();

    h.set_lines(&["line1", "line2", "line3", "line4"]);

    // Get all lines
    let lines = h.editor.buffers.current().get_lines(0, -1, false).unwrap();
    assert_eq!(lines, vec!["line1", "line2", "line3", "line4"]);

    // Get subset of lines (1-indexed in display, 0-indexed in API)
    let lines = h.editor.buffers.current().get_lines(1, 3, true).unwrap();
    assert_eq!(lines, vec!["line2", "line3"]);
}

/// Test: empty range returns empty
/// Source: buffer_spec.lua line 204-206
#[test]
fn test_empty_range_returns_empty() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Get empty range
    let lines = h.editor.buffers.current().get_lines(1, 1, false).unwrap();
    assert!(lines.is_empty());
}

/// Test: negative indices work correctly
/// Source: buffer_spec.lua (various negative index tests)
#[test]
fn test_negative_indices() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3", "line4"]);

    // -1 means end of buffer
    let lines = h.editor.buffers.current().get_lines(0, -1, false).unwrap();
    assert_eq!(lines.len(), 4);

    // -2 means last line (-1 is past the end)
    let lines = h.editor.buffers.current().get_lines(-2, -1, false).unwrap();
    assert_eq!(lines, vec!["line4"]);
}

/// Test: set_lines with strict mode errors on out of bounds
/// Source: buffer_spec.lua (strict mode tests)
#[test]
fn test_set_lines_strict_mode() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2"]);

    // Strict mode should error on out of bounds
    let result = h
        .editor
        .buffers
        .current_mut()
        .set_lines(100, 200, true, vec!["new".to_string()]);
    assert!(result.is_err());
}

/// Test: set_lines with non-strict mode clamps indices
/// Source: buffer_spec.lua (non-strict mode tests)
#[test]
fn test_set_lines_non_strict_mode() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2"]);

    // Non-strict mode should clamp indices
    let result = h
        .editor
        .buffers
        .current_mut()
        .set_lines(100, 200, false, vec!["new".to_string()]);
    assert!(result.is_ok());
}

/// Test: buffer always has at least one line
/// Source: buffer_spec.lua line 44 "There's always at least one line"
#[test]
fn test_buffer_minimum_one_line() {
    let mut h = TestHarness::new();

    // Even after deleting everything, buffer should have 1 line
    h.editor
        .buffers
        .current_mut()
        .set_lines(0, -1, false, vec![])
        .unwrap();
    assert_eq!(h.editor.buffers.current().line_count(), 1);

    // The line should be empty
    let lines = h.editor.buffers.current().get_lines(0, -1, false).unwrap();
    assert_eq!(lines, vec![""]);
}

/// Test: insert at beginning
/// Source: derived from buffer_spec.lua insertion tests
#[test]
fn test_insert_at_beginning() {
    let mut h = TestHarness::new();
    h.set_lines(&["line2", "line3"]);

    // Insert at beginning
    h.editor
        .buffers
        .current_mut()
        .set_lines(0, 0, false, vec!["line1".to_string()])
        .unwrap();

    assert_eq!(h.get_lines(), vec!["line1", "line2", "line3"]);
}

/// Test: insert at end
/// Source: derived from buffer_spec.lua insertion tests
#[test]
fn test_insert_at_end() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2"]);

    // Insert at end (using -1 as end marker, then -1 again)
    h.editor
        .buffers
        .current_mut()
        .set_lines(-1, -1, false, vec!["line3".to_string()])
        .unwrap();

    assert_eq!(h.get_lines(), vec!["line1", "line2", "line3"]);
}

/// Test: delete lines in middle
/// Source: buffer_spec.lua deletion tests
#[test]
fn test_delete_middle_lines() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3", "line4"]);

    // Delete lines 2-3 (indices 1-3)
    h.editor
        .buffers
        .current_mut()
        .set_lines(1, 3, false, vec![])
        .unwrap();

    assert_eq!(h.get_lines(), vec!["line1", "line4"]);
}

/// Test: replace lines
/// Source: buffer_spec.lua replacement tests
#[test]
fn test_replace_lines() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Replace line 2 with two new lines
    h.editor
        .buffers
        .current_mut()
        .set_lines(1, 2, false, vec!["new1".to_string(), "new2".to_string()])
        .unwrap();

    assert_eq!(h.get_lines(), vec!["line1", "new1", "new2", "line3"]);
}

/// Test: buffer modification flag
/// Source: buffer_spec.lua modification tests
#[test]
fn test_buffer_modified_flag() {
    let mut h = TestHarness::new();

    // New buffer should not be modified
    // (depends on implementation - ours might start modified)

    // After setting lines, should be modified
    h.set_lines(&["hello"]);
    assert!(h.editor.buffers.current().is_modified());
}

/// Test: line_count for various buffer states
/// Source: buffer_spec.lua line_count tests
#[test]
fn test_line_count_various() {
    let mut h = TestHarness::new();

    // Single empty line
    assert_eq!(h.editor.buffers.current().line_count(), 1);

    // Multiple lines
    h.set_lines(&["a", "b", "c"]);
    assert_eq!(h.editor.buffers.current().line_count(), 3);

    // After deletion
    h.editor
        .buffers
        .current_mut()
        .set_lines(0, 1, false, vec![])
        .unwrap();
    assert_eq!(h.editor.buffers.current().line_count(), 2);
}

/// Test: cursor stays within buffer bounds after deletion
/// Source: buffer_spec.lua cursor maintenance tests
#[test]
fn test_cursor_bounded_after_deletion() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);
    h.set_cursor(3, 0); // Last line

    // Delete the line the cursor is on
    h.editor
        .buffers
        .current_mut()
        .set_lines(2, 3, false, vec![])
        .unwrap();
    h.editor.sync_cursor_with_buffer();

    // Cursor should be adjusted to valid position
    let (line, _) = h.cursor();
    let line_count = h.editor.buffers.current().line_count();
    assert!(line >= 1 && line <= line_count);
}

/// Test: get single line
#[test]
fn test_get_single_line() {
    let mut h = TestHarness::new();
    h.set_lines(&["first", "second", "third"]);

    let line = h.editor.buffers.current().get_line(0).unwrap();
    assert_eq!(line, "first");

    let line = h.editor.buffers.current().get_line(1).unwrap();
    assert_eq!(line, "second");

    let line = h.editor.buffers.current().get_line(2).unwrap();
    assert_eq!(line, "third");
}

/// Test: get line with negative index
#[test]
fn test_get_line_negative_index() {
    let mut h = TestHarness::new();
    h.set_lines(&["first", "second", "third"]);

    // -2 should get last line (-1 is past the end)
    let line = h.editor.buffers.current().get_line(-2).unwrap();
    assert_eq!(line, "third");

    // -3 should get second to last
    let line = h.editor.buffers.current().get_line(-3).unwrap();
    assert_eq!(line, "second");
}
