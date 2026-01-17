//! # vxd - Vim eXact Definition
//!
//! A trait-based specification of Vim's expected behaviors, derived from
//! Neovim's test suite. This crate provides:
//!
//! - Trait definitions that capture Vim's behavioral contracts
//! - Type stubs representing Vim's data model
//! - Tests that verify behavioral conformance (quirks and all)
//!
//! ## Design Philosophy
//!
//! Each feature is isolated behind a cargo feature flag, allowing implementations
//! to adopt Vim compatibility incrementally. The traits are designed to be
//! implementable by any editor that wishes to achieve Vim compatibility.
//!
//! ## Features
//!
//! - `buffer` - Buffer operations and text manipulation
//! - `cursor` - Cursor movement and positioning
//! - `modes` - Mode switching (Normal/Insert/Visual/Cmdline)
//! - `motions` - Movement commands (w, b, e, gg, G, etc.)
//! - `operators` - Operator commands (d, c, y, etc.)
//! - `registers` - Register system (yank/paste/named)
//! - `marks` - Mark system (local and global marks)
//! - `search` - Search and pattern matching
//! - `commands` - Ex command system
//! - `cmdline` - Command-line UI/history behaviors
//! - `options` - Option/setting system
//! - `autocmd` - Autocommand event system
//! - `windows` - Window management
//! - `tabs` - Tab page management
//! - `folds` - Folding system
//! - `completion` - Completion system
//! - `undo` - Undo/redo tree
//! - `visual` - Visual mode selections
//! - `textobjects` - Text objects (iw, aw, ip, etc.)

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ============================================================================
// Core Types (always available)
// ============================================================================

pub mod types;
pub use types::*;

// ============================================================================
// Feature Modules
// ============================================================================

#[cfg(feature = "buffer")]
pub mod buffer;

#[cfg(feature = "cursor")]
pub mod cursor;

#[cfg(feature = "modes")]
pub mod modes;

#[cfg(feature = "motions")]
pub mod motions;

#[cfg(feature = "operators")]
pub mod operators;

#[cfg(feature = "registers")]
pub mod registers;

#[cfg(feature = "marks")]
pub mod marks;

#[cfg(feature = "search")]
pub mod search;

#[cfg(feature = "commands")]
pub mod commands;

#[cfg(feature = "cmdline")]
pub mod cmdline;

#[cfg(feature = "options")]
pub mod options;

#[cfg(feature = "autocmd")]
pub mod autocmd;

#[cfg(feature = "windows")]
pub mod windows;

#[cfg(feature = "tabs")]
pub mod tabs;

#[cfg(feature = "folds")]
pub mod folds;

#[cfg(feature = "completion")]
pub mod completion;

#[cfg(feature = "undo")]
pub mod undo;

#[cfg(feature = "visual")]
pub mod visual;

#[cfg(feature = "textobjects")]
pub mod textobjects;

// ============================================================================
// Prelude - convenient imports for implementors
// ============================================================================

/// Convenient imports for implementing Vim-compatible editors
pub mod prelude {
    pub use crate::types::*;

    #[cfg(feature = "buffer")]
    pub use crate::buffer::Buffer;

    #[cfg(feature = "cursor")]
    pub use crate::cursor::Cursor;

    #[cfg(feature = "modes")]
    pub use crate::modes::{Mode, ModeManager};

    #[cfg(feature = "motions")]
    pub use crate::motions::Motion;

    #[cfg(feature = "operators")]
    pub use crate::operators::Operator;

    #[cfg(feature = "registers")]
    pub use crate::registers::RegisterBank;

    #[cfg(feature = "marks")]
    pub use crate::marks::MarkManager;

    #[cfg(feature = "search")]
    pub use crate::search::SearchEngine;

    #[cfg(feature = "commands")]
    pub use crate::commands::CommandExecutor;

    #[cfg(feature = "cmdline")]
    pub use crate::cmdline::{Cmdline, CmdlineHistory, CmdlineHistoryKind};

    #[cfg(feature = "options")]
    pub use crate::options::OptionManager;

    #[cfg(feature = "autocmd")]
    pub use crate::autocmd::AutocmdManager;

    #[cfg(feature = "windows")]
    pub use crate::windows::WindowManager;

    #[cfg(feature = "tabs")]
    pub use crate::tabs::TabManager;

    #[cfg(feature = "folds")]
    pub use crate::folds::FoldManager;

    #[cfg(feature = "completion")]
    pub use crate::completion::CompletionEngine;

    #[cfg(feature = "undo")]
    pub use crate::undo::UndoTree;

    #[cfg(feature = "visual")]
    pub use crate::visual::VisualSelection;

    #[cfg(feature = "textobjects")]
    pub use crate::textobjects::TextObject;
}
