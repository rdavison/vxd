//! Tab page management.
//!
//! Tab pages are collections of windows. Each tab has its own window layout.

use crate::types::*;
use crate::windows::WinHandle;

// ============================================================================
// Tab Types
// ============================================================================

/// Tab handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabHandle(pub usize);

impl TabHandle {
    /// Current tab handle (0 in API)
    pub const CURRENT: TabHandle = TabHandle(0);
}

/// Tab page information
#[derive(Debug, Clone)]
pub struct TabInfo {
    /// Tab handle
    pub handle: TabHandle,
    /// Windows in this tab
    pub windows: Vec<WinHandle>,
    /// Current window in this tab
    pub current_window: WinHandle,
}

// ============================================================================
// Tab Manager Trait
// ============================================================================

/// Manages tab pages
pub trait TabManager {
    /// Get the current tab
    fn current(&self) -> TabHandle;

    /// Get all tab handles
    fn list(&self) -> Vec<TabHandle>;

    /// Get info about a tab
    fn info(&self, tab: TabHandle) -> Option<TabInfo>;

    /// Create a new tab
    fn create(&mut self) -> VimResult<TabHandle>;

    /// Close a tab
    fn close(&mut self, tab: TabHandle, force: bool) -> VimResult<()>;

    /// Go to a specific tab
    fn go_to(&mut self, tab: TabHandle) -> VimResult<()>;

    /// Go to next tab
    fn next(&mut self) -> VimResult<()>;

    /// Go to previous tab
    fn prev(&mut self) -> VimResult<()>;

    /// Get number of tabs
    fn count(&self) -> usize {
        self.list().len()
    }

    /// Get windows in a tab
    fn windows(&self, tab: TabHandle) -> Vec<WinHandle>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Tab Behavioral Tests
        //!
        //! ## Key Quirks
        //!
        //! 1. **Tab-local state**: Each tab maintains its own window layout.
        //!
        //! 2. **Current window**: Each tab remembers which window was active.
        //!
        //! 3. **Close behavior**: Closing last tab with unsaved changes prompts.
    }
}
