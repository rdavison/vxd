//! Exit and quit command handling for the TUI.
//!
//! This module models core quit flows like :q, :q!, and ZZ.

use vxd::buffer::{Buffer, BufferManager};
use vxd::types::{VimError, VimResult};

use crate::editor::Editor;

const E37_NO_WRITE: &str = "No write since last change (add ! to override)";

/// Execute an ex-style quit command (":q", ":q!", ":quit").
///
/// Returns true if the editor should quit.
pub fn handle_ex_quit(editor: &mut Editor, cmdline: &str) -> VimResult<bool> {
    let cmd = cmdline.trim();
    let cmd = cmd.strip_prefix(':').unwrap_or(cmd).trim();
    if cmd.is_empty() {
        return Ok(false);
    }

    let (name, bang) = parse_bang(cmd);
    match name {
        "q" | "quit" => quit_if_allowed(editor, bang),
        _ => Err(VimError::NotEditorCommand(name.to_string())),
    }
}

/// Execute the normal-mode ZZ command (write and quit).
///
/// Returns true if the editor should quit.
pub fn handle_zz(editor: &mut Editor) -> VimResult<bool> {
    write_if_modified(editor)?;
    Ok(true)
}

fn quit_if_allowed(editor: &mut Editor, force: bool) -> VimResult<bool> {
    let modified = editor.buffers.current().is_modified();
    if modified && !force {
        return Err(VimError::Error(37, E37_NO_WRITE.to_string()));
    }
    Ok(true)
}

fn write_if_modified(editor: &mut Editor) -> VimResult<()> {
    if editor.buffers.current().is_modified() {
        editor.buffers.current_mut().set_modified(false)?;
    }
    Ok(())
}

fn parse_bang(cmd: &str) -> (&str, bool) {
    if let Some(stripped) = cmd.strip_suffix('!') {
        (stripped.trim_end(), true)
    } else {
        (cmd, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bang() {
        let (name, bang) = parse_bang("q!");
        assert_eq!(name, "q");
        assert!(bang);

        let (name, bang) = parse_bang("quit");
        assert_eq!(name, "quit");
        assert!(!bang);
    }
}
