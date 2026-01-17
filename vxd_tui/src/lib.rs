//! # vxd_tui - TUI Implementation of vxd Traits
//!
//! This crate provides concrete implementations of the traits defined in `vxd`,
//! creating a working Vim-compatible editor.
//!
//! The implementation aims for exact Vim compatibility, validated against
//! tests derived from Neovim's test suite.

pub mod buffer;
pub mod cursor;
pub mod editor;
pub mod marks;
pub mod modes;
pub mod registers;

pub use editor::Editor;

/// Retry queue for tests that need to be revisited
pub mod retry {
    use std::collections::VecDeque;

    /// A FIFO queue for tests that are blocked or need revisiting
    #[derive(Debug, Default)]
    pub struct RetryQueue {
        queue: VecDeque<String>,
    }

    impl RetryQueue {
        /// Create a new empty retry queue
        pub fn new() -> Self {
            RetryQueue {
                queue: VecDeque::new(),
            }
        }

        /// Add a test to the retry queue
        pub fn push(&mut self, test_name: impl Into<String>) {
            self.queue.push_back(test_name.into());
        }

        /// Get the next test to retry
        pub fn pop(&mut self) -> Option<String> {
            self.queue.pop_front()
        }

        /// Check if queue is empty
        pub fn is_empty(&self) -> bool {
            self.queue.is_empty()
        }

        /// Get number of tests waiting
        pub fn len(&self) -> usize {
            self.queue.len()
        }

        /// View all tests in queue
        pub fn peek_all(&self) -> Vec<&str> {
            self.queue.iter().map(|s| s.as_str()).collect()
        }
    }
}
