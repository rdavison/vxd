//! Cmdline tests ported from Neovim's test/functional/editor/mode_cmdline_spec.lua.

use vxd::cmdline::{Cmdline, CmdlineHistory};
use vxd::cmdline::behavior::{CmdlineBehaviorTests, CmdlineHistoryBehaviorTests};
use vxd::types::VimResult;
use vxd_tui::cmdline::TuiCmdline;

struct CmdlineHarness {
    inner: TuiCmdline,
}

impl CmdlineHarness {
    fn new() -> Self {
        CmdlineHarness {
            inner: TuiCmdline::new(),
        }
    }
}

impl Cmdline for CmdlineHarness {
    fn getcmdline(&self) -> &str {
        self.inner.getcmdline()
    }

    fn setcmdline(&mut self, text: &str) -> VimResult<()> {
        self.inner.setcmdline(text)
    }
}

impl CmdlineHistory for CmdlineHarness {
    fn history_limit(&self) -> usize {
        self.inner.history_limit()
    }

    fn set_history_limit(&mut self, limit: usize) {
        self.inner.set_history_limit(limit);
    }

    fn hist_add(&mut self, kind: vxd::cmdline::CmdlineHistoryKind, entry: &str) -> usize {
        self.inner.hist_add(kind, entry)
    }

    fn hist_del(&mut self, kind: vxd::cmdline::CmdlineHistoryKind, index: Option<i64>) -> usize {
        self.inner.hist_del(kind, index)
    }

    fn hist_get(&self, kind: vxd::cmdline::CmdlineHistoryKind, index: i64) -> String {
        self.inner.hist_get(kind, index)
    }
}

impl CmdlineBehaviorTests for CmdlineHarness {}
impl CmdlineHistoryBehaviorTests for CmdlineHarness {}

/// Test: Ctrl-R pasting non-special registers inserts <CR> between lines.
/// Source: mode_cmdline_spec.lua (Ctrl-R)
#[test]
fn test_ctrl_r_non_special_registers() {
    let mut cmdline = CmdlineHarness::new();
    cmdline.test_paste_non_special_register_inserts_cr_between_lines();
}

/// Test: Ctrl-R pasting special registers preserves \n and \r.
/// Source: mode_cmdline_spec.lua (Ctrl-R)
#[test]
fn test_ctrl_r_special_registers() {
    let mut cmdline = CmdlineHarness::new();
    cmdline.test_paste_special_register_preserves_newlines();
}

/// Test: cmdline history clear and delete behavior.
/// Source: mode_cmdline_spec.lua (history)
#[test]
fn test_cmdline_history_behavior() {
    let mut cmdline = CmdlineHarness::new();
    cmdline.test_history_clear_start();

    let mut cmdline = CmdlineHarness::new();
    cmdline.test_history_clear_end();

    let mut cmdline = CmdlineHarness::new();
    cmdline.test_history_remove_item();
}

/// Test: history limit evicts oldest entries.
#[test]
fn test_cmdline_history_limit_eviction() {
    use vxd::cmdline::CmdlineHistoryKind;

    let mut cmdline = CmdlineHarness::new();
    cmdline.set_history_limit(2);
    assert_eq!(cmdline.history_limit(), 2);

    cmdline.hist_add(CmdlineHistoryKind::Command, "first");
    cmdline.hist_add(CmdlineHistoryKind::Command, "second");
    cmdline.hist_add(CmdlineHistoryKind::Command, "third");

    assert_eq!(cmdline.hist_get(CmdlineHistoryKind::Command, 0), "second");
    assert_eq!(cmdline.hist_get(CmdlineHistoryKind::Command, 1), "third");
    assert_eq!(cmdline.hist_get(CmdlineHistoryKind::Command, -1), "third");
}
