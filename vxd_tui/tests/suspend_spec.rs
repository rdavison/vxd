//! Suspend/resume tests (usr_21.1).

use vxd::suspend::Suspender;
use vxd_tui::suspend::TuiSuspender;

#[test]
fn test_suspend_resume() {
    let mut suspender = TuiSuspender::new();
    assert!(!suspender.is_suspended());
    suspender.suspend().unwrap();
    assert!(suspender.is_suspended());
    suspender.resume().unwrap();
    assert!(!suspender.is_suspended());
}
