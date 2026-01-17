//! Mark tests ported from Neovim's test/functional/editor/mark_spec.lua
//!
//! These tests verify mark behavior including:
//! - Setting marks with m<letter>
//! - Moving to marks with ' and `
//! - Local marks (a-z) vs global marks (A-Z)
//! - Mark navigation and context marks

mod common;

use common::TestHarness;
use vxd::buffer::{Buffer, BufferManager};
use vxd::cursor::{Cursor, CursorPosition};
use vxd::marks::{Mark, MarkManager, MarkValue};
use vxd::types::LineNr;

/// Test: marks can be set
/// Source: mark_spec.lua line 32-42
#[test]
fn test_marks_can_be_set() {
    let mut h = TestHarness::new();
    h.set_lines(&["1test1", "1test2", "1test3", "1test4"]);

    // Set mark 'a' on line 1
    h.set_cursor(1, 0);
    let pos = h.editor.cursor.position();
    h.editor
        .marks
        .set(Mark::Local('a'), MarkValue::new(pos))
        .unwrap();

    let mark_val = h.editor.marks.get(Mark::Local('a'));
    assert!(mark_val.is_some());
    let val = mark_val.unwrap();
    assert_eq!(val.position.line, LineNr(1));
    assert_eq!(val.position.col, 0);

    // Set mark 'b' on line 2
    h.set_cursor(2, 0);
    let pos = h.editor.cursor.position();
    h.editor
        .marks
        .set(Mark::Local('b'), MarkValue::new(pos))
        .unwrap();

    let mark_val = h.editor.marks.get(Mark::Local('b'));
    assert!(mark_val.is_some());
    let val = mark_val.unwrap();
    assert_eq!(val.position.line, LineNr(2));
}

/// Test: global marks can be set (A-Z)
/// Source: mark_spec.lua line 38
#[test]
fn test_global_marks_can_be_set() {
    let mut h = TestHarness::new();
    h.set_lines(&["1test1", "1test2", "1test3", "1test4"]);

    // Set global mark 'B' on line 3
    h.set_cursor(3, 0);
    let pos = h.editor.cursor.position();
    h.editor
        .marks
        .set(Mark::Global('B'), MarkValue::new(pos))
        .unwrap();

    let mark_val = h.editor.marks.get(Mark::Global('B'));
    assert!(mark_val.is_some());
    let val = mark_val.unwrap();
    assert_eq!(val.position.line, LineNr(3));
}

/// Test: mark position includes column
/// Source: mark_spec.lua line 103-108
#[test]
fn test_mark_preserves_column() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello world", "test line"]);

    // Set cursor at column 6 (the 'w' in 'world')
    h.set_cursor(1, 6);
    let pos = h.editor.cursor.position();
    h.editor
        .marks
        .set(Mark::Local('a'), MarkValue::new(pos))
        .unwrap();

    let mark_val = h.editor.marks.get(Mark::Local('a'));
    assert!(mark_val.is_some());
    let val = mark_val.unwrap();
    assert_eq!(val.position.col, 6);
}

/// Test: getting unset mark returns None
/// Source: mark_spec.lua line 70-74 (errors when moving to unset mark)
#[test]
fn test_unset_mark_returns_none() {
    let h = TestHarness::new();

    let mark_val = h.editor.marks.get(Mark::Local('z'));
    assert!(mark_val.is_none());
}

/// Test: visual marks < and >
/// Source: mark_spec.lua line 78 (errors for unset '>' mark)
#[test]
fn test_visual_marks() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Initially visual marks should be unset
    let mark_val = h.editor.marks.get(Mark::VisualStart);
    assert!(mark_val.is_none());

    let mark_val = h.editor.marks.get(Mark::VisualEnd);
    assert!(mark_val.is_none());

    // Set visual selection marks
    let start = CursorPosition::new(LineNr(1), 2);
    let end = CursorPosition::new(LineNr(2), 4);
    h.editor.marks.set_visual_marks(start, end);

    // Now they should be set
    let start_val = h.editor.marks.get(Mark::VisualStart).unwrap();
    assert_eq!(start_val.position.line, LineNr(1));
    assert_eq!(start_val.position.col, 2);

    let end_val = h.editor.marks.get(Mark::VisualEnd).unwrap();
    assert_eq!(end_val.position.line, LineNr(2));
    assert_eq!(end_val.position.col, 4);
}

/// Test: marks are adjusted when lines are inserted above
/// Source: Vim behavior - marks adjust when text is inserted before them
#[test]
fn test_marks_adjust_on_insert_above() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Set mark on line 2
    h.set_cursor(2, 0);
    let pos = h.editor.cursor.position();
    h.editor
        .marks
        .set(Mark::Local('a'), MarkValue::new(pos))
        .unwrap();

    // Verify mark is on line 2
    let val = h.editor.marks.get(Mark::Local('a')).unwrap();
    assert_eq!(val.position.line, LineNr(2));

    // Insert a line before the mark
    h.editor
        .buffers
        .current_mut()
        .set_lines(0, 0, false, vec!["new line".to_string()])
        .unwrap();

    // Notify marks of the insertion (line 1, added 1 line)
    h.editor.marks.adjust(LineNr(1), 0, 1, 0);

    // Mark should now be on line 3
    let val = h.editor.marks.get(Mark::Local('a')).unwrap();
    assert_eq!(val.position.line, LineNr(3));
}

/// Test: marks are adjusted when lines are deleted
/// Source: Vim behavior - marks adjust when text before them is deleted
#[test]
fn test_marks_adjust_on_delete() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3", "line4"]);

    // Set mark on line 4
    h.set_cursor(4, 0);
    let pos = h.editor.cursor.position();
    h.editor
        .marks
        .set(Mark::Local('a'), MarkValue::new(pos))
        .unwrap();

    // Delete line 2
    h.editor
        .buffers
        .current_mut()
        .set_lines(1, 2, false, vec![])
        .unwrap();

    // Notify marks of the deletion (line 2, removed 1 line = -1)
    h.editor.marks.adjust(LineNr(2), 0, -1, 0);

    // Mark should now be on line 3
    let val = h.editor.marks.get(Mark::Local('a')).unwrap();
    assert_eq!(val.position.line, LineNr(3));
}

/// Test: all lowercase marks a-z
/// Source: mark_spec.lua line 32-42 (tests various letters)
#[test]
fn test_all_lowercase_marks() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Test a few representative lowercase marks
    for (i, c) in ['a', 'm', 'z'].iter().enumerate() {
        h.set_cursor((i % 3) + 1, i);
        let pos = h.editor.cursor.position();
        h.editor
            .marks
            .set(Mark::Local(*c), MarkValue::new(pos))
            .unwrap();

        let val = h.editor.marks.get(Mark::Local(*c)).unwrap();
        assert_eq!(val.position.line, LineNr((i % 3) + 1));
    }
}

/// Test: all uppercase marks A-Z
/// Source: mark_spec.lua (global mark tests)
#[test]
fn test_all_uppercase_marks() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Test a few representative uppercase marks
    for (i, c) in ['A', 'M', 'Z'].iter().enumerate() {
        h.set_cursor((i % 3) + 1, i);
        let pos = h.editor.cursor.position();
        h.editor
            .marks
            .set(Mark::Global(*c), MarkValue::new(pos))
            .unwrap();

        let val = h.editor.marks.get(Mark::Global(*c)).unwrap();
        assert_eq!(val.position.line, LineNr((i % 3) + 1));
    }
}

/// Test: special marks [ and ] (last change boundaries)
/// Source: mark_spec.lua (various special mark tests)
#[test]
fn test_change_marks() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Set change boundaries
    let start = CursorPosition::new(LineNr(1), 0);
    let end = CursorPosition::new(LineNr(2), 5);
    h.editor.marks.set_change_marks(start, end);

    let start_val = h.editor.marks.get(Mark::ChangeStart).unwrap();
    assert_eq!(start_val.position.line, LineNr(1));

    let end_val = h.editor.marks.get(Mark::ChangeEnd).unwrap();
    assert_eq!(end_val.position.line, LineNr(2));
    assert_eq!(end_val.position.col, 5);
}

/// Test: last insert position mark ^
/// Source: mark_spec.lua (last insert mark)
#[test]
fn test_last_insert_mark() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello world"]);

    // Simulate setting the last insert position
    h.set_cursor(1, 5);
    let pos = h.editor.cursor.position();
    h.editor
        .marks
        .set(Mark::LastInsert, MarkValue::new(pos))
        .unwrap();

    let val = h.editor.marks.get(Mark::LastInsert).unwrap();
    assert_eq!(val.position.line, LineNr(1));
    assert_eq!(val.position.col, 5);
}

/// Test: last cursor position mark "
/// Source: mark_spec.lua (last cursor position tests)
#[test]
fn test_last_cursor_position_mark() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Set last cursor position
    h.set_cursor(2, 3);
    let pos = h.editor.cursor.position();
    let result = h.editor.marks.set(Mark::LastExit, MarkValue::new(pos));
    assert!(result.is_err());
}

/// Test: Mark::from_char parsing
/// Source: API test
#[test]
fn test_mark_from_char() {
    assert!(matches!(Mark::from_char('a'), Ok(Mark::Local('a'))));
    assert!(matches!(Mark::from_char('z'), Ok(Mark::Local('z'))));
    assert!(matches!(Mark::from_char('A'), Ok(Mark::Global('A'))));
    assert!(matches!(Mark::from_char('Z'), Ok(Mark::Global('Z'))));
    assert!(matches!(Mark::from_char('0'), Ok(Mark::Numbered(0))));
    assert!(matches!(Mark::from_char('9'), Ok(Mark::Numbered(9))));
    assert!(matches!(Mark::from_char('<'), Ok(Mark::VisualStart)));
    assert!(matches!(Mark::from_char('>'), Ok(Mark::VisualEnd)));
    assert!(matches!(Mark::from_char('['), Ok(Mark::ChangeStart)));
    assert!(matches!(Mark::from_char(']'), Ok(Mark::ChangeEnd)));
}

/// Test: Mark::to_char
/// Source: API test
#[test]
fn test_mark_to_char() {
    assert_eq!(Mark::Local('a').to_char(), 'a');
    assert_eq!(Mark::Global('A').to_char(), 'A');
    assert_eq!(Mark::Numbered(5).to_char(), '5');
    assert_eq!(Mark::VisualStart.to_char(), '<');
    assert_eq!(Mark::VisualEnd.to_char(), '>');
}

/// Test: Mark::is_local and is_global
/// Source: API test
#[test]
fn test_mark_locality() {
    assert!(Mark::Local('a').is_local());
    assert!(!Mark::Local('a').is_global());

    assert!(Mark::Global('A').is_global());
    assert!(!Mark::Global('A').is_local());

    assert!(Mark::Numbered(0).is_global());
}

/// Test: Jump list basics
/// Source: jump_spec.lua
#[test]
fn test_jump_list_basic() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Record some jumps
    h.editor
        .marks
        .record_jump(CursorPosition::new(LineNr(1), 0));
    h.editor
        .marks
        .record_jump(CursorPosition::new(LineNr(2), 0));
    h.editor
        .marks
        .record_jump(CursorPosition::new(LineNr(3), 0));

    assert_eq!(h.editor.marks.jump_list().len(), 3);
}

/// Test: Change list basics
/// Source: undo_spec.lua (change tracking)
#[test]
fn test_change_list_basic() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2"]);

    // Record some changes
    h.editor
        .marks
        .record_change(CursorPosition::new(LineNr(1), 5));
    h.editor
        .marks
        .record_change(CursorPosition::new(LineNr(2), 3));

    assert_eq!(h.editor.marks.change_list().len(), 2);
}
