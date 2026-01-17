//! Exit flow tests based on usr_02 "Getting out".

use vxd::buffer::{Buffer, BufferManager};
use vxd::types::VimError;
use vxd_tui::editor::Editor;
use vxd_tui::exit::{handle_ex_quit, handle_zz};

#[test]
fn test_q_on_unmodified_quits() {
    let mut editor = Editor::new();
    let should_quit = handle_ex_quit(&mut editor, ":q").unwrap();
    assert!(should_quit);
}

#[test]
fn test_q_on_modified_errors() {
    let mut editor = Editor::new();
    editor
        .buffers
        .current_mut()
        .set_lines(0, -1, false, vec!["changed".to_string()])
        .unwrap();

    let err = handle_ex_quit(&mut editor, ":q").unwrap_err();
    assert_eq!(
        err,
        VimError::Error(37, "No write since last change (add ! to override)".to_string())
    );
}

#[test]
fn test_q_bang_on_modified_quits() {
    let mut editor = Editor::new();
    editor
        .buffers
        .current_mut()
        .set_lines(0, -1, false, vec!["changed".to_string()])
        .unwrap();

    let should_quit = handle_ex_quit(&mut editor, ":q!").unwrap();
    assert!(should_quit);
    assert!(editor.buffers.current().is_modified());
}

#[test]
fn test_zz_writes_and_quits() {
    let mut editor = Editor::new();
    editor
        .buffers
        .current_mut()
        .set_lines(0, -1, false, vec!["changed".to_string()])
        .unwrap();
    assert!(editor.buffers.current().is_modified());

    let should_quit = handle_zz(&mut editor).unwrap();
    assert!(should_quit);
    assert!(!editor.buffers.current().is_modified());
}

#[test]
fn test_wq_writes_and_quits() {
    let mut editor = Editor::new();
    editor
        .buffers
        .current_mut()
        .set_lines(0, -1, false, vec!["changed".to_string()])
        .unwrap();
    assert!(editor.buffers.current().is_modified());

    let should_quit = handle_ex_quit(&mut editor, ":wq").unwrap();
    assert!(should_quit);
    assert!(!editor.buffers.current().is_modified());
}

#[test]
fn test_x_writes_if_modified_and_quits() {
    let mut editor = Editor::new();
    editor
        .buffers
        .current_mut()
        .set_lines(0, -1, false, vec!["changed".to_string()])
        .unwrap();
    assert!(editor.buffers.current().is_modified());

    let should_quit = handle_ex_quit(&mut editor, ":x").unwrap();
    assert!(should_quit);
    assert!(!editor.buffers.current().is_modified());
}

#[test]
fn test_x_on_unmodified_quits_without_write() {
    let mut editor = Editor::new();
    assert!(!editor.buffers.current().is_modified());

    let should_quit = handle_ex_quit(&mut editor, ":x").unwrap();
    assert!(should_quit);
    assert!(!editor.buffers.current().is_modified());
}
