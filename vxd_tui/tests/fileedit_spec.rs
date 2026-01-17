//! Multi-file editing tests (usr_07.1-07.3).

use vxd::fileedit::FileEditor;
use vxd::types::VimError;
use vxd_tui::fileedit::TuiFileEditor;

#[test]
fn test_edit_adds_to_arglist() {
    let mut editor = TuiFileEditor::new();
    editor.edit("one.txt").unwrap();
    editor.edit("two.txt").unwrap();

    assert_eq!(editor.arglist(), &["one.txt".to_string(), "two.txt".to_string()]);
    assert_eq!(editor.current_file(), Some("two.txt"));
}

#[test]
fn test_edit_existing_file_keeps_order() {
    let mut editor = TuiFileEditor::new();
    editor.edit("one.txt").unwrap();
    editor.edit("two.txt").unwrap();
    editor.edit("one.txt").unwrap();

    assert_eq!(editor.arglist(), &["one.txt".to_string(), "two.txt".to_string()]);
    assert_eq!(editor.current_file(), Some("one.txt"));
}

#[test]
fn test_next_prev_file_navigation() {
    let mut editor = TuiFileEditor::new();
    editor.edit("one.txt").unwrap();
    editor.edit("two.txt").unwrap();
    editor.edit("three.txt").unwrap();

    editor.prev_file().unwrap();
    assert_eq!(editor.current_file(), Some("two.txt"));

    editor.prev_file().unwrap();
    assert_eq!(editor.current_file(), Some("one.txt"));

    editor.next_file().unwrap();
    assert_eq!(editor.current_file(), Some("two.txt"));
}

#[test]
fn test_next_prev_file_boundaries_error() {
    let mut editor = TuiFileEditor::new();
    editor.edit("one.txt").unwrap();

    let err = editor.prev_file().unwrap_err();
    assert_eq!(err, VimError::Error(1, "Already at first file".to_string()));

    let err = editor.next_file().unwrap_err();
    assert_eq!(err, VimError::Error(1, "Already at last file".to_string()));
}

#[test]
fn test_edit_rejects_empty_name() {
    let mut editor = TuiFileEditor::new();
    let err = editor.edit("").unwrap_err();
    assert_eq!(err, VimError::Error(1, "Empty file name".to_string()));
}
