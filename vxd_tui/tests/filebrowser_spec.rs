//! File browser tests (usr_22.1).

use vxd::filebrowser::{BrowseSort, FileEntry, FileBrowser};
use vxd::types::VimError;
use vxd_tui::filebrowser::TuiFileBrowser;

#[test]
fn test_browser_dir_set_get() {
    let mut browser = TuiFileBrowser::new();
    browser.set_dir("/tmp").unwrap();
    assert_eq!(browser.dir(), "/tmp");
}

#[test]
fn test_browser_dir_rejects_empty() {
    let mut browser = TuiFileBrowser::new();
    let err = browser.set_dir("").unwrap_err();
    assert_eq!(err, VimError::Error(1, "Empty directory".to_string()));
}

#[test]
fn test_browser_sort_by_name() {
    let mut browser = TuiFileBrowser::new();
    browser.set_entries(vec![
        FileEntry {
            name: "b.txt".to_string(),
            is_dir: false,
            size: 20,
            mtime: 2,
        },
        FileEntry {
            name: "a.txt".to_string(),
            is_dir: false,
            size: 10,
            mtime: 1,
        },
    ]);

    let list = browser.list(BrowseSort::Name);
    assert_eq!(list[0].name, "a.txt");
    assert_eq!(list[1].name, "b.txt");
}
