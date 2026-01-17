//! Multi-file editing implementation for the TUI.

use vxd::buffer::{BufHandle, BufferManager};
use vxd::fileedit::FileEditor;
use vxd::types::{VimError, VimResult};

use crate::buffer::TuiBufferManager;

/// File editor that manages an argument list and buffers.
#[derive(Debug, Default)]
pub struct TuiFileEditor {
    buffers: TuiBufferManager,
    files: Vec<String>,
    current_idx: Option<usize>,
}

impl TuiFileEditor {
    /// Create a new file editor.
    pub fn new() -> Self {
        TuiFileEditor {
            buffers: TuiBufferManager::new(),
            files: Vec::new(),
            current_idx: None,
        }
    }

    fn select_handle_for_name(&mut self, name: &str) -> VimResult<BufHandle> {
        if let Some(handle) = self.buffers.get_by_name(name) {
            self.buffers.set_current(handle)?;
            Ok(handle)
        } else {
            let handle = self.buffers.create_named(name)?;
            self.buffers.set_current(handle)?;
            Ok(handle)
        }
    }
}

impl FileEditor for TuiFileEditor {
    fn edit(&mut self, name: &str) -> VimResult<()> {
        if name.trim().is_empty() {
            return Err(VimError::Error(1, "Empty file name".to_string()));
        }

        self.select_handle_for_name(name)?;
        if let Some(pos) = self.files.iter().position(|f| f == name) {
            self.current_idx = Some(pos);
        } else {
            self.files.push(name.to_string());
            self.current_idx = Some(self.files.len() - 1);
        }
        Ok(())
    }

    fn current_file(&self) -> Option<&str> {
        self.current_idx
            .and_then(|idx| self.files.get(idx))
            .map(|s| s.as_str())
    }

    fn arglist(&self) -> &[String] {
        &self.files
    }

    fn next_file(&mut self) -> VimResult<()> {
        let Some(idx) = self.current_idx else {
            return Err(VimError::Error(1, "No current file".to_string()));
        };
        if idx + 1 >= self.files.len() {
            return Err(VimError::Error(1, "Already at last file".to_string()));
        }
        let next = idx + 1;
        let name = self.files[next].clone();
        self.select_handle_for_name(&name)?;
        self.current_idx = Some(next);
        Ok(())
    }

    fn prev_file(&mut self) -> VimResult<()> {
        let Some(idx) = self.current_idx else {
            return Err(VimError::Error(1, "No current file".to_string()));
        };
        if idx == 0 {
            return Err(VimError::Error(1, "Already at first file".to_string()));
        }
        let prev = idx - 1;
        let name = self.files[prev].clone();
        self.select_handle_for_name(&name)?;
        self.current_idx = Some(prev);
        Ok(())
    }
}
