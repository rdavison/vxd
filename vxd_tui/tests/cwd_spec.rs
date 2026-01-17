//! Working directory tests (usr_22.2).

use vxd::cwd::WorkingDirectory;
use vxd::types::VimError;
use vxd_tui::cwd::TuiWorkingDirectory;

#[test]
fn test_cwd_default() {
    let wd = TuiWorkingDirectory::new();
    assert_eq!(wd.getcwd(), ".");
}

#[test]
fn test_cwd_set_get() {
    let mut wd = TuiWorkingDirectory::new();
    wd.setcwd("/tmp").unwrap();
    assert_eq!(wd.getcwd(), "/tmp");
}

#[test]
fn test_cwd_rejects_empty() {
    let mut wd = TuiWorkingDirectory::new();
    let err = wd.setcwd("").unwrap_err();
    assert_eq!(err, VimError::Error(1, "Empty path".to_string()));
}
