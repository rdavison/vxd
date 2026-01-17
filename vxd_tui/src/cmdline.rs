//! Command-line implementation.

use std::collections::{HashMap, VecDeque};

use vxd::cmdline::{Cmdline, CmdlineHistory, CmdlineHistoryKind};
use vxd::types::VimResult;

/// Command-line implementation for the TUI.
#[derive(Debug, Default)]
pub struct TuiCmdline {
    text: String,
    history_limit: usize,
    history: HashMap<CmdlineHistoryKind, VecDeque<String>>,
}

impl TuiCmdline {
    /// Create a new command-line instance.
    pub fn new() -> Self {
        TuiCmdline {
            text: String::new(),
            history_limit: 100,
            history: HashMap::new(),
        }
    }

    fn history_mut(&mut self, kind: CmdlineHistoryKind) -> &mut VecDeque<String> {
        self.history.entry(kind).or_default()
    }
}

impl Cmdline for TuiCmdline {
    fn getcmdline(&self) -> &str {
        &self.text
    }

    fn setcmdline(&mut self, text: &str) -> VimResult<()> {
        self.text = text.to_string();
        Ok(())
    }
}

impl CmdlineHistory for TuiCmdline {
    fn history_limit(&self) -> usize {
        self.history_limit
    }

    fn set_history_limit(&mut self, limit: usize) {
        self.history_limit = limit.max(1);
        for deque in self.history.values_mut() {
            while deque.len() > self.history_limit {
                deque.pop_front();
            }
        }
    }

    fn hist_add(&mut self, kind: CmdlineHistoryKind, entry: &str) -> usize {
        let limit = self.history_limit;
        let deque = self.history_mut(kind);
        deque.push_back(entry.to_string());
        while deque.len() > limit {
            deque.pop_front();
        }
        1
    }

    fn hist_del(&mut self, kind: CmdlineHistoryKind, index: Option<i64>) -> usize {
        let deque = self.history_mut(kind);
        match index {
            None => {
                deque.clear();
                1
            }
            Some(idx) => {
                if deque.is_empty() {
                    return 0;
                }
                let len = deque.len() as i64;
                let actual = if idx < 0 { len + idx } else { idx };
                if actual < 0 || actual >= len {
                    return 0;
                }
                deque.remove(actual as usize);
                1
            }
        }
    }

    fn hist_get(&self, kind: CmdlineHistoryKind, index: i64) -> String {
        let deque = self.history.get(&kind);
        let Some(deque) = deque else {
            return String::new();
        };
        let len = deque.len() as i64;
        let actual = if index < 0 { len + index } else { index };
        if actual < 0 || actual >= len {
            return String::new();
        }
        deque[actual as usize].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vxd::cmdline::behavior::{CmdlineBehaviorTests, CmdlineHistoryBehaviorTests};
    use vxd::registers::RegisterContent;

    impl CmdlineBehaviorTests for TuiCmdline {}
    impl CmdlineHistoryBehaviorTests for TuiCmdline {}

    #[test]
    fn test_cmdline_behavior_contracts() {
        let mut cmdline = TuiCmdline::new();
        cmdline.test_paste_non_special_register_inserts_cr_between_lines();
        cmdline.test_paste_special_register_preserves_newlines();
    }

    #[test]
    fn test_cmdline_history_contracts() {
        let mut cmdline = TuiCmdline::new();
        cmdline.test_history_clear_start();

        let mut cmdline = TuiCmdline::new();
        cmdline.test_history_clear_end();

        let mut cmdline = TuiCmdline::new();
        cmdline.test_history_remove_item();
    }

    #[test]
    fn test_cmdline_basic_set_get() {
        let mut cmdline = TuiCmdline::new();
        cmdline.setcmdline(":write").unwrap();
        assert_eq!(cmdline.getcmdline(), ":write");
    }

    #[test]
    fn test_cmdline_paste_register_raw() {
        let mut cmdline = TuiCmdline::new();
        let content = RegisterContent::characterwise("foo\nbar\rbaz");
        cmdline.paste_register(&content, true).unwrap();
        assert_eq!(cmdline.getcmdline(), "foo\nbar\rbaz");
    }
}
