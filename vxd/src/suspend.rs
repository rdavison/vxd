//! Suspend/resume model.
//!
//! This module models the editor's suspended state.

use crate::types::VimResult;

/// Trait for suspend/resume behavior.
pub trait Suspender {
    /// Suspend the editor.
    fn suspend(&mut self) -> VimResult<()>;

    /// Resume the editor.
    fn resume(&mut self) -> VimResult<()>;

    /// Check if suspended.
    fn is_suspended(&self) -> bool;
}
