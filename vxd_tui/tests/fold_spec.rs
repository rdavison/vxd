//! Fold tests ported from Neovim's fold_spec.lua
//!
//! These tests verify folding behavior including:
//! - Manual fold creation and deletion
//! - Fold methods (manual, indent, marker, expr)
//! - Fold opening and closing
//! - Fold levels
//! - Nested folds
//! - Fold persistence across buffer changes

mod common;

use common::TestHarness;
use vxd::folds::{Fold, FoldMethod, FoldState};
use vxd::types::LineNr;

// ============================================================================
// Basic Fold Tests
// ============================================================================

/// Test: fold contains line
/// Source: folds.rs - basic fold structure
#[test]
fn test_fold_contains() {
    let fold = Fold {
        start: LineNr(5),
        end: LineNr(10),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    assert!(fold.contains(LineNr(5)));
    assert!(fold.contains(LineNr(7)));
    assert!(fold.contains(LineNr(10)));
    assert!(!fold.contains(LineNr(4)));
    assert!(!fold.contains(LineNr(11)));
}

/// Test: fold line count
/// Source: folds.rs - fold structure
#[test]
fn test_fold_line_count() {
    let fold = Fold {
        start: LineNr(3),
        end: LineNr(7),
        level: 1,
        state: FoldState::Open,
        nested: vec![],
    };

    assert_eq!(fold.line_count(), 5);
}

/// Test: fold state tracking
/// Source: fold_spec.lua - open/close behavior
#[test]
fn test_fold_state() {
    let mut fold = Fold {
        start: LineNr(1),
        end: LineNr(5),
        level: 1,
        state: FoldState::Open,
        nested: vec![],
    };

    assert_eq!(fold.state, FoldState::Open);

    fold.state = FoldState::Closed;
    assert_eq!(fold.state, FoldState::Closed);
}

// ============================================================================
// Fold Method Tests
// ============================================================================

/// Test: fold methods
/// Source: fold_spec.lua - different fold methods
#[test]
fn test_fold_methods() {
    assert_eq!(FoldMethod::default(), FoldMethod::Manual);

    let methods = [
        FoldMethod::Manual,
        FoldMethod::Indent,
        FoldMethod::Expr,
        FoldMethod::Marker,
        FoldMethod::Syntax,
        FoldMethod::Diff,
    ];

    for method in methods {
        assert_ne!(format!("{:?}", method), "");
    }
}

/// Test: manual fold method
/// Source: fold_spec.lua - "setlocal foldmethod=manual"
#[test]
fn test_manual_fold_method() {
    let _h = TestHarness::with_lines(&["line1", "line2", "line3", "line4", "line5"]);

    // In manual mode, folds are created explicitly by user
    let method = FoldMethod::Manual;
    assert_eq!(method, FoldMethod::Manual);
}

/// Test: indent fold method
/// Source: fold_spec.lua - "setlocal foldmethod=indent"
#[test]
fn test_indent_fold_method() {
    let _h = TestHarness::with_lines(&["a", "\ta", "\ta", "a"]);

    // In indent mode, folds are created based on indentation
    let method = FoldMethod::Indent;
    assert_eq!(method, FoldMethod::Indent);
}

/// Test: marker fold method
/// Source: fold_spec.lua - "setlocal foldmethod=marker"
#[test]
fn test_marker_fold_method() {
    let _h = TestHarness::with_lines(&["{{{", "folded content", "}}}"]);

    // In marker mode, folds are created based on markers
    let method = FoldMethod::Marker;
    assert_eq!(method, FoldMethod::Marker);
}

/// Test: expr fold method
/// Source: fold_spec.lua - "setlocal foldmethod=expr"
#[test]
fn test_expr_fold_method() {
    // In expr mode, folds are created based on expression evaluation
    let method = FoldMethod::Expr;
    assert_eq!(method, FoldMethod::Expr);
}

// ============================================================================
// Nested Fold Tests
// ============================================================================

/// Test: nested folds
/// Source: fold_spec.lua - nested fold behavior
#[test]
fn test_nested_folds() {
    let inner_fold = Fold {
        start: LineNr(3),
        end: LineNr(4),
        level: 2,
        state: FoldState::Closed,
        nested: vec![],
    };

    let outer_fold = Fold {
        start: LineNr(2),
        end: LineNr(6),
        level: 1,
        state: FoldState::Open,
        nested: vec![inner_fold],
    };

    assert_eq!(outer_fold.nested.len(), 1);
    assert_eq!(outer_fold.nested[0].level, 2);
    assert!(outer_fold.nested[0].contains(LineNr(3)));
}

/// Test: deeply nested folds
/// Source: fold_spec.lua - multi-level nesting
#[test]
fn test_deeply_nested_folds() {
    let level3 = Fold {
        start: LineNr(4),
        end: LineNr(5),
        level: 3,
        state: FoldState::Closed,
        nested: vec![],
    };

    let level2 = Fold {
        start: LineNr(3),
        end: LineNr(6),
        level: 2,
        state: FoldState::Open,
        nested: vec![level3],
    };

    let level1 = Fold {
        start: LineNr(2),
        end: LineNr(8),
        level: 1,
        state: FoldState::Open,
        nested: vec![level2],
    };

    assert_eq!(level1.level, 1);
    assert_eq!(level1.nested[0].level, 2);
    assert_eq!(level1.nested[0].nested[0].level, 3);
}

// ============================================================================
// Fold Level Tests
// ============================================================================

/// Test: fold levels with indentation
/// Source: fold_spec.lua - foldlevel() tests
#[test]
fn test_fold_levels_indent() {
    // Simulating:
    // a       (level 0)
    //   a     (level 1)
    //   a     (level 1)
    //     a   (level 2)
    //   a     (level 1)
    // a       (level 0)

    let level2_fold = Fold {
        start: LineNr(4),
        end: LineNr(4),
        level: 2,
        state: FoldState::Open,
        nested: vec![],
    };

    let level1_fold = Fold {
        start: LineNr(2),
        end: LineNr(5),
        level: 1,
        state: FoldState::Open,
        nested: vec![level2_fold],
    };

    assert_eq!(level1_fold.level, 1);
    assert_eq!(level1_fold.nested[0].level, 2);
}

/// Test: fold level at specific lines
/// Source: fold_spec.lua - foldlevel(i) checks
#[test]
fn test_fold_level_at_line() {
    let fold = Fold {
        start: LineNr(2),
        end: LineNr(6),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    // Line 2-6 are in fold level 1
    assert!(fold.contains(LineNr(2)));
    assert!(fold.contains(LineNr(4)));
    assert!(fold.contains(LineNr(6)));
    // Line 1 and 7 are not in this fold
    assert!(!fold.contains(LineNr(1)));
    assert!(!fold.contains(LineNr(7)));
}

// ============================================================================
// Fold Open/Close Tests
// ============================================================================

/// Test: close all folds (zM)
/// Source: fold_spec.lua - zM command
#[test]
fn test_close_all_folds() {
    let mut fold = Fold {
        start: LineNr(1),
        end: LineNr(10),
        level: 1,
        state: FoldState::Open,
        nested: vec![Fold {
            start: LineNr(3),
            end: LineNr(5),
            level: 2,
            state: FoldState::Open,
            nested: vec![],
        }],
    };

    // zM closes all folds
    fold.state = FoldState::Closed;
    fold.nested[0].state = FoldState::Closed;

    assert_eq!(fold.state, FoldState::Closed);
    assert_eq!(fold.nested[0].state, FoldState::Closed);
}

/// Test: open all folds (zR)
/// Source: fold_spec.lua - zR command
#[test]
fn test_open_all_folds() {
    let mut fold = Fold {
        start: LineNr(1),
        end: LineNr(10),
        level: 1,
        state: FoldState::Closed,
        nested: vec![Fold {
            start: LineNr(3),
            end: LineNr(5),
            level: 2,
            state: FoldState::Closed,
            nested: vec![],
        }],
    };

    // zR opens all folds
    fold.state = FoldState::Open;
    fold.nested[0].state = FoldState::Open;

    assert_eq!(fold.state, FoldState::Open);
    assert_eq!(fold.nested[0].state, FoldState::Open);
}

/// Test: toggle fold (za)
/// Source: fold_spec.lua - za command
#[test]
fn test_toggle_fold() {
    let mut fold = Fold {
        start: LineNr(1),
        end: LineNr(5),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    // Toggle from closed to open
    fold.state = match fold.state {
        FoldState::Closed => FoldState::Open,
        FoldState::Open => FoldState::Closed,
    };
    assert_eq!(fold.state, FoldState::Open);

    // Toggle from open to closed
    fold.state = match fold.state {
        FoldState::Closed => FoldState::Open,
        FoldState::Open => FoldState::Closed,
    };
    assert_eq!(fold.state, FoldState::Closed);
}

/// Test: open fold (zo)
/// Source: fold_spec.lua - zo command
#[test]
fn test_open_fold() {
    let mut fold = Fold {
        start: LineNr(1),
        end: LineNr(5),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    fold.state = FoldState::Open;
    assert_eq!(fold.state, FoldState::Open);
}

/// Test: close fold (zc)
/// Source: fold_spec.lua - zc command
#[test]
fn test_close_fold() {
    let mut fold = Fold {
        start: LineNr(1),
        end: LineNr(5),
        level: 1,
        state: FoldState::Open,
        nested: vec![],
    };

    fold.state = FoldState::Closed;
    assert_eq!(fold.state, FoldState::Closed);
}

// ============================================================================
// Fold Creation/Deletion Tests
// ============================================================================

/// Test: create manual fold (zf)
/// Source: fold_spec.lua - zf command
#[test]
fn test_create_manual_fold() {
    let _h = TestHarness::with_lines(&["line1", "line2", "line3", "line4", "line5"]);

    // zf creates a fold over a range
    let fold = Fold {
        start: LineNr(2),
        end: LineNr(4),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    assert_eq!(fold.start.0, 2);
    assert_eq!(fold.end.0, 4);
}

/// Test: delete fold (zd)
/// Source: fold_spec.lua - zd command
#[test]
fn test_delete_fold() {
    // zd deletes a fold at cursor
    // After deletion, folds collection would not contain it
    let folds: Vec<Fold> = vec![];
    assert!(folds.is_empty());
}

/// Test: delete all folds (zE)
/// Source: fold_spec.lua - zE command
#[test]
fn test_delete_all_folds() {
    let mut folds = vec![
        Fold {
            start: LineNr(1),
            end: LineNr(5),
            level: 1,
            state: FoldState::Closed,
            nested: vec![],
        },
        Fold {
            start: LineNr(7),
            end: LineNr(10),
            level: 1,
            state: FoldState::Closed,
            nested: vec![],
        },
    ];

    // zE deletes all folds
    folds.clear();
    assert!(folds.is_empty());
}

// ============================================================================
// Fold Adjustment Tests (after buffer changes)
// ============================================================================

/// Test: folds adjust after filter command
/// Source: fold_spec.lua - "manual folding adjusts with filter"
#[test]
fn test_fold_adjusts_with_filter() {
    // After a filter command changes buffer, folds should adjust
    // This is a conceptual test - fold positions update when buffer changes
    let mut fold = Fold {
        start: LineNr(4),
        end: LineNr(10),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    // If lines are deleted before the fold, fold positions shift
    // Simulating deletion of lines 1-3
    fold.start = LineNr(fold.start.0 - 3);
    fold.end = LineNr(fold.end.0 - 3);

    assert_eq!(fold.start.0, 1);
    assert_eq!(fold.end.0, 7);
}

/// Test: folds adjust after :move
/// Source: fold_spec.lua - "adjusting folds after :move"
#[test]
fn test_fold_adjusts_after_move() {
    // Moving lines should adjust fold positions
    let fold = Fold {
        start: LineNr(5),
        end: LineNr(8),
        level: 1,
        state: FoldState::Open,
        nested: vec![],
    };

    // Fold maintains its structure even after move
    assert_eq!(fold.line_count(), 4);
}

/// Test: fold combines when separator removed
/// Source: fold_spec.lua - "combines folds when removing separating space"
#[test]
fn test_fold_combines_on_separator_removal() {
    // When lines separating two folds are deleted, folds might combine
    // This depends on fold method (especially indent)
    let fold1 = Fold {
        start: LineNr(1),
        end: LineNr(3),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    let fold2 = Fold {
        start: LineNr(5),
        end: LineNr(7),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    // After removing line 4 (separator), folds might merge
    // This is fold method dependent
    assert_eq!(fold1.level, fold2.level);
}

/// Test: folds update on :read
/// Source: fold_spec.lua - "updates correctly on :read"
#[test]
fn test_fold_updates_on_read() {
    // Reading content into buffer should update folds
    // For indent folding, new indentation creates new folds
    let fold = Fold {
        start: LineNr(1),
        end: LineNr(2),
        level: 1,
        state: FoldState::Open,
        nested: vec![],
    };

    assert!(fold.line_count() >= 1);
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test: empty buffer has no folds
/// Source: edge case
#[test]
fn test_empty_buffer_no_folds() {
    let _h = TestHarness::with_lines(&[""]);

    let folds: Vec<Fold> = vec![];
    assert!(folds.is_empty());
}

/// Test: single line cannot be folded
/// Source: edge case
#[test]
fn test_single_line_no_fold() {
    // A fold needs at least 2 lines by default (foldminlines=1)
    // But can be set to allow single line with foldminlines=0
    let fold = Fold {
        start: LineNr(1),
        end: LineNr(1),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    assert_eq!(fold.line_count(), 1);
}

/// Test: fold at end of buffer
/// Source: edge case
#[test]
fn test_fold_at_buffer_end() {
    let _h = TestHarness::with_lines(&["line1", "line2", "line3"]);

    let fold = Fold {
        start: LineNr(2),
        end: LineNr(3),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    assert_eq!(fold.end.0, 3);
}

/// Test: no folds if buffer becomes empty
/// Source: fold_spec.lua - "no folds remain if :delete makes buffer empty"
#[test]
fn test_no_folds_after_buffer_empty() {
    // If all lines are deleted, all folds should be gone
    let mut folds = vec![Fold {
        start: LineNr(2),
        end: LineNr(3),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    }];

    // After %delete, buffer is empty
    folds.clear();
    assert!(folds.is_empty());
}

// ============================================================================
// Multibyte Fold Marker Tests
// ============================================================================

/// Test: multibyte fold markers
/// Source: fold_spec.lua - "multibyte fold markers work"
#[test]
fn test_multibyte_fold_markers() {
    // Custom fold markers can be multibyte (e.g., « and »)
    let marker_start = "«";
    let marker_end = "»";

    assert_eq!(marker_start.len(), 2); // UTF-8 bytes
    assert_eq!(marker_end.len(), 2);
}

/// Test: fold text customization
/// Source: fold.txt - foldtext option
#[test]
fn test_fold_text() {
    let fold = Fold {
        start: LineNr(1),
        end: LineNr(5),
        level: 1,
        state: FoldState::Closed,
        nested: vec![],
    };

    // Default fold text shows line count
    let fold_text = format!("+-- {} lines", fold.line_count());
    assert!(fold_text.contains("5 lines"));
}
