//! Mode switching (Normal/Insert/Visual/Cmdline/etc).
//!
//! Vim is a modal editor with distinct modes that determine how
//! keystrokes are interpreted. This module defines the mode system.
//!
//! # Key Behavioral Contracts
//!
//! - Exactly one primary mode is active at any time
//! - Mode determines valid commands and key behaviors
//! - Mode transitions are deterministic
//! - Visual selection is cleared on exit to normal mode
//! - Some states are "blocking" (prevent RPC interruption)

use crate::types::*;

// ============================================================================
// Mode Types
// ============================================================================

/// The primary editing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Normal mode - command mode, cursor on characters
    Normal,
    /// Insert mode - text insertion
    Insert,
    /// Replace mode - overwrite existing characters
    Replace,
    /// Visual mode (characterwise, linewise, or blockwise)
    Visual(VisualMode),
    /// Select mode (like visual, but typing replaces selection)
    Select(VisualMode),
    /// Command-line mode (entering ex commands)
    CommandLine(CommandLineMode),
    /// Operator-pending mode (awaiting motion after operator)
    OperatorPending,
    /// Terminal mode (in embedded terminal)
    Terminal(TerminalMode),
}

impl Mode {
    /// Get the mode code as returned by nvim_get_mode()
    pub fn code(&self) -> &'static str {
        match self {
            Mode::Normal => "n",
            Mode::Insert => "i",
            Mode::Replace => "R",
            Mode::Visual(VisualMode::Char) => "v",
            Mode::Visual(VisualMode::Line) => "V",
            Mode::Visual(VisualMode::Block) => "\x16", // Ctrl-V
            Mode::Select(VisualMode::Char) => "s",
            Mode::Select(VisualMode::Line) => "S",
            Mode::Select(VisualMode::Block) => "\x13", // Ctrl-S
            Mode::CommandLine(_) => "c",
            Mode::OperatorPending => "no",
            Mode::Terminal(TerminalMode::Insert) => "t",
            Mode::Terminal(TerminalMode::Normal) => "nt",
        }
    }

    /// Get the display name for the mode (shown in status line)
    pub fn display_name(&self) -> &'static str {
        match self {
            Mode::Normal => "",
            Mode::Insert => "-- INSERT --",
            Mode::Replace => "-- REPLACE --",
            Mode::Visual(VisualMode::Char) => "-- VISUAL --",
            Mode::Visual(VisualMode::Line) => "-- VISUAL LINE --",
            Mode::Visual(VisualMode::Block) => "-- VISUAL BLOCK --",
            Mode::Select(VisualMode::Char) => "-- SELECT --",
            Mode::Select(VisualMode::Line) => "-- SELECT LINE --",
            Mode::Select(VisualMode::Block) => "-- SELECT BLOCK --",
            Mode::CommandLine(_) => "",
            Mode::OperatorPending => "",
            Mode::Terminal(TerminalMode::Insert) => "-- TERMINAL --",
            Mode::Terminal(TerminalMode::Normal) => "",
        }
    }

    /// Check if this mode allows text insertion
    pub fn allows_insertion(&self) -> bool {
        matches!(
            self,
            Mode::Insert | Mode::Replace | Mode::Terminal(TerminalMode::Insert)
        )
    }

    /// Check if this mode is a visual/select mode
    pub fn is_visual(&self) -> bool {
        matches!(self, Mode::Visual(_) | Mode::Select(_))
    }

    /// Check if cursor should be past-EOL capable
    pub fn allows_cursor_past_eol(&self) -> bool {
        matches!(self, Mode::Insert | Mode::Replace)
    }

    /// Get the UI mode name for cursor styling
    pub fn ui_mode_name(&self) -> &'static str {
        match self {
            Mode::Normal => "normal",
            Mode::Insert => "insert",
            Mode::Replace => "replace",
            Mode::Visual(_) => "visual",
            Mode::Select(_) => "visual_select",
            Mode::CommandLine(CommandLineMode::Normal) => "cmdline_normal",
            Mode::CommandLine(CommandLineMode::Insert) => "cmdline_insert",
            Mode::CommandLine(CommandLineMode::Replace) => "cmdline_replace",
            Mode::OperatorPending => "operator",
            Mode::Terminal(_) => "terminal",
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}

/// Visual mode sub-type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VisualMode {
    /// Characterwise visual mode (`v`)
    #[default]
    Char,
    /// Linewise visual mode (`V`)
    Line,
    /// Blockwise visual mode (`Ctrl-V`)
    Block,
}

impl VisualMode {
    /// Get the motion type for operators in this visual mode
    pub fn motion_type(&self) -> MotionType {
        match self {
            VisualMode::Char => MotionType::Characterwise,
            VisualMode::Line => MotionType::Linewise,
            VisualMode::Block => MotionType::Blockwise,
        }
    }
}

/// Command-line sub-mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CommandLineMode {
    /// Normal mode within command line
    #[default]
    Normal,
    /// Insert mode within command line
    Insert,
    /// Replace mode within command line
    Replace,
}

/// Terminal mode sub-type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TerminalMode {
    /// Terminal insert mode (input goes to child process)
    #[default]
    Insert,
    /// Terminal normal mode (can use vim commands)
    Normal,
}

// ============================================================================
// Mode State
// ============================================================================

/// Extended mode state including modifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeState {
    /// The primary mode
    pub mode: Mode,
    /// Whether we're in a blocking state (operator-pending, etc.)
    pub blocking: bool,
    /// Pending operator (if in operator-pending mode or visual with operator)
    pub pending_operator: Option<char>,
    /// Count prefix for pending operation
    pub count: Option<usize>,
    /// Temporary normal mode from insert (Ctrl-O)
    pub ctrl_o_mode: Option<CtrlOMode>,
}

impl ModeState {
    /// Create a new mode state in normal mode
    pub fn new() -> Self {
        ModeState {
            mode: Mode::Normal,
            blocking: false,
            pending_operator: None,
            count: None,
            ctrl_o_mode: None,
        }
    }

    /// Get the effective mode code (accounting for Ctrl-O states)
    pub fn effective_code(&self) -> String {
        match self.ctrl_o_mode {
            Some(CtrlOMode::FromInsert) => "niI".to_string(),
            Some(CtrlOMode::FromVisual) => "vs".to_string(),
            None => self.mode.code().to_string(),
        }
    }

    /// Check if in a blocking state
    pub fn is_blocking(&self) -> bool {
        self.blocking
    }
}

impl Default for ModeState {
    fn default() -> Self {
        ModeState::new()
    }
}

/// Ctrl-O temporary mode state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CtrlOMode {
    /// Temporary normal mode from insert mode
    FromInsert,
    /// Temporary normal mode from visual/select mode
    FromVisual,
}

// ============================================================================
// Mode Transition
// ============================================================================

/// A mode transition event
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeTransition {
    /// Mode before transition
    pub from: Mode,
    /// Mode after transition
    pub to: Mode,
    /// Key or command that triggered the transition
    pub trigger: Option<String>,
}

/// Error when a mode transition is not allowed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidTransition {
    /// Attempted source mode
    pub from: Mode,
    /// Attempted target mode
    pub to: Mode,
    /// Reason the transition was rejected
    pub reason: String,
}

// ============================================================================
// Mode Manager Trait
// ============================================================================

/// Manages mode state and transitions
pub trait ModeManager {
    /// Get the current mode
    fn mode(&self) -> Mode;

    /// Get the full mode state
    fn state(&self) -> &ModeState;

    /// Check if a transition to the target mode is allowed
    fn can_transition_to(&self, target: Mode) -> bool;

    /// Transition to a new mode
    ///
    /// Returns the transition that occurred, or an error if not allowed.
    fn transition_to(&mut self, target: Mode) -> Result<ModeTransition, InvalidTransition>;

    /// Enter insert mode
    fn enter_insert(&mut self) -> Result<ModeTransition, InvalidTransition> {
        self.transition_to(Mode::Insert)
    }

    /// Enter replace mode
    fn enter_replace(&mut self) -> Result<ModeTransition, InvalidTransition> {
        self.transition_to(Mode::Replace)
    }

    /// Enter visual mode (characterwise)
    fn enter_visual(&mut self) -> Result<ModeTransition, InvalidTransition> {
        self.transition_to(Mode::Visual(VisualMode::Char))
    }

    /// Enter visual line mode
    fn enter_visual_line(&mut self) -> Result<ModeTransition, InvalidTransition> {
        self.transition_to(Mode::Visual(VisualMode::Line))
    }

    /// Enter visual block mode
    fn enter_visual_block(&mut self) -> Result<ModeTransition, InvalidTransition> {
        self.transition_to(Mode::Visual(VisualMode::Block))
    }

    /// Enter command-line mode
    fn enter_cmdline(&mut self) -> Result<ModeTransition, InvalidTransition> {
        self.transition_to(Mode::CommandLine(CommandLineMode::Insert))
    }

    /// Return to normal mode
    fn escape_to_normal(&mut self) -> Result<ModeTransition, InvalidTransition> {
        self.transition_to(Mode::Normal)
    }

    /// Enter operator-pending mode with the given operator
    fn enter_operator_pending(
        &mut self,
        operator: char,
    ) -> Result<ModeTransition, InvalidTransition>;

    /// Exit operator-pending mode (after motion received)
    fn exit_operator_pending(&mut self) -> Result<ModeTransition, InvalidTransition>;

    /// Enter temporary normal mode (Ctrl-O from insert)
    fn enter_ctrl_o(&mut self) -> Result<(), InvalidTransition>;

    /// Exit temporary normal mode (return to insert)
    fn exit_ctrl_o(&mut self) -> Result<(), InvalidTransition>;

    /// Set the blocking state
    fn set_blocking(&mut self, blocking: bool);

    /// Set the count prefix
    fn set_count(&mut self, count: Option<usize>);

    /// Get the count prefix
    fn count(&self) -> Option<usize> {
        self.state().count
    }
}

// ============================================================================
// Mode Transition Rules
// ============================================================================

/// Check if a mode transition is generally valid
pub fn is_valid_transition(from: Mode, to: Mode) -> bool {
    use Mode::*;

    match (from, to) {
        // From Normal
        (Normal, Insert) => true,
        (Normal, Replace) => true,
        (Normal, Visual(_)) => true,
        (Normal, Select(_)) => true,
        (Normal, CommandLine(_)) => true,
        (Normal, OperatorPending) => true,
        (Normal, Terminal(TerminalMode::Normal)) => true,

        // From Insert
        (Insert, Normal) => true,
        (Insert, Replace) => true, // via Insert key

        // From Replace
        (Replace, Normal) => true,
        (Replace, Insert) => true, // via Insert key

        // From Visual modes
        (Visual(_), Normal) => true,
        (Visual(_), Visual(_)) => true, // switch visual type
        (Visual(_), Select(_)) => true,
        (Visual(_), OperatorPending) => true,
        (Visual(_), Insert) => true, // after operator like 'c'

        // From Select modes
        (Select(_), Normal) => true,
        (Select(_), Visual(_)) => true,
        (Select(_), Select(_)) => true, // switch select type
        (Select(_), Insert) => true,    // typing enters insert

        // From Command-line
        (CommandLine(_), Normal) => true,
        (CommandLine(_), CommandLine(_)) => true, // sub-mode switch

        // From Operator-pending
        (OperatorPending, Normal) => true,
        (OperatorPending, Visual(_)) => true, // some operators enter visual

        // From Terminal
        (Terminal(TerminalMode::Insert), Terminal(TerminalMode::Normal)) => true,
        (Terminal(TerminalMode::Normal), Terminal(TerminalMode::Insert)) => true,
        (Terminal(TerminalMode::Normal), Normal) => true,

        // Same mode is always allowed (no-op)
        (a, b) if a == b => true,

        _ => false,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_codes() {
        assert_eq!(Mode::Normal.code(), "n");
        assert_eq!(Mode::Insert.code(), "i");
        assert_eq!(Mode::Replace.code(), "R");
        assert_eq!(Mode::Visual(VisualMode::Char).code(), "v");
        assert_eq!(Mode::Visual(VisualMode::Line).code(), "V");
        assert_eq!(Mode::OperatorPending.code(), "no");
    }

    #[test]
    fn test_valid_transitions() {
        // Normal can go to insert
        assert!(is_valid_transition(Mode::Normal, Mode::Insert));

        // Insert can go back to normal
        assert!(is_valid_transition(Mode::Insert, Mode::Normal));

        // Visual can switch types
        assert!(is_valid_transition(
            Mode::Visual(VisualMode::Char),
            Mode::Visual(VisualMode::Line)
        ));

        // Terminal can toggle between insert and normal
        assert!(is_valid_transition(
            Mode::Terminal(TerminalMode::Insert),
            Mode::Terminal(TerminalMode::Normal)
        ));
    }

    #[test]
    fn test_mode_display_names() {
        assert_eq!(Mode::Insert.display_name(), "-- INSERT --");
        assert_eq!(
            Mode::Visual(VisualMode::Line).display_name(),
            "-- VISUAL LINE --"
        );
        assert_eq!(Mode::Normal.display_name(), ""); // Normal has no indicator
    }

    /// Behavioral tests for mode implementations
    pub trait ModeBehaviorTests: ModeManager + Sized {
        // ====================================================================
        // Basic Transition Tests
        // ====================================================================

        /// Test: Can enter insert mode from normal
        fn test_normal_to_insert(&mut self) {
            self.escape_to_normal().ok();
            assert_eq!(self.mode(), Mode::Normal);

            let result = self.enter_insert();
            assert!(result.is_ok());
            assert_eq!(self.mode(), Mode::Insert);
        }

        /// Test: Can return to normal from insert
        fn test_insert_to_normal(&mut self) {
            self.enter_insert().ok();
            assert_eq!(self.mode(), Mode::Insert);

            let result = self.escape_to_normal();
            assert!(result.is_ok());
            assert_eq!(self.mode(), Mode::Normal);
        }

        /// Test: Visual mode types can be switched
        fn test_visual_mode_switching(&mut self) {
            self.escape_to_normal().ok();

            // Enter characterwise visual
            self.enter_visual().ok();
            assert_eq!(self.mode(), Mode::Visual(VisualMode::Char));

            // Switch to linewise
            self.transition_to(Mode::Visual(VisualMode::Line)).ok();
            assert_eq!(self.mode(), Mode::Visual(VisualMode::Line));

            // Switch to blockwise
            self.transition_to(Mode::Visual(VisualMode::Block)).ok();
            assert_eq!(self.mode(), Mode::Visual(VisualMode::Block));
        }

        // ====================================================================
        // Blocking State Tests
        // ====================================================================

        /// Test: Operator-pending sets blocking state
        fn test_operator_pending_blocking(&mut self) {
            self.escape_to_normal().ok();
            self.enter_operator_pending('d').ok();

            assert!(
                self.state().is_blocking(),
                "Operator-pending should be blocking"
            );
        }

        // ====================================================================
        // Ctrl-O Tests
        // ====================================================================

        /// Test: Ctrl-O from insert gives niI mode code
        fn test_ctrl_o_from_insert(&mut self) {
            self.enter_insert().ok();
            self.enter_ctrl_o().ok();

            assert_eq!(
                self.state().effective_code(),
                "niI",
                "Ctrl-O from insert should give niI code"
            );
        }
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Mode Behavioral Tests
        //!
        //! These tests document expected Vim mode behavior from:
        //! - test/functional/ui/mode_spec.lua
        //! - test/functional/api/vim_spec.lua (nvim_get_mode)
        //! - test/functional/autocmd/modechanged_spec.lua
        //!
        //! ## Key Quirks
        //!
        //! 1. **Mode codes**: Some modes have multi-character codes
        //!    (e.g., "no" for operator-pending, "niI" for Ctrl-O from insert).
        //!
        //! 2. **Blocking states**: Operator-pending and some prompts block
        //!    RPC calls from interrupting.
        //!
        //! 3. **Visual mode clearing**: Exiting visual mode clears the
        //!    selection (unless entering insert via 'c' operator).
        //!
        //! 4. **ModeChanged event**: Fires on every mode transition with
        //!    old_mode and new_mode in v:event.
        //!
        //! 5. **Select mode**: Typing immediately enters insert mode and
        //!    replaces the selection.
    }
}
