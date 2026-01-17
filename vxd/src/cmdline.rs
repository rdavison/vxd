//! Command-line (Ex/search/input) behavior.
//!
//! This module models the ":" command-line UI state and history. It is intended
//! to capture observable behavior from Neovim's command-line tests.

use crate::registers::{RegisterContent, RegisterType};
use crate::types::VimResult;

/// Command-line history kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CmdlineHistoryKind {
    /// ":" command-line history.
    Command,
    /// "/" search history.
    SearchForward,
    /// "?" search history.
    SearchBackward,
    /// "=" expression history.
    Expression,
    /// "input()" history.
    Input,
}

/// Command-line buffer operations.
pub trait Cmdline {
    /// Get the current command-line text.
    fn getcmdline(&self) -> &str;

    /// Replace the current command-line text.
    fn setcmdline(&mut self, text: &str) -> VimResult<()>;

    /// Clear the command-line text.
    fn clear_cmdline(&mut self) -> VimResult<()> {
        self.setcmdline("")
    }

    /// Paste register content into the command-line.
    ///
    /// If `special` is true, the content is inserted verbatim (preserving
    /// both `\n` and `\r`). Otherwise, lines are joined with `\r`.
    fn paste_register(&mut self, content: &RegisterContent, special: bool) -> VimResult<()> {
        let insert = if special {
            content.as_string()
        } else {
            match content.reg_type {
                RegisterType::Linewise | RegisterType::Characterwise | RegisterType::Blockwise { .. } => {
                    content.text.join("\r")
                }
            }
        };

        let mut next = String::from(self.getcmdline());
        next.push_str(&insert);
        self.setcmdline(&next)
    }
}

/// Command-line history operations.
pub trait CmdlineHistory {
    /// Get the history limit.
    fn history_limit(&self) -> usize;

    /// Set the history limit.
    fn set_history_limit(&mut self, limit: usize);

    /// Add an entry to history. Returns 1 on success.
    fn hist_add(&mut self, kind: CmdlineHistoryKind, entry: &str) -> usize;

    /// Delete history entries.
    ///
    /// If `index` is None, clears the history and returns 1.
    /// If `index` is Some, removes that entry and returns 1 on success.
    fn hist_del(&mut self, kind: CmdlineHistoryKind, index: Option<i64>) -> usize;

    /// Get a history entry by index. Returns empty string if out of range.
    fn hist_get(&self, kind: CmdlineHistoryKind, index: i64) -> String;
}

// ============================================================================
// Behavioral Tests (portable)
// ============================================================================

/// Portable behavior checks derived from Neovim cmdline tests.
pub mod behavior {
    use super::*;
    use crate::registers::RegisterContent;

    /// Behavioral tests for cmdline implementations.
    pub trait CmdlineBehaviorTests: Cmdline + Sized {
        /// Non-special registers insert <CR> between lines.
        fn test_paste_non_special_register_inserts_cr_between_lines(&mut self) {
            self.clear_cmdline().unwrap();

            let linewise = RegisterContent::linewise(vec![
                "line1abc".into(),
                "line2somemoretext".into(),
            ]);
            self.paste_register(&linewise, false).unwrap();
            assert_eq!(self.getcmdline(), "line1abc\rline2somemoretext");

            self.clear_cmdline().unwrap();
            let charwise = RegisterContent {
                text: vec!["abc".into(), "line2".into()],
                reg_type: RegisterType::Characterwise,
            };
            self.paste_register(&charwise, false).unwrap();
            assert_eq!(self.getcmdline(), "abc\rline2");

            self.clear_cmdline().unwrap();
            let oneline = RegisterContent::linewise(vec!["line1abc".into()]);
            self.paste_register(&oneline, false).unwrap();
            assert_eq!(self.getcmdline(), "line1abc");
        }

        /// Special registers preserve \n and \r.
        fn test_paste_special_register_preserves_newlines(&mut self) {
            self.clear_cmdline().unwrap();
            let content = RegisterContent::characterwise("foo\nbar\rbaz");
            self.paste_register(&content, true).unwrap();
            assert_eq!(self.getcmdline(), "foo\nbar\rbaz");
        }
    }

    /// Behavioral tests for cmdline history.
    pub trait CmdlineHistoryBehaviorTests: CmdlineHistory + Sized {
        /// Clearing start of history should empty it.
        fn test_history_clear_start(&mut self) {
            assert_eq!(1, self.hist_add(CmdlineHistoryKind::Command, "foo"));
            assert_eq!(1, self.hist_del(CmdlineHistoryKind::Command, None));
            assert_eq!("", self.hist_get(CmdlineHistoryKind::Command, -1));
        }

        /// Clearing end of history with limit should empty it.
        fn test_history_clear_end(&mut self) {
            self.set_history_limit(1);
            assert_eq!(1, self.hist_add(CmdlineHistoryKind::Command, "foo"));
            assert_eq!(1, self.hist_del(CmdlineHistoryKind::Command, None));
            assert_eq!("", self.hist_get(CmdlineHistoryKind::Command, -1));
        }

        /// Removing an item should not corrupt history.
        fn test_history_remove_item(&mut self) {
            assert_eq!(1, self.hist_add(CmdlineHistoryKind::Command, "foo"));
            assert_eq!(1, self.hist_add(CmdlineHistoryKind::Command, "bar"));
            assert_eq!(1, self.hist_add(CmdlineHistoryKind::Command, "baz"));
            assert_eq!(1, self.hist_del(CmdlineHistoryKind::Command, Some(-2)));
            assert_eq!(1, self.hist_del(CmdlineHistoryKind::Command, None));
            assert_eq!("", self.hist_get(CmdlineHistoryKind::Command, -1));
        }
    }
}

