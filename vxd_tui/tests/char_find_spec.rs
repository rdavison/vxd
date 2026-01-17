//! Character find motion tests (f/F/t/T/;/,).

mod common;

use common::TestHarness;

/// Test: f finds next occurrence on the line.
/// Source: Vim f{char} behavior
#[test]
fn test_find_char_forward() {
    let mut h = TestHarness::with_lines(&["one two three"]);
    h.set_cursor(1, 0);

    h.feed("ft");
    assert_eq!(h.cursor(), (1, 4));
}

/// Test: F finds previous occurrence on the line.
/// Source: Vim F{char} behavior
#[test]
fn test_find_char_backward() {
    let mut h = TestHarness::with_lines(&["one two three"]);
    h.set_cursor(1, 12);

    h.feed("Ft");
    assert_eq!(h.cursor(), (1, 8));
}

/// Test: t stops before the target.
/// Source: Vim t{char} behavior
#[test]
fn test_till_char_forward() {
    let mut h = TestHarness::with_lines(&["one two three"]);
    h.set_cursor(1, 0);

    h.feed("tt");
    assert_eq!(h.cursor(), (1, 3));
}

/// Test: T stops after the target when moving backward.
/// Source: Vim T{char} behavior
#[test]
fn test_till_char_backward() {
    let mut h = TestHarness::with_lines(&["one two three"]);
    h.set_cursor(1, 12);

    h.feed("Tt");
    assert_eq!(h.cursor(), (1, 9));
}

/// Test: ; repeats forward and , repeats backward.
/// Source: Vim ; and , behavior
#[test]
fn test_repeat_char_find() {
    let mut h = TestHarness::with_lines(&["ax bx cx"]);
    h.set_cursor(1, 0);

    h.feed("fx");
    assert_eq!(h.cursor(), (1, 1));

    h.feed(";");
    assert_eq!(h.cursor(), (1, 4));

    h.feed(",");
    assert_eq!(h.cursor(), (1, 1));
}
