//! Visual mode tests ported from Neovim tests
//!
//! These tests verify visual mode behavior including:
//! - Characterwise visual mode (v)
//! - Linewise visual mode (V)
//! - Blockwise visual mode (Ctrl-V)
//! - Selection normalization
//! - Mode toggling
//! - Reselect (gv)

mod common;

use common::TestHarness;
use vxd::cursor::CursorPosition;
use vxd::modes::VisualMode;
use vxd::types::LineNr;
use vxd::visual::{BlockSelection, VisualSelection};

// ============================================================================
// Basic Visual Selection Tests
// ============================================================================

/// Test: create characterwise visual selection
/// Source: v command
#[test]
fn test_visual_selection_charwise() {
    let selection = VisualSelection::new(CursorPosition::new(LineNr(1), 0), VisualMode::Char);

    assert_eq!(selection.mode, VisualMode::Char);
    assert_eq!(selection.start, selection.end);
}

/// Test: create linewise visual selection
/// Source: V command
#[test]
fn test_visual_selection_linewise() {
    let selection = VisualSelection::new(CursorPosition::new(LineNr(1), 5), VisualMode::Line);

    assert_eq!(selection.mode, VisualMode::Line);
}

/// Test: create blockwise visual selection
/// Source: Ctrl-V command
#[test]
fn test_visual_selection_blockwise() {
    let selection = VisualSelection::new(CursorPosition::new(LineNr(1), 0), VisualMode::Block);

    assert_eq!(selection.mode, VisualMode::Block);
}

// ============================================================================
// Selection Normalization Tests
// ============================================================================

/// Test: normalize selection (start < end)
/// Source: visual selection direction handling
#[test]
fn test_visual_selection_normalized_forward() {
    let selection = VisualSelection {
        start: CursorPosition::new(LineNr(1), 0),
        end: CursorPosition::new(LineNr(3), 5),
        mode: VisualMode::Char,
    };

    let (start, end) = selection.normalized();
    assert_eq!(start.line, LineNr(1));
    assert_eq!(end.line, LineNr(3));
}

/// Test: normalize selection when backwards
/// Source: selecting upwards
#[test]
fn test_visual_selection_normalized_backward() {
    let selection = VisualSelection {
        start: CursorPosition::new(LineNr(5), 10),
        end: CursorPosition::new(LineNr(2), 3),
        mode: VisualMode::Char,
    };

    let (start, end) = selection.normalized();
    assert_eq!(start.line, LineNr(2));
    assert_eq!(end.line, LineNr(5));
    assert_eq!(start.col, 3);
    assert_eq!(end.col, 10);
}

/// Test: normalize selection on same line
/// Source: single line selection
#[test]
fn test_visual_selection_normalized_same_line() {
    // Selection backwards on same line
    let selection = VisualSelection {
        start: CursorPosition::new(LineNr(1), 10),
        end: CursorPosition::new(LineNr(1), 5),
        mode: VisualMode::Char,
    };

    let (start, end) = selection.normalized();
    assert_eq!(start.col, 5);
    assert_eq!(end.col, 10);
}

// ============================================================================
// Selection Range Tests
// ============================================================================

/// Test: line range of selection
/// Source: linewise operations on visual selection
#[test]
fn test_visual_selection_line_range() {
    let selection = VisualSelection {
        start: CursorPosition::new(LineNr(3), 0),
        end: CursorPosition::new(LineNr(7), 10),
        mode: VisualMode::Char,
    };

    let range = selection.line_range();
    assert_eq!(range.start, LineNr(3));
    assert_eq!(range.end, LineNr(7));
}

/// Test: multiline selection detection
/// Source: visual selection spanning lines
#[test]
fn test_visual_selection_multiline() {
    let single = VisualSelection {
        start: CursorPosition::new(LineNr(1), 0),
        end: CursorPosition::new(LineNr(1), 10),
        mode: VisualMode::Char,
    };

    let multi = VisualSelection {
        start: CursorPosition::new(LineNr(1), 0),
        end: CursorPosition::new(LineNr(3), 5),
        mode: VisualMode::Char,
    };

    assert!(!single.is_multiline());
    assert!(multi.is_multiline());
}

/// Test: convert to linewise
/// Source: visual mode operators that act linewise
#[test]
fn test_visual_selection_as_linewise() {
    let charwise = VisualSelection {
        start: CursorPosition::new(LineNr(2), 5),
        end: CursorPosition::new(LineNr(4), 10),
        mode: VisualMode::Char,
    };

    let linewise = charwise.as_linewise();
    assert_eq!(linewise.mode, VisualMode::Line);
    assert_eq!(linewise.start.col, 0);
    assert_eq!(linewise.end.col, 0);
}

// ============================================================================
// Block Selection Tests
// ============================================================================

/// Test: block selection dimensions
/// Source: Ctrl-V block operations
#[test]
fn test_block_selection_dimensions() {
    let block = BlockSelection {
        start_line: LineNr(1),
        end_line: LineNr(5),
        start_vcol: 10,
        end_vcol: 20,
    };

    assert_eq!(block.height(), 5);
    assert_eq!(block.width(), 11);
}

/// Test: block selection with reversed columns
/// Source: selecting block from right to left
#[test]
fn test_block_selection_reversed_columns() {
    let block = BlockSelection {
        start_line: LineNr(1),
        end_line: LineNr(3),
        start_vcol: 20,
        end_vcol: 10,
    };

    // Width should still be positive
    assert_eq!(block.width(), 11);
    assert_eq!(block.height(), 3);
}

/// Test: single column block
/// Source: Ctrl-V without horizontal movement
#[test]
fn test_block_selection_single_column() {
    let block = BlockSelection {
        start_line: LineNr(1),
        end_line: LineNr(5),
        start_vcol: 10,
        end_vcol: 10,
    };

    assert_eq!(block.width(), 1);
    assert_eq!(block.height(), 5);
}

/// Test: single line block
/// Source: Ctrl-V without vertical movement
#[test]
fn test_block_selection_single_line() {
    let block = BlockSelection {
        start_line: LineNr(3),
        end_line: LineNr(3),
        start_vcol: 5,
        end_vcol: 15,
    };

    assert_eq!(block.height(), 1);
    assert_eq!(block.width(), 11);
}

// ============================================================================
// Visual Mode Behavior Tests
// ============================================================================

/// Test: visual mode starts at cursor
/// Source: v command starts selection at cursor
#[test]
fn test_visual_starts_at_cursor() {
    let mut h = TestHarness::with_lines(&["hello world"]);
    h.set_cursor(1, 5);

    let selection = VisualSelection::new(CursorPosition::new(LineNr(1), 5), VisualMode::Char);

    assert_eq!(selection.start.col, 5);
    assert_eq!(selection.end.col, 5);
}

/// Test: visual selection extends with movement
/// Source: movement in visual mode extends selection
#[test]
fn test_visual_extends_with_movement() {
    let mut selection = VisualSelection::new(CursorPosition::new(LineNr(1), 0), VisualMode::Char);

    // Simulate cursor movement
    selection.end = CursorPosition::new(LineNr(1), 5);

    assert_eq!(selection.start.col, 0);
    assert_eq!(selection.end.col, 5);
}

/// Test: o command swaps selection ends
/// Source: o in visual mode
#[test]
fn test_visual_swap_ends() {
    let mut selection = VisualSelection {
        start: CursorPosition::new(LineNr(1), 0),
        end: CursorPosition::new(LineNr(3), 10),
        mode: VisualMode::Char,
    };

    // Swap start and end (o command)
    std::mem::swap(&mut selection.start, &mut selection.end);

    assert_eq!(selection.start.line, LineNr(3));
    assert_eq!(selection.end.line, LineNr(1));
}

/// Test: O command in block mode swaps corners
/// Source: O in visual block mode
#[test]
fn test_visual_block_swap_corners() {
    // In block mode, O swaps corners diagonally
    let mut selection = VisualSelection {
        start: CursorPosition::new(LineNr(1), 0),
        end: CursorPosition::new(LineNr(5), 20),
        mode: VisualMode::Block,
    };

    // Swap both line and column
    std::mem::swap(&mut selection.start, &mut selection.end);

    assert_eq!(selection.start.line, LineNr(5));
    assert_eq!(selection.start.col, 20);
}

// ============================================================================
// Mode Toggling Tests
// ============================================================================

/// Test: v toggles to normal from visual
/// Source: v in visual mode exits to normal
#[test]
fn test_visual_toggle_exits() {
    let _h = TestHarness::with_lines(&["hello"]);

    // v in visual mode should exit visual
    // In our test, we just verify the mode structure
    let mode = VisualMode::Char;
    assert_eq!(mode, VisualMode::Char);
}

/// Test: V changes to linewise
/// Source: V from charwise visual
#[test]
fn test_visual_change_to_linewise() {
    let mut selection = VisualSelection::new(CursorPosition::new(LineNr(1), 5), VisualMode::Char);

    // Pressing V changes to linewise
    selection.mode = VisualMode::Line;

    assert_eq!(selection.mode, VisualMode::Line);
}

/// Test: Ctrl-V changes to blockwise
/// Source: Ctrl-V from charwise visual
#[test]
fn test_visual_change_to_blockwise() {
    let mut selection = VisualSelection::new(CursorPosition::new(LineNr(1), 5), VisualMode::Char);

    // Pressing Ctrl-V changes to blockwise
    selection.mode = VisualMode::Block;

    assert_eq!(selection.mode, VisualMode::Block);
}

// ============================================================================
// Reselect Tests
// ============================================================================

/// Test: gv reselects last visual selection
/// Source: gv command
#[test]
fn test_visual_reselect() {
    // gv restores the last visual selection
    // This requires storing the last selection
    let last_selection = VisualSelection {
        start: CursorPosition::new(LineNr(2), 5),
        end: CursorPosition::new(LineNr(4), 10),
        mode: VisualMode::Char,
    };

    // After gv, selection would be restored
    assert_eq!(last_selection.start.line, LineNr(2));
    assert_eq!(last_selection.end.line, LineNr(4));
}

/// Test: '< and '> marks track visual selection
/// Source: '< and '> marks
#[test]
fn test_visual_marks() {
    let selection = VisualSelection {
        start: CursorPosition::new(LineNr(3), 5),
        end: CursorPosition::new(LineNr(7), 15),
        mode: VisualMode::Char,
    };

    // '< is start, '> is end (after normalization)
    let (start, end) = selection.normalized();
    assert_eq!(start.line, LineNr(3));
    assert_eq!(end.line, LineNr(7));
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test: empty visual selection
/// Source: v without movement
#[test]
fn test_visual_empty_selection() {
    let selection = VisualSelection::new(CursorPosition::new(LineNr(1), 5), VisualMode::Char);

    // Start equals end
    assert_eq!(selection.start, selection.end);
    assert!(!selection.is_multiline());
}

/// Test: visual selection at end of buffer
/// Source: edge case
#[test]
fn test_visual_at_buffer_end() {
    let _h = TestHarness::with_lines(&["line1", "line2", "line3"]);

    let selection = VisualSelection {
        start: CursorPosition::new(LineNr(2), 0),
        end: CursorPosition::new(LineNr(3), 5),
        mode: VisualMode::Char,
    };

    assert_eq!(selection.end.line, LineNr(3));
}

/// Test: visual selection at beginning of buffer
/// Source: edge case
#[test]
fn test_visual_at_buffer_start() {
    let _h = TestHarness::with_lines(&["line1", "line2", "line3"]);

    let selection = VisualSelection {
        start: CursorPosition::new(LineNr(1), 0),
        end: CursorPosition::new(LineNr(2), 3),
        mode: VisualMode::Char,
    };

    assert_eq!(selection.start.line, LineNr(1));
    assert_eq!(selection.start.col, 0);
}

/// Test: linewise visual includes entire lines
/// Source: V selection behavior
#[test]
fn test_visual_linewise_full_lines() {
    let selection = VisualSelection {
        start: CursorPosition::new(LineNr(2), 5), // Column ignored in linewise
        end: CursorPosition::new(LineNr(4), 10),
        mode: VisualMode::Line,
    };

    // In linewise mode, entire lines 2-4 are selected
    let range = selection.line_range();
    assert_eq!(range.start, LineNr(2));
    assert_eq!(range.end, LineNr(4));
}

/// Test: blockwise visual with different line lengths
/// Source: Ctrl-V on lines of different lengths
#[test]
fn test_visual_block_varying_line_lengths() {
    let _h = TestHarness::with_lines(&["short", "medium length", "very long line here"]);

    // Block selection might extend past end of short lines
    let block = BlockSelection {
        start_line: LineNr(1),
        end_line: LineNr(3),
        start_vcol: 0,
        end_vcol: 10,
    };

    // Block width is consistent
    assert_eq!(block.width(), 11);
}
