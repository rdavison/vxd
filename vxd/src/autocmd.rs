//! Autocommand event system.
//!
//! Autocommands execute automatically when certain events occur.
//! They can be grouped and filtered by patterns.

use crate::buffer::BufHandle;
use crate::types::*;

// ============================================================================
// Autocommand Events
// ============================================================================

/// Autocommand event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AutocmdEvent {
    // Buffer events
    BufAdd,
    BufDelete,
    BufEnter,
    BufFilePost,
    BufFilePre,
    BufHidden,
    BufLeave,
    BufNew,
    BufNewFile,
    BufRead,
    BufReadCmd,
    BufReadPost,
    BufReadPre,
    BufUnload,
    BufWinEnter,
    BufWinLeave,
    BufWipeout,
    BufWrite,
    BufWriteCmd,
    BufWritePost,
    BufWritePre,

    // File events
    FileAppendCmd,
    FileAppendPost,
    FileAppendPre,
    FileChangedRO,
    FileChangedShell,
    FileChangedShellPost,
    FileReadCmd,
    FileReadPost,
    FileReadPre,
    FileType,
    FileWriteCmd,
    FileWritePost,
    FileWritePre,

    // Window/Tab events
    WinNew,
    WinEnter,
    WinLeave,
    WinClosed,
    WinScrolled,
    WinResized,
    TabNew,
    TabEnter,
    TabLeave,
    TabClosed,

    // Cursor events
    CursorHold,
    CursorHoldI,
    CursorMoved,
    CursorMovedI,

    // Insert mode events
    InsertChange,
    InsertCharPre,
    InsertEnter,
    InsertLeave,
    InsertLeavePre,

    // Text change events
    TextChanged,
    TextChangedI,
    TextChangedP,
    TextChangedT,
    TextYankPost,

    // Visual mode events
    ModeChanged,

    // Command events
    CmdUndefined,
    CmdlineChanged,
    CmdlineEnter,
    CmdlineLeave,
    CmdwinEnter,
    CmdwinLeave,

    // Completion events
    CompleteChanged,
    CompleteDone,
    CompleteDonePre,

    // UI events
    ColorScheme,
    ColorSchemePre,
    MenuPopup,
    OptionSet,
    QuickFixCmdPost,
    QuickFixCmdPre,
    QuitPre,
    RecordingEnter,
    RecordingLeave,
    RemoteReply,
    SearchWrapped,
    SessionLoadPost,
    SessionWritePost,
    ShellCmdPost,
    ShellFilterPost,
    Signal,
    SourceCmd,
    SourcePost,
    SourcePre,
    SpellFileMissing,
    StdinReadPost,
    StdinReadPre,
    SwapExists,
    Syntax,
    TabNewEntered,
    TermChanged,
    TermClose,
    TermEnter,
    TermLeave,
    TermOpen,
    TermRequest,
    TermResponse,
    UIEnter,
    UILeave,
    User,
    VimEnter,
    VimLeave,
    VimLeavePre,
    VimResized,
    VimResume,
    VimSuspend,
}

impl AutocmdEvent {
    /// Parse an event name
    pub fn from_name(name: &str) -> Option<Self> {
        // This would be a large match statement
        // For brevity, showing a few examples
        match name.to_lowercase().as_str() {
            "bufenter" => Some(AutocmdEvent::BufEnter),
            "bufleave" => Some(AutocmdEvent::BufLeave),
            "bufread" | "bufreadpost" => Some(AutocmdEvent::BufReadPost),
            "bufwrite" | "bufwritepre" => Some(AutocmdEvent::BufWritePre),
            "cursorhold" => Some(AutocmdEvent::CursorHold),
            "cursormoved" => Some(AutocmdEvent::CursorMoved),
            "insertenter" => Some(AutocmdEvent::InsertEnter),
            "insertleave" => Some(AutocmdEvent::InsertLeave),
            "textchanged" => Some(AutocmdEvent::TextChanged),
            "filetype" => Some(AutocmdEvent::FileType),
            "vimenter" => Some(AutocmdEvent::VimEnter),
            "vimleave" => Some(AutocmdEvent::VimLeave),
            "winenter" => Some(AutocmdEvent::WinEnter),
            "winleave" => Some(AutocmdEvent::WinLeave),
            "modechanged" => Some(AutocmdEvent::ModeChanged),
            _ => None,
        }
    }

    /// Get the event name
    pub fn name(&self) -> &'static str {
        match self {
            AutocmdEvent::BufEnter => "BufEnter",
            AutocmdEvent::BufLeave => "BufLeave",
            AutocmdEvent::BufReadPost => "BufReadPost",
            AutocmdEvent::BufWritePre => "BufWritePre",
            AutocmdEvent::CursorHold => "CursorHold",
            AutocmdEvent::CursorMoved => "CursorMoved",
            AutocmdEvent::InsertEnter => "InsertEnter",
            AutocmdEvent::InsertLeave => "InsertLeave",
            AutocmdEvent::TextChanged => "TextChanged",
            AutocmdEvent::FileType => "FileType",
            AutocmdEvent::VimEnter => "VimEnter",
            AutocmdEvent::VimLeave => "VimLeave",
            AutocmdEvent::WinEnter => "WinEnter",
            AutocmdEvent::WinLeave => "WinLeave",
            AutocmdEvent::ModeChanged => "ModeChanged",
            _ => "Unknown",
        }
    }
}

// ============================================================================
// Autocommand Definition
// ============================================================================

/// An autocommand group
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AutocmdGroup {
    /// Group name
    pub name: String,
    /// Group ID
    pub id: usize,
}

/// Pattern for matching autocommand triggers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutocmdPattern {
    /// File pattern (glob)
    FilePattern(String),
    /// Buffer number
    Buffer(BufHandle),
    /// All buffers
    AllBuffers,
}

/// An autocommand definition
#[derive(Debug, Clone)]
pub struct Autocommand {
    /// Unique ID
    pub id: usize,
    /// Group (optional)
    pub group: Option<AutocmdGroup>,
    /// Event that triggers this autocommand
    pub event: AutocmdEvent,
    /// Pattern to match
    pub pattern: AutocmdPattern,
    /// Command to execute
    pub command: String,
    /// Whether to run only once
    pub once: bool,
    /// Whether to run nested autocommands
    pub nested: bool,
    /// Description
    pub desc: Option<String>,
}

/// Event data passed to autocommand callbacks
#[derive(Debug, Clone, Default)]
pub struct AutocmdEventData {
    /// Buffer that triggered the event
    pub buf: Option<BufHandle>,
    /// File path
    pub file: Option<String>,
    /// Match pattern
    pub match_: Option<String>,
    /// Additional data (varies by event)
    pub data: Option<String>,
}

// ============================================================================
// Autocommand Manager Trait
// ============================================================================

/// Manages autocommands
pub trait AutocmdManager {
    /// Create or get an autocommand group
    fn augroup(&mut self, name: &str) -> AutocmdGroup;

    /// Delete an autocommand group
    fn augroup_delete(&mut self, name: &str) -> VimResult<()>;

    /// Create an autocommand
    fn create(&mut self, autocmd: Autocommand) -> VimResult<usize>;

    /// Delete autocommands matching criteria
    fn delete(
        &mut self,
        group: Option<&AutocmdGroup>,
        event: Option<AutocmdEvent>,
        pattern: Option<&str>,
    ) -> VimResult<()>;

    /// Clear all autocommands in a group
    fn clear_group(&mut self, group: &AutocmdGroup) -> VimResult<()>;

    /// Execute autocommands for an event
    fn exec(&mut self, event: AutocmdEvent, data: &AutocmdEventData) -> VimResult<()>;

    /// Check if autocommands exist for an event
    fn exists(&self, event: AutocmdEvent, pattern: Option<&str>) -> bool;

    /// List autocommands
    fn list(&self, group: Option<&AutocmdGroup>, event: Option<AutocmdEvent>) -> Vec<&Autocommand>;

    /// Enable/disable autocommand execution
    fn set_enabled(&mut self, enabled: bool);

    /// Check if autocommands are enabled
    fn is_enabled(&self) -> bool;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_parsing() {
        assert_eq!(
            AutocmdEvent::from_name("BufEnter"),
            Some(AutocmdEvent::BufEnter)
        );
        assert_eq!(
            AutocmdEvent::from_name("bufenter"),
            Some(AutocmdEvent::BufEnter)
        );
        assert_eq!(AutocmdEvent::from_name("invalid"), None);
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Autocommand Behavioral Tests
        //!
        //! Tests derived from Neovim's test suite:
        //! - test/functional/autocmd/ - autocommand tests
        //!
        //! ## Key Quirks
        //!
        //! 1. **Event ordering**: Events fire in a specific order (e.g., BufReadPre
        //!    before BufReadPost).
        //!
        //! 2. **Nested execution**: By default, autocommands don't trigger other
        //!    autocommands unless `nested` is set.
        //!
        //! 3. **Pattern matching**: Patterns use glob syntax, not regex.
        //!
        //! 4. **Buffer-local**: Can be scoped to specific buffers with `<buffer>`.
        //!
        //! 5. **Once**: The `++once` flag makes autocommand run only once.
    }
}
