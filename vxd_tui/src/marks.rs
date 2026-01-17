//! Marks implementation.
//!
//! This module provides a concrete implementation of Vim's mark system.

use std::collections::HashMap;
use vxd::buffer::BufHandle;
use vxd::cursor::CursorPosition;
use vxd::marks::{ChangeEntry, ChangeList, JumpEntry, JumpList, Mark, MarkManager, MarkValue};
use vxd::types::{LineNr, VimError, VimResult};

/// Jump list implementation
#[derive(Debug, Clone, Default)]
pub struct TuiJumpList {
    entries: Vec<JumpEntry>,
    position: usize,
}

impl JumpList for TuiJumpList {
    fn position(&self) -> usize {
        self.position
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn get(&self, index: usize) -> Option<&JumpEntry> {
        self.entries.get(index)
    }

    fn push(&mut self, entry: JumpEntry) {
        // Remove entries after current position
        self.entries.truncate(self.position);
        self.entries.push(entry);
        self.position = self.entries.len();

        // Limit size
        const MAX_JUMP_LIST: usize = 100;
        if self.entries.len() > MAX_JUMP_LIST {
            self.entries.remove(0);
            self.position = self.position.saturating_sub(1);
        }
    }

    fn go_older(&mut self) -> Option<&JumpEntry> {
        if self.position > 0 {
            self.position -= 1;
            self.entries.get(self.position)
        } else {
            None
        }
    }

    fn go_newer(&mut self) -> Option<&JumpEntry> {
        if self.position < self.entries.len() {
            let entry = self.entries.get(self.position);
            self.position += 1;
            entry
        } else {
            None
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.position = 0;
    }
}

/// Change list implementation
#[derive(Debug, Clone, Default)]
pub struct TuiChangeList {
    entries: Vec<ChangeEntry>,
    position: usize,
}

impl ChangeList for TuiChangeList {
    fn position(&self) -> usize {
        self.position
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn get(&self, index: usize) -> Option<&ChangeEntry> {
        self.entries.get(index)
    }

    fn push(&mut self, entry: ChangeEntry) {
        self.entries.push(entry);
        self.position = self.entries.len();

        // Limit size
        const MAX_CHANGE_LIST: usize = 100;
        if self.entries.len() > MAX_CHANGE_LIST {
            self.entries.remove(0);
            self.position = self.position.saturating_sub(1);
        }
    }

    fn go_older(&mut self) -> Option<&ChangeEntry> {
        if self.position > 0 {
            self.position -= 1;
            self.entries.get(self.position)
        } else {
            None
        }
    }

    fn go_newer(&mut self) -> Option<&ChangeEntry> {
        if self.position < self.entries.len() {
            let entry = self.entries.get(self.position);
            self.position += 1;
            entry
        } else {
            None
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.position = 0;
    }
}

/// Mark manager implementation
#[derive(Debug, Clone)]
pub struct TuiMarkManager {
    /// Local marks (a-z)
    local_marks: HashMap<char, MarkValue>,
    /// Global marks (A-Z, 0-9)
    global_marks: HashMap<char, MarkValue>,
    /// Special marks
    special_marks: HashMap<Mark, MarkValue>,
    /// Jump list
    jump_list: TuiJumpList,
    /// Change list
    change_list: TuiChangeList,
    /// Current buffer (for local marks context)
    current_buffer: BufHandle,
}

impl Default for TuiMarkManager {
    fn default() -> Self {
        TuiMarkManager {
            local_marks: HashMap::new(),
            global_marks: HashMap::new(),
            special_marks: HashMap::new(),
            jump_list: TuiJumpList::default(),
            change_list: TuiChangeList::default(),
            current_buffer: BufHandle(1),
        }
    }
}

impl TuiMarkManager {
    /// Create a new mark manager
    pub fn new() -> Self {
        TuiMarkManager::default()
    }

    /// Set the current buffer context
    pub fn set_current_buffer(&mut self, buf: BufHandle) {
        self.current_buffer = buf;
    }

    /// Push a jump entry, coalescing duplicates at the current history position.
    pub fn push_jump(&mut self, buffer: BufHandle, position: CursorPosition) {
        let entry = JumpEntry {
            buffer,
            position,
            file: None,
        };
        let should_skip = if self.jump_list.position > 0 {
            let last_index = self.jump_list.position - 1;
            self.jump_list
                .entries
                .get(last_index)
                .map(|last| last.buffer == entry.buffer && last.position == entry.position)
                .unwrap_or(false)
        } else {
            false
        };
        if should_skip {
            return;
        }

        self.jump_list.push(entry);
    }

    /// Jump back (Ctrl-O) and return the target cursor position.
    pub fn jump_back(
        &mut self,
        _buffer: BufHandle,
        _current: CursorPosition,
    ) -> Option<CursorPosition> {
        self.jump_list.go_older().map(|entry| entry.position)
    }

    /// Jump forward (Ctrl-I) and return the target cursor position.
    pub fn jump_forward(&mut self) -> Option<CursorPosition> {
        self.jump_list.go_newer().map(|entry| entry.position)
    }

    /// Return a copy of the current jump list entries.
    pub fn jump_list_entries(&self) -> Vec<JumpEntry> {
        self.jump_list.entries.clone()
    }

    /// Return the current jump list position.
    pub fn jump_list_position(&self) -> usize {
        self.jump_list.position()
    }

    /// Clear the jump list.
    pub fn clear_jump_list(&mut self) {
        self.jump_list.clear();
    }
}

impl MarkManager for TuiMarkManager {
    fn get(&self, mark: Mark) -> Option<&MarkValue> {
        match mark {
            Mark::Local(c) => self.local_marks.get(&c),
            Mark::Global(c) => self.global_marks.get(&c),
            Mark::Numbered(n) => self.global_marks.get(&((b'0' + n) as char)),
            _ => self.special_marks.get(&mark),
        }
    }

    fn set(&mut self, mark: Mark, value: MarkValue) -> VimResult<()> {
        if mark.is_readonly() {
            return Err(VimError::Error(
                1,
                format!("Cannot set read-only mark '{}'", mark.to_char()),
            ));
        }

        match mark {
            Mark::Local(c) => {
                self.local_marks.insert(c, value);
            }
            Mark::Global(c) => {
                self.global_marks.insert(c, value);
            }
            Mark::Numbered(n) => {
                self.global_marks.insert((b'0' + n) as char, value);
            }
            _ => {
                self.special_marks.insert(mark, value);
            }
        }

        Ok(())
    }

    fn delete(&mut self, mark: Mark) -> VimResult<()> {
        match mark {
            Mark::Local(c) => {
                self.local_marks.remove(&c);
            }
            Mark::Global(c) => {
                self.global_marks.remove(&c);
            }
            _ => {
                self.special_marks.remove(&mark);
            }
        }
        Ok(())
    }

    fn list(&self) -> Vec<(Mark, &MarkValue)> {
        let mut result = Vec::new();

        for (c, v) in &self.local_marks {
            result.push((Mark::Local(*c), v));
        }

        for (c, v) in &self.global_marks {
            if c.is_ascii_uppercase() {
                result.push((Mark::Global(*c), v));
            } else if c.is_ascii_digit() {
                result.push((Mark::Numbered(*c as u8 - b'0'), v));
            }
        }

        for (m, v) in &self.special_marks {
            result.push((*m, v));
        }

        result
    }

    fn adjust(&mut self, line: LineNr, _col: usize, lines_added: i64, _bytes_added: i64) {
        // Adjust local marks
        for value in self.local_marks.values_mut() {
            if value.position.line.0 >= line.0 {
                let new_line = (value.position.line.0 as i64 + lines_added).max(1) as usize;
                value.position.line = LineNr(new_line);
            }
        }

        // Adjust special marks
        for value in self.special_marks.values_mut() {
            if value.position.line.0 >= line.0 {
                let new_line = (value.position.line.0 as i64 + lines_added).max(1) as usize;
                value.position.line = LineNr(new_line);
            }
        }
    }

    fn jump_list(&self) -> &dyn JumpList {
        &self.jump_list
    }

    fn jump_list_mut(&mut self) -> &mut dyn JumpList {
        &mut self.jump_list
    }

    fn change_list(&self) -> &dyn ChangeList {
        &self.change_list
    }

    fn change_list_mut(&mut self) -> &mut dyn ChangeList {
        &mut self.change_list
    }

    fn record_jump(&mut self, from: CursorPosition) {
        let entry = JumpEntry {
            buffer: self.current_buffer,
            position: from,
            file: None,
        };
        self.jump_list.push(entry);
    }

    fn record_change(&mut self, at: CursorPosition) {
        let entry = ChangeEntry {
            position: at,
            col: at.col,
        };
        self.change_list.push(entry);
    }

    fn set_visual_marks(&mut self, start: CursorPosition, end: CursorPosition) {
        self.special_marks
            .insert(Mark::VisualStart, MarkValue::new(start));
        self.special_marks
            .insert(Mark::VisualEnd, MarkValue::new(end));
    }

    fn set_change_marks(&mut self, start: CursorPosition, end: CursorPosition) {
        self.special_marks
            .insert(Mark::ChangeStart, MarkValue::new(start));
        self.special_marks
            .insert(Mark::ChangeEnd, MarkValue::new(end));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_marks() {
        let mut mgr = TuiMarkManager::new();
        let pos = CursorPosition::new(LineNr(5), 10);
        let value = MarkValue::new(pos);

        mgr.set(Mark::Local('a'), value.clone()).unwrap();
        assert_eq!(mgr.get(Mark::Local('a')), Some(&value));
    }

    #[test]
    fn test_global_marks() {
        let mut mgr = TuiMarkManager::new();
        let pos = CursorPosition::new(LineNr(10), 5);
        let value = MarkValue::new(pos);

        mgr.set(Mark::Global('A'), value.clone()).unwrap();
        assert_eq!(mgr.get(Mark::Global('A')), Some(&value));
    }

    #[test]
    fn test_marks_adjust_on_insert() {
        let mut mgr = TuiMarkManager::new();
        let pos = CursorPosition::new(LineNr(10), 0);
        let value = MarkValue::new(pos);

        mgr.set(Mark::Local('a'), value).unwrap();

        // Insert 5 lines at line 5
        mgr.adjust(LineNr(5), 0, 5, 0);

        // Mark should move from line 10 to line 15
        let mark = mgr.get(Mark::Local('a')).unwrap();
        assert_eq!(mark.position.line, LineNr(15));
    }

    #[test]
    fn test_marks_adjust_on_delete() {
        let mut mgr = TuiMarkManager::new();
        let pos = CursorPosition::new(LineNr(10), 0);
        let value = MarkValue::new(pos);

        mgr.set(Mark::Local('a'), value).unwrap();

        // Delete 3 lines at line 5
        mgr.adjust(LineNr(5), 0, -3, 0);

        // Mark should move from line 10 to line 7
        let mark = mgr.get(Mark::Local('a')).unwrap();
        assert_eq!(mark.position.line, LineNr(7));
    }

    #[test]
    fn test_jump_list() {
        let mut mgr = TuiMarkManager::new();

        mgr.record_jump(CursorPosition::new(LineNr(1), 0));
        mgr.record_jump(CursorPosition::new(LineNr(10), 0));
        mgr.record_jump(CursorPosition::new(LineNr(20), 0));

        assert_eq!(mgr.jump_list().len(), 3);

        // Go older
        let entry = mgr.jump_list_mut().go_older().unwrap();
        assert_eq!(entry.position.line, LineNr(20));
    }

    #[test]
    fn test_change_list() {
        let mut mgr = TuiMarkManager::new();

        mgr.record_change(CursorPosition::new(LineNr(5), 0));
        mgr.record_change(CursorPosition::new(LineNr(10), 0));

        assert_eq!(mgr.change_list().len(), 2);
    }

    #[test]
    fn test_visual_marks() {
        let mut mgr = TuiMarkManager::new();

        let start = CursorPosition::new(LineNr(1), 5);
        let end = CursorPosition::new(LineNr(3), 10);

        mgr.set_visual_marks(start, end);

        assert_eq!(mgr.get(Mark::VisualStart).unwrap().position, start);
        assert_eq!(mgr.get(Mark::VisualEnd).unwrap().position, end);
    }
}
