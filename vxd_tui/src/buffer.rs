//! Buffer implementation.
//!
//! This module provides a concrete implementation of the `vxd::buffer::Buffer` trait.

use vxd::buffer::{
    BufDeleteMode, BufHandle, BufHidden, Buffer, BufferLoadState, BufferManager, BufferType,
};
use vxd::types::{VimError, VimResult};

/// A concrete buffer implementation
#[derive(Debug, Clone)]
pub struct TuiBuffer {
    /// Buffer handle/number
    handle: BufHandle,
    /// Buffer name (file path or display name)
    name: String,
    /// Lines in the buffer
    lines: Vec<String>,
    /// Whether buffer has unsaved changes
    modified: bool,
    /// Whether buffer can be modified
    modifiable: bool,
    /// Whether buffer is read-only
    readonly: bool,
    /// Buffer type
    buftype: BufferType,
    /// Hidden behavior
    bufhidden: BufHidden,
    /// Load state
    load_state: BufferLoadState,
    /// Whether buffer is listed
    listed: bool,
    /// Change tick (version number)
    changedtick: u64,
}

impl TuiBuffer {
    /// Create a new empty buffer
    pub fn new(handle: BufHandle) -> Self {
        TuiBuffer {
            handle,
            name: String::new(),
            lines: vec![String::new()], // Always at least one line
            modified: false,
            modifiable: true,
            readonly: false,
            buftype: BufferType::Normal,
            bufhidden: BufHidden::UseGlobal,
            load_state: BufferLoadState::Loaded,
            listed: true,
            changedtick: 0,
        }
    }

    /// Create a buffer with a name
    pub fn with_name(handle: BufHandle, name: impl Into<String>) -> Self {
        let mut buf = Self::new(handle);
        buf.name = name.into();
        buf
    }

    /// Normalize a line index (handle negative indices)
    fn normalize_index(&self, idx: i64) -> usize {
        if idx < 0 {
            let len = self.lines.len() as i64;
            let normalized = len + idx + 1;
            if normalized < 0 {
                0
            } else {
                normalized as usize
            }
        } else {
            idx as usize
        }
    }

    /// Increment the changedtick
    fn bump_changedtick(&mut self) {
        self.changedtick += 1;
    }
}

impl Buffer for TuiBuffer {
    fn handle(&self) -> BufHandle {
        self.handle
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) -> VimResult<()> {
        self.name = name.to_string();
        Ok(())
    }

    fn is_valid(&self) -> bool {
        self.load_state != BufferLoadState::Wiped
    }

    fn line_count(&self) -> usize {
        if self.load_state == BufferLoadState::Unloaded {
            0
        } else {
            self.lines.len()
        }
    }

    fn get_lines(&self, start: i64, end: i64, strict_indexing: bool) -> VimResult<Vec<String>> {
        // Unloaded buffers return empty
        if self.load_state == BufferLoadState::Unloaded {
            return Ok(vec![]);
        }

        let len = self.lines.len();
        let start_idx = self.normalize_index(start);
        let end_idx = if end == -1 {
            len
        } else {
            self.normalize_index(end)
        };

        // Handle out of bounds
        if strict_indexing {
            if start_idx > len || end_idx > len {
                return Err(VimError::Error(5, "Index out of bounds".to_string()));
            }
        }

        // Clamp to valid range
        let start_idx = start_idx.min(len);
        let end_idx = end_idx.min(len);

        // Empty range
        if start_idx >= end_idx {
            return Ok(vec![]);
        }

        Ok(self.lines[start_idx..end_idx].to_vec())
    }

    fn set_lines(
        &mut self,
        start: i64,
        end: i64,
        strict_indexing: bool,
        replacement: Vec<String>,
    ) -> VimResult<()> {
        // Check modifiable
        if !self.modifiable {
            return Err(VimError::Error(
                21,
                "Cannot make changes, 'modifiable' is off".to_string(),
            ));
        }

        let len = self.lines.len();
        let start_idx = self.normalize_index(start);
        let end_idx = if end == -1 {
            len
        } else {
            self.normalize_index(end)
        };

        // Handle out of bounds in strict mode
        if strict_indexing {
            if start_idx > len || end_idx > len {
                return Err(VimError::Error(5, "Index out of bounds".to_string()));
            }
        }

        // Clamp to valid range
        let start_idx = start_idx.min(len);
        let end_idx = end_idx.max(start_idx).min(len);

        // Replace the lines
        let before: Vec<String> = self.lines.drain(start_idx..end_idx).collect();
        for (i, line) in replacement.into_iter().enumerate() {
            self.lines.insert(start_idx + i, line);
        }

        // Ensure at least one line
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        // Mark as modified
        self.modified = true;
        self.bump_changedtick();

        Ok(())
    }

    fn set_text(
        &mut self,
        start_row: i64,
        start_col: i64,
        end_row: i64,
        end_col: i64,
        replacement: Vec<String>,
    ) -> VimResult<()> {
        // Check modifiable
        if !self.modifiable {
            return Err(VimError::Error(
                21,
                "Cannot make changes, 'modifiable' is off".to_string(),
            ));
        }

        let start_row = self.normalize_index(start_row);
        let end_row = self.normalize_index(end_row);
        let start_col = start_col.max(0) as usize;
        let end_col = end_col.max(0) as usize;

        if start_row >= self.lines.len() || end_row >= self.lines.len() {
            return Err(VimError::Error(5, "Index out of bounds".to_string()));
        }

        // Get the text before and after the replacement region
        let prefix = if start_col <= self.lines[start_row].len() {
            self.lines[start_row][..start_col].to_string()
        } else {
            self.lines[start_row].clone()
        };

        let suffix = if end_col <= self.lines[end_row].len() {
            self.lines[end_row][end_col..].to_string()
        } else {
            String::new()
        };

        // Build the replacement
        let mut new_lines: Vec<String> = Vec::new();

        if replacement.is_empty() {
            // Just join prefix and suffix
            new_lines.push(format!("{}{}", prefix, suffix));
        } else if replacement.len() == 1 {
            // Single line replacement
            new_lines.push(format!("{}{}{}", prefix, replacement[0], suffix));
        } else {
            // Multi-line replacement
            new_lines.push(format!("{}{}", prefix, replacement[0]));
            for line in &replacement[1..replacement.len() - 1] {
                new_lines.push(line.clone());
            }
            new_lines.push(format!("{}{}", replacement[replacement.len() - 1], suffix));
        }

        // Remove the old lines and insert new ones
        for _ in start_row..=end_row {
            if start_row < self.lines.len() {
                self.lines.remove(start_row);
            }
        }
        for (i, line) in new_lines.into_iter().enumerate() {
            self.lines.insert(start_row + i, line);
        }

        // Ensure at least one line
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        self.modified = true;
        self.bump_changedtick();

        Ok(())
    }

    fn is_modified(&self) -> bool {
        self.modified
    }

    fn set_modified(&mut self, modified: bool) -> VimResult<()> {
        self.modified = modified;
        Ok(())
    }

    fn is_modifiable(&self) -> bool {
        self.modifiable
    }

    fn set_modifiable(&mut self, modifiable: bool) -> VimResult<()> {
        self.modifiable = modifiable;
        Ok(())
    }

    fn is_readonly(&self) -> bool {
        self.readonly
    }

    fn set_readonly(&mut self, readonly: bool) -> VimResult<()> {
        self.readonly = readonly;
        Ok(())
    }

    fn buftype(&self) -> BufferType {
        self.buftype
    }

    fn set_buftype(&mut self, buftype: BufferType) -> VimResult<()> {
        self.buftype = buftype;
        Ok(())
    }

    fn bufhidden(&self) -> BufHidden {
        self.bufhidden
    }

    fn set_bufhidden(&mut self, bufhidden: BufHidden) -> VimResult<()> {
        self.bufhidden = bufhidden;
        Ok(())
    }

    fn is_listed(&self) -> bool {
        self.listed
    }

    fn set_listed(&mut self, listed: bool) -> VimResult<()> {
        self.listed = listed;
        Ok(())
    }

    fn load_state(&self) -> BufferLoadState {
        self.load_state
    }

    fn changedtick(&self) -> u64 {
        self.changedtick
    }

    fn unload(&mut self) -> VimResult<()> {
        self.load_state = BufferLoadState::Unloaded;
        self.lines.clear();
        Ok(())
    }

    fn delete(&mut self, _force: bool) -> VimResult<()> {
        self.listed = false;
        self.unload()
    }

    fn wipe(&mut self, _force: bool) -> VimResult<()> {
        self.load_state = BufferLoadState::Wiped;
        self.lines.clear();
        Ok(())
    }
}

/// Buffer manager implementation
#[derive(Debug, Default)]
pub struct TuiBufferManager {
    /// All buffers
    buffers: Vec<TuiBuffer>,
    /// Current buffer index
    current: usize,
    /// Next handle to assign
    next_handle: usize,
}

impl TuiBufferManager {
    /// Create a new buffer manager with one empty buffer
    pub fn new() -> Self {
        let mut mgr = TuiBufferManager {
            buffers: Vec::new(),
            current: 0,
            next_handle: 1,
        };
        // Create initial buffer
        mgr.create().ok();
        mgr
    }
}

impl BufferManager for TuiBufferManager {
    type Buf = TuiBuffer;

    fn create(&mut self) -> VimResult<BufHandle> {
        let handle = BufHandle(self.next_handle);
        self.next_handle += 1;
        let buf = TuiBuffer::new(handle);
        self.buffers.push(buf);
        Ok(handle)
    }

    fn create_named(&mut self, name: &str) -> VimResult<BufHandle> {
        let handle = BufHandle(self.next_handle);
        self.next_handle += 1;
        let buf = TuiBuffer::with_name(handle, name);
        self.buffers.push(buf);
        Ok(handle)
    }

    fn get(&self, handle: BufHandle) -> Option<&Self::Buf> {
        if handle == BufHandle::CURRENT {
            self.buffers.get(self.current)
        } else {
            self.buffers.iter().find(|b| b.handle == handle)
        }
    }

    fn get_mut(&mut self, handle: BufHandle) -> Option<&mut Self::Buf> {
        if handle == BufHandle::CURRENT {
            self.buffers.get_mut(self.current)
        } else {
            self.buffers.iter_mut().find(|b| b.handle == handle)
        }
    }

    fn current(&self) -> &Self::Buf {
        &self.buffers[self.current]
    }

    fn current_mut(&mut self) -> &mut Self::Buf {
        &mut self.buffers[self.current]
    }

    fn set_current(&mut self, handle: BufHandle) -> VimResult<()> {
        if let Some(idx) = self.buffers.iter().position(|b| b.handle == handle) {
            self.current = idx;
            Ok(())
        } else {
            Err(VimError::BufferNotFound(vxd::types::BufferId(handle.0)))
        }
    }

    fn list(&self) -> Vec<BufHandle> {
        self.buffers
            .iter()
            .filter(|b| b.is_valid())
            .map(|b| b.handle)
            .collect()
    }

    fn list_listed(&self) -> Vec<BufHandle> {
        self.buffers
            .iter()
            .filter(|b| b.is_valid() && b.is_listed())
            .map(|b| b.handle)
            .collect()
    }

    fn delete(&mut self, handle: BufHandle, mode: BufDeleteMode, force: bool) -> VimResult<()> {
        if let Some(buf) = self.get_mut(handle) {
            match mode {
                BufDeleteMode::Unlist => {
                    buf.set_listed(false)?;
                }
                BufDeleteMode::Unload => {
                    buf.unload()?;
                }
                BufDeleteMode::Wipe => {
                    buf.wipe(force)?;
                }
            }
            Ok(())
        } else {
            Err(VimError::BufferNotFound(vxd::types::BufferId(handle.0)))
        }
    }

    fn get_by_name(&self, name: &str) -> Option<BufHandle> {
        self.buffers
            .iter()
            .find(|b| b.name == name && b.is_valid())
            .map(|b| b.handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vxd::buffer::behavior::BufferBehaviorTests;

    impl BufferBehaviorTests for TuiBuffer {}

    fn with_new_buffer(mut f: impl FnMut(&mut TuiBuffer)) {
        let mut buf = TuiBuffer::new(BufHandle(1));
        f(&mut buf);
    }

    #[test]
    fn test_new_buffer_has_one_line() {
        let buf = TuiBuffer::new(BufHandle(1));
        assert_eq!(buf.line_count(), 1);
    }

    #[test]
    fn test_cannot_have_zero_lines() {
        let mut buf = TuiBuffer::new(BufHandle(1));
        buf.set_lines(0, -1, false, vec!["line1".into(), "line2".into()])
            .unwrap();
        assert_eq!(buf.line_count(), 2);

        // Delete all lines
        buf.set_lines(0, -1, false, vec![]).unwrap();
        assert_eq!(buf.line_count(), 1);

        let lines = buf.get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec![""]);
    }

    #[test]
    fn test_empty_range_returns_empty() {
        let buf = TuiBuffer::new(BufHandle(1));
        let lines = buf.get_lines(0, 0, false).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_negative_index_end() {
        let mut buf = TuiBuffer::new(BufHandle(1));
        buf.set_lines(0, -1, false, vec!["a".into(), "b".into(), "c".into()])
            .unwrap();

        let lines = buf.get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_out_of_bounds_non_strict() {
        let buf = TuiBuffer::new(BufHandle(1));
        let lines = buf.get_lines(100, 200, false).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_out_of_bounds_strict() {
        let buf = TuiBuffer::new(BufHandle(1));
        let result = buf.get_lines(100, 200, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_at_beginning() {
        let mut buf = TuiBuffer::new(BufHandle(1));
        buf.set_lines(0, -1, false, vec!["original".into()])
            .unwrap();
        buf.set_lines(0, 0, false, vec!["inserted".into()]).unwrap();

        let lines = buf.get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["inserted", "original"]);
    }

    #[test]
    fn test_insert_at_end() {
        let mut buf = TuiBuffer::new(BufHandle(1));
        buf.set_lines(0, -1, false, vec!["original".into()])
            .unwrap();
        buf.set_lines(-1, -1, false, vec!["appended".into()])
            .unwrap();

        let lines = buf.get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["original", "appended"]);
    }

    #[test]
    fn test_replace_lines() {
        let mut buf = TuiBuffer::new(BufHandle(1));
        buf.set_lines(0, -1, false, vec!["a".into(), "b".into(), "c".into()])
            .unwrap();
        buf.set_lines(1, 2, false, vec!["B".into()]).unwrap();

        let lines = buf.get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["a", "B", "c"]);
    }

    #[test]
    fn test_buffer_behavior_contracts() {
        with_new_buffer(|buf| buf.test_new_buffer_has_one_line());
        with_new_buffer(|buf| buf.test_cannot_have_zero_lines());
        with_new_buffer(|buf| buf.test_empty_range_returns_empty());
        with_new_buffer(|buf| buf.test_negative_index_end());
        with_new_buffer(|buf| buf.test_negative_start_index());
        with_new_buffer(|buf| buf.test_out_of_bounds_non_strict());
        with_new_buffer(|buf| buf.test_out_of_bounds_strict());
        with_new_buffer(|buf| buf.test_insert_at_beginning());
        with_new_buffer(|buf| buf.test_insert_at_end());
        with_new_buffer(|buf| buf.test_replace_lines());
        with_new_buffer(|buf| buf.test_delete_lines());
        with_new_buffer(|buf| buf.test_new_buffer_not_modified());
        with_new_buffer(|buf| buf.test_set_lines_marks_modified());
        with_new_buffer(|buf| buf.test_clear_modified_flag());
        with_new_buffer(|buf| buf.test_non_modifiable_rejects_changes());
    }

    #[test]
    fn test_delete_lines() {
        let mut buf = TuiBuffer::new(BufHandle(1));
        buf.set_lines(0, -1, false, vec!["a".into(), "b".into(), "c".into()])
            .unwrap();
        buf.set_lines(1, 2, false, vec![]).unwrap();

        let lines = buf.get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["a", "c"]);
    }

    #[test]
    fn test_new_buffer_not_modified() {
        let buf = TuiBuffer::new(BufHandle(1));
        assert!(!buf.is_modified());
    }

    #[test]
    fn test_set_lines_marks_modified() {
        let mut buf = TuiBuffer::new(BufHandle(1));
        buf.set_lines(0, -1, false, vec!["content".into()]).unwrap();
        assert!(buf.is_modified());
    }

    #[test]
    fn test_clear_modified_flag() {
        let mut buf = TuiBuffer::new(BufHandle(1));
        buf.set_lines(0, -1, false, vec!["content".into()]).unwrap();
        buf.set_modified(false).unwrap();
        assert!(!buf.is_modified());
    }

    #[test]
    fn test_non_modifiable_rejects_changes() {
        let mut buf = TuiBuffer::new(BufHandle(1));
        buf.set_modifiable(false).unwrap();
        let result = buf.set_lines(0, -1, false, vec!["content".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_buffer_manager() {
        let mut mgr = TuiBufferManager::new();

        // Should have one buffer initially
        assert_eq!(mgr.list().len(), 1);

        // Create another
        let handle = mgr.create().unwrap();
        assert_eq!(mgr.list().len(), 2);

        // Set some content
        mgr.get_mut(handle)
            .unwrap()
            .set_lines(0, -1, false, vec!["test".into()])
            .unwrap();

        // Verify content
        let lines = mgr.get(handle).unwrap().get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["test"]);
    }
}
