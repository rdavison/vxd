//! Replace mode tests.

mod common;

use common::TestHarness;
use vxd::modes::Mode;

/// Test: replace overwrites character at cursor.
/// Source: Vim R behavior
#[test]
fn test_replace_overwrites_char() {
    let mut h = TestHarness::with_lines(&["hello"]);
    h.set_cursor(1, 1);

    h.feed("Ra");
    assert_eq!(h.content(), "hallo");
    assert_eq!(h.mode(), Mode::Replace);

    h.feed("<Esc>");
    assert_eq!(h.mode(), Mode::Normal);
}

/// Test: replace extends line when moving past end.
/// Source: Vim R behavior
#[test]
fn test_replace_extends_line() {
    let mut h = TestHarness::with_lines(&["hi"]);
    h.set_cursor(1, 1);

    h.feed("Rxy");
    assert_eq!(h.content(), "hxy");
}
