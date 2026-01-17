//! Suspend/resume implementation for the TUI.

use vxd::suspend::Suspender;
use vxd::types::VimResult;

/// In-memory suspend state tracker.
#[derive(Debug, Default)]
pub struct TuiSuspender {
    suspended: bool,
}

impl TuiSuspender {
    /// Create a new suspender.
    pub fn new() -> Self {
        TuiSuspender { suspended: false }
    }
}

impl Suspender for TuiSuspender {
    fn suspend(&mut self) -> VimResult<()> {
        self.suspended = true;
        Ok(())
    }

    fn resume(&mut self) -> VimResult<()> {
        self.suspended = false;
        Ok(())
    }

    fn is_suspended(&self) -> bool {
        self.suspended
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspend_resume_toggle() {
        let mut suspender = TuiSuspender::new();
        assert!(!suspender.is_suspended());
        suspender.suspend().unwrap();
        assert!(suspender.is_suspended());
        suspender.resume().unwrap();
        assert!(!suspender.is_suspended());
    }
}
