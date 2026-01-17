//! View-only buffer tests.

mod common;

use common::TestHarness;
use vxd::buffer::{Buffer, BufferManager};
use vxd::types::VimError;

/// Test: non-modifiable buffers reject changes.
/// Source: Vim :view behavior (nomodifiable)
#[test]
fn test_view_only_rejects_insert() {
    let mut h = TestHarness::with_lines(&["hello"]);
    h.editor
        .buffers
        .current_mut()
        .set_modifiable(false)
        .unwrap();

    h.editor.enter_insert().unwrap();
    let err = h.editor.insert_char('x').unwrap_err();
    assert_eq!(
        err,
        VimError::Error(21, "Cannot make changes, 'modifiable' is off".to_string())
    );
    assert_eq!(h.content(), "hello");
}
