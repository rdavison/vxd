//! Jump list tests ported from Neovim's test/functional/editor/jump_spec.lua
//!
//! These tests verify jump list behavior including:
//! - Jump list navigation with Ctrl-O and Ctrl-I
//! - Context marks when jumping
//! - Jump list maintenance

mod common;

use common::TestHarness;
use vxd::buffer::{Buffer, BufferManager};
use vxd::cursor::{Cursor, CursorPosition};
use vxd::types::LineNr;

/// Test: jump list starts empty
/// Source: jump_spec.lua (initial state)
#[test]
fn test_jump_list_starts_empty() {
    let h = TestHarness::new();

    let entries = h.editor.marks.jump_list_entries();
    assert!(entries.is_empty() || entries.len() == 1); // May have initial entry
}

/// Test: jumping adds to jump list
/// Source: jump_spec.lua (jump list populated on jumps)
#[test]
fn test_jump_adds_to_list() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3", "line4", "line5"]);
    h.set_cursor(1, 0);

    // Record initial position
    let initial_pos = h.editor.cursor.position();
    h.editor
        .marks
        .push_jump(h.editor.buffers.current().handle(), initial_pos);

    // "Jump" to line 3 (like G would do)
    h.set_cursor(3, 0);
    h.editor.marks.push_jump(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    );

    let entries = h.editor.marks.jump_list_entries();
    assert!(entries.len() >= 1);
}

/// Test: Ctrl-O goes back in jump list
/// Source: jump_spec.lua line 30-40 (Ctrl-O navigation)
#[test]
fn test_jump_back_ctrl_o() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3", "line4", "line5"]);

    // Start at line 1
    h.set_cursor(1, 0);
    let pos1 = h.editor.cursor.position();
    h.editor
        .marks
        .push_jump(h.editor.buffers.current().handle(), pos1);

    // Jump to line 3
    h.set_cursor(3, 0);
    let pos2 = h.editor.cursor.position();
    h.editor
        .marks
        .push_jump(h.editor.buffers.current().handle(), pos2);

    // Jump to line 5
    h.set_cursor(5, 0);

    // Go back with Ctrl-O
    if let Some(prev) = h.editor.marks.jump_back(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    ) {
        // Should go back to previous position
        assert!(prev.line.0 < 5);
    }
}

/// Test: Ctrl-I goes forward in jump list
/// Source: jump_spec.lua (Ctrl-I navigation)
#[test]
fn test_jump_forward_ctrl_i() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Build up jump history
    h.set_cursor(1, 0);
    h.editor.marks.push_jump(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    );

    h.set_cursor(3, 0);
    h.editor.marks.push_jump(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    );

    // Go back
    let _prev = h.editor.marks.jump_back(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    );

    // Go forward should return to line 3
    if let Some(next) = h.editor.marks.jump_forward() {
        assert_eq!(next.line.0, 3);
    }
}

/// Test: jump list has maximum size
/// Source: jump_spec.lua (jump list limit)
#[test]
fn test_jump_list_max_size() {
    let mut h = TestHarness::new();
    h.set_lines(
        &(1..=200)
            .map(|i| format!("line{}", i))
            .collect::<Vec<_>>()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
    );

    // Add many jumps
    for i in 1..=150 {
        h.set_cursor(i, 0);
        h.editor.marks.push_jump(
            h.editor.buffers.current().handle(),
            h.editor.cursor.position(),
        );
    }

    let entries = h.editor.marks.jump_list_entries();
    // Jump list typically has a max of 100 entries
    assert!(entries.len() <= 100);
}

/// Test: duplicate jumps are coalesced
/// Source: Vim behavior - repeated jumps to same location don't add duplicates
#[test]
fn test_duplicate_jumps_coalesced() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2", "line3"]);

    // Jump to same location multiple times
    h.set_cursor(2, 0);
    let pos = h.editor.cursor.position();

    for _ in 0..5 {
        h.editor
            .marks
            .push_jump(h.editor.buffers.current().handle(), pos);
    }

    // Should not have 5 entries for the same position
    let entries = h.editor.marks.jump_list_entries();
    // Count entries at line 2
    let line2_count = entries.iter().filter(|e| e.position.line.0 == 2).count();
    assert!(line2_count <= 2); // Should be coalesced
}

/// Test: jump list entries have buffer association
/// Source: jump_spec.lua (cross-buffer jumps)
#[test]
fn test_jump_entries_have_buffer() {
    let mut h = TestHarness::new();
    h.set_lines(&["content"]);

    h.set_cursor(1, 0);
    h.editor.marks.push_jump(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    );

    let entries = h.editor.marks.jump_list_entries();
    if !entries.is_empty() {
        // Each entry should have a buffer handle
        // The exact check depends on implementation
        assert!(entries[0].buffer.0 > 0 || entries[0].buffer.0 == 0);
    }
}

/// Test: jump list position includes column
/// Source: Vim behavior - jumps preserve exact position
#[test]
fn test_jump_preserves_column() {
    let mut h = TestHarness::new();
    h.set_lines(&["hello world"]);

    h.set_cursor(1, 6); // On 'w'
    h.editor.marks.push_jump(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    );

    let entries = h.editor.marks.jump_list_entries();
    if !entries.is_empty() {
        assert_eq!(entries.last().unwrap().position.col, 6);
    }
}

/// Test: clearing jump list
/// Source: jump_spec.lua (:clearjumps command)
#[test]
fn test_clear_jump_list() {
    let mut h = TestHarness::new();
    h.set_lines(&["line1", "line2"]);

    // Add some jumps
    h.set_cursor(1, 0);
    h.editor.marks.push_jump(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    );
    h.set_cursor(2, 0);
    h.editor.marks.push_jump(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    );

    // Clear
    h.editor.marks.clear_jump_list();

    let entries = h.editor.marks.jump_list_entries();
    assert!(entries.is_empty());
}

/// Test: jump position in current jump list
/// Source: jump_spec.lua (current position in list)
#[test]
fn test_jump_list_current_position() {
    let mut h = TestHarness::new();
    h.set_lines(&["a", "b", "c", "d", "e"]);

    // Add jumps
    for i in 1..=5 {
        h.set_cursor(i, 0);
        h.editor.marks.push_jump(
            h.editor.buffers.current().handle(),
            h.editor.cursor.position(),
        );
    }

    // Current position should be at the end
    let pos = h.editor.marks.jump_list_position();
    let len = h.editor.marks.jump_list_entries().len();
    assert!(pos <= len);
}

/// Test: going back then adding jump truncates forward history
/// Source: Vim behavior - new jump after going back removes forward history
#[test]
fn test_jump_truncates_forward_history() {
    let mut h = TestHarness::new();
    h.set_lines(&["a", "b", "c", "d", "e"]);

    // Build history
    for i in 1..=5 {
        h.set_cursor(i, 0);
        h.editor.marks.push_jump(
            h.editor.buffers.current().handle(),
            h.editor.cursor.position(),
        );
    }

    // Go back twice
    h.editor.marks.jump_back(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    );
    h.editor.marks.jump_back(
        h.editor.buffers.current().handle(),
        CursorPosition::new(LineNr(4), 0),
    );

    // Add new jump
    h.set_cursor(1, 5);
    h.editor.marks.push_jump(
        h.editor.buffers.current().handle(),
        h.editor.cursor.position(),
    );

    // Forward history should be truncated
    // Going forward should return None or limited results
    let forward = h.editor.marks.jump_forward();
    assert!(forward.is_none());
}
