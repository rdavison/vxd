//! Matching bracket (%) tests.

mod common;

use common::TestHarness;

/// Test: % jumps from opening to matching closing bracket.
/// Source: Vim % behavior
#[test]
fn test_percent_matches_forward() {
    let mut h = TestHarness::with_lines(&["(a [b {c} d] e)"]);
    h.set_cursor(1, 0);

    h.feed("%");
    assert_eq!(h.cursor(), (1, 14));
}

/// Test: % jumps from closing to matching opening bracket.
/// Source: Vim % behavior
#[test]
fn test_percent_matches_backward() {
    let mut h = TestHarness::with_lines(&["(a [b {c} d] e)"]);
    h.set_cursor(1, 14);

    h.feed("%");
    assert_eq!(h.cursor(), (1, 0));
}

/// Test: % on a non-bracket does nothing.
/// Source: Vim % behavior
#[test]
fn test_percent_non_bracket_noop() {
    let mut h = TestHarness::with_lines(&["(ab)"]);
    h.set_cursor(1, 2);

    h.feed("%");
    assert_eq!(h.cursor(), (1, 2));
}
