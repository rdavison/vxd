//! Buffer list tests (usr_22.4).

use vxd::buffer::{BufDeleteMode, Buffer, BufferManager};
use vxd_tui::buffer::TuiBufferManager;

#[test]
fn test_buffer_list_includes_current_and_created() {
    let mut mgr = TuiBufferManager::new();
    let current = mgr.current().handle();
    let a = mgr.create().unwrap();
    let b = mgr.create().unwrap();

    let list = mgr.list();
    assert!(list.contains(&current));
    assert!(list.contains(&a));
    assert!(list.contains(&b));
}

#[test]
fn test_list_listed_excludes_unlisted() {
    let mut mgr = TuiBufferManager::new();
    let handle = mgr.create().unwrap();
    assert!(mgr.list_listed().contains(&handle));

    mgr.delete(handle, BufDeleteMode::Unlist, false).unwrap();
    assert!(!mgr.list_listed().contains(&handle));
}

#[test]
fn test_get_by_name_finds_named_buffer() {
    let mut mgr = TuiBufferManager::new();
    let handle = mgr.create_named("notes.txt").unwrap();
    assert_eq!(mgr.get_by_name("notes.txt"), Some(handle));
}
