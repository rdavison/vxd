//! Editor - the main entry point combining all components.
//!
//! This module provides the `Editor` struct which combines buffer, cursor,
//! mode, register, and mark management into a cohesive editor.

use crate::buffer::TuiBufferManager;
use crate::cursor::TuiCursor;
use crate::marks::TuiMarkManager;
use crate::modes::TuiModeManager;
use crate::registers::TuiRegisterBank;

use vxd::abbreviations::{AbbreviationManager, SimpleAbbreviationManager};
use vxd::buffer::{Buffer, BufferManager};
use vxd::cursor::{Cursor, CursorContext, CursorPosition, VirtualEdit};
use vxd::mappings::{MappingManager, SimpleMappingManager};
use vxd::marks::MarkManager;
use vxd::modes::{Mode, ModeManager, VisualMode};
use vxd::motions::CharFindMotion;
use vxd::registers::{Register, RegisterBank, RegisterContent, RegisterType};
use vxd::types::{LineNr, VimError, VimResult};

/// The main editor struct combining all components
#[derive(Debug)]
pub struct Editor {
    /// Buffer manager
    pub buffers: TuiBufferManager,
    /// Cursor for current window
    pub cursor: TuiCursor,
    /// Mode manager
    pub modes: TuiModeManager,
    /// Register bank
    pub registers: TuiRegisterBank,
    /// Mark manager
    pub marks: TuiMarkManager,
    /// Abbreviation manager
    pub abbreviations: SimpleAbbreviationManager,
    /// Mapping manager
    pub mappings: SimpleMappingManager,
    /// Visual selection anchor
    pub visual_anchor: Option<CursorPosition>,
    last_char_find: Option<CharFindMotion>,
    current_insert: Option<String>,
    block_op_context: Option<BlockOpContext>,
}

#[derive(Debug, Clone)]
struct BlockOpContext {
    start_line: LineNr,
    end_line: LineNr,
    col: usize,
    // op_type is implicitly Insert for now, as we only use this for repeating insert
}

impl Editor {
    /// Create a new editor instance
    pub fn new() -> Self {
        let mut editor = Editor {
            buffers: TuiBufferManager::new(),
            cursor: TuiCursor::new(),
            modes: TuiModeManager::new(),
            registers: TuiRegisterBank::new(),
            marks: TuiMarkManager::new(),
            abbreviations: SimpleAbbreviationManager::new(),
            mappings: SimpleMappingManager::new(),
            visual_anchor: None,
            last_char_find: None,
            current_insert: None,
            block_op_context: None,
        };

        // Sync cursor with initial buffer
        editor.sync_cursor_with_buffer();

        editor
    }

    /// Sync cursor line lengths with current buffer
    pub fn sync_cursor_with_buffer(&mut self) {
        let lines = self
            .buffers
            .current()
            .get_lines(0, -1, false)
            .unwrap_or_default();
        self.cursor.update_line_lengths(&lines);
        let pos = self.cursor.position();
        let ctx = self.cursor_context();
        let _ = self.cursor.set_position(pos, &ctx);
    }

    /// Get cursor context based on current mode
    pub fn cursor_context(&self) -> CursorContext {
        CursorContext {
            allow_past_eol: self.modes.mode().allows_cursor_past_eol(),
            virtualedit: VirtualEdit::None, // TODO: Get from options
            visual_selection: self.modes.mode().is_visual(),
        }
    }

    /// Get current line content
    pub fn current_line(&self) -> String {
        self.buffers
            .current()
            .get_line(self.cursor.line().0 as i64 - 1)
            .unwrap_or_default()
    }

    /// Get the current mode
    pub fn mode(&self) -> Mode {
        self.modes.mode()
    }

    /// Enter insert mode
    pub fn enter_insert(&mut self) -> VimResult<()> {
        self.modes
            .enter_insert()
            .map(|_| ())
            .map_err(|err| VimError::NotAllowedInMode(err.reason))?;
        self.current_insert = Some(String::new());
        Ok(())
    }

    /// Enter replace mode
    pub fn enter_replace(&mut self) -> VimResult<()> {
        self.modes
            .enter_replace()
            .map(|_| ())
            .map_err(|err| VimError::NotAllowedInMode(err.reason))?;
        self.current_insert = Some(String::new());
        Ok(())
    }

    /// Enter visual mode
    pub fn enter_visual(&mut self) -> VimResult<()> {
        self.modes
            .enter_visual()
            .map(|_| ())
            .map_err(|err| VimError::NotAllowedInMode(err.reason))?;
        self.visual_anchor = Some(self.cursor.position());
        Ok(())
    }

    /// Enter visual line mode
    pub fn enter_visual_line(&mut self) -> VimResult<()> {
        self.modes
            .enter_visual_line()
            .map(|_| ())
            .map_err(|err| VimError::NotAllowedInMode(err.reason))?;
        self.visual_anchor = Some(self.cursor.position());
        Ok(())
    }

    /// Enter visual block mode
    pub fn enter_visual_block(&mut self) -> VimResult<()> {
        self.modes
            .enter_visual_block()
            .map(|_| ())
            .map_err(|err| VimError::NotAllowedInMode(err.reason))?;
        self.visual_anchor = Some(self.cursor.position());
        Ok(())
    }

    /// Enter normal mode (escape)
    pub fn escape(&mut self) -> VimResult<()> {
        self.modes
            .escape_to_normal()
            .map(|_| ())
            .map_err(|err| VimError::NotAllowedInMode(err.reason))?;
        if let Some(inserted) = self.current_insert.take() {
            if !inserted.is_empty() {
                self.registers
                    .set_last_inserted(vxd::registers::RegisterContent::characterwise(inserted.clone()));
            }

            // Apply block operation if active
            if let Some(ctx) = self.block_op_context.take() {
                // Visual block insert typically only supports single-line text insertion being replicated.
                if !inserted.contains('\n') && !inserted.is_empty() {
                    let current_line = self.cursor.line();
                    for i in ctx.start_line.0..=ctx.end_line.0 {
                        let line_nr = LineNr(i);
                        if line_nr == current_line {
                            continue; // Already done
                        }
                        
                        // Insert text at ctx.col
                        if let Ok(mut line) = self.buffers.current().get_line((i as i64) - 1) {
                             // Pad line if needed? Vim does. For now, simple check.
                             if ctx.col <= line.len() {
                                 line.insert_str(ctx.col, &inserted);
                                 self.buffers.current_mut().set_lines(
                                     (i as i64) - 1,
                                     (i as i64),
                                     false,
                                     vec![line]
                                 ).ok(); // Ignore errors
                             }
                        }
                    }
                }
            }
        }
        // In normal mode, cursor can't be past EOL
        let ctx = self.cursor_context();
        self.cursor.check_cursor(&ctx);
        self.visual_anchor = None;
        Ok(())
    }

    /// Move cursor down
    pub fn cursor_down(&mut self, count: usize) -> VimResult<()> {
        let ctx = self.cursor_context();
        let new_line =
            LineNr((self.cursor.line().0 + count).min(self.buffers.current().line_count()));
        let want_col = self.cursor.curswant().value();
        self.cursor
            .set_position(CursorPosition::new(new_line, want_col), &ctx)?;
        self.sync_cursor_with_buffer();
        Ok(())
    }

    /// Move cursor up
    pub fn cursor_up(&mut self, count: usize) -> VimResult<()> {
        let ctx = self.cursor_context();
        let new_line = LineNr(self.cursor.line().0.saturating_sub(count).max(1));
        let want_col = self.cursor.curswant().value();
        self.cursor
            .set_position(CursorPosition::new(new_line, want_col), &ctx)?;
        self.sync_cursor_with_buffer();
        Ok(())
    }

    /// Move cursor left
    pub fn cursor_left(&mut self, count: usize) -> VimResult<()> {
        let ctx = self.cursor_context();
        let new_col = self.cursor.col().saturating_sub(count);
        self.cursor.set_col(new_col, &ctx)?;
        self.cursor.update_curswant();
        Ok(())
    }

    /// Move cursor right
    pub fn cursor_right(&mut self, count: usize) -> VimResult<()> {
        let ctx = self.cursor_context();
        let new_col = self.cursor.col() + count;
        self.cursor.set_col(new_col, &ctx)?;
        self.cursor.update_curswant();
        Ok(())
    }

    /// Insert text at cursor position (checks for abbreviations)
    pub fn insert_char(&mut self, c: char) -> VimResult<()> {
        if !self.modes.mode().allows_insertion() {
            return Ok(()); // Ignore in non-insert modes
        }

        // Check for abbreviations (only on non-keyword chars)
        // Simplified keyword check: alphanumeric or underscore
        if !c.is_alphanumeric() && c != '_' {
            let current_line = self.current_line();
            let col = self.cursor.col();
            if let Some((word, start_col)) = self.get_word_before_cursor(&current_line, col) {
                // Check if an abbreviation exists and clone the RHS to avoid borrow issues
                let expansion = self.abbreviations.check(self.modes.mode(), &word).map(|a| a.rhs.clone());
                
                if let Some(rhs) = expansion {
                    // Remove the abbreviation
                    // Move cursor to start of word
                    let ctx = self.cursor_context();
                    self.cursor.set_col(start_col, &ctx)?;
                    
                    // Delete the word
                    for _ in 0..word.len() {
                        self.delete_char()?;
                    }
                    
                    // Insert the expansion (raw, to avoid recursive expansion loops immediately)
                    // Note: Vim allows remapping in expansion, but here we just insert text.
                    // If 'noremap' is false, we might need to feed keys? 
                    // For now, let's just insert text raw.
                    self.insert_text_raw(&rhs)?;
                    
                    // Insert the triggering character
                    return self.insert_char_raw(c);
                }
            }
        }

        self.insert_char_raw(c)
    }

    /// Insert text at cursor position without checking for abbreviations
    pub fn insert_char_raw(&mut self, c: char) -> VimResult<()> {
        if !self.modes.mode().allows_insertion() {
            return Ok(()); // Ignore in non-insert modes
        }

        let line_idx = self.cursor.line().0 as i64 - 1;
        let col = self.cursor.col();

        // Get current line
        let current_line = self.current_line();

        let new_line = if self.modes.mode() == Mode::Replace && col < current_line.len() {
            let char_end = col
                + current_line[col..]
                    .chars()
                    .next()
                    .map(|ch| ch.len_utf8())
                    .unwrap_or(1);
            format!("{}{}{}", &current_line[..col], c, &current_line[char_end..])
        } else {
            let (before, after) = if col <= current_line.len() {
                (&current_line[..col], &current_line[col..])
            } else {
                (current_line.as_str(), "")
            };
            format!("{}{}{}", before, c, after)
        };
        self.buffers
            .current_mut()
            .set_lines(line_idx, line_idx + 1, false, vec![new_line])?;

        // Move cursor right
        self.sync_cursor_with_buffer();
        let ctx = self.cursor_context();
        self.cursor.set_col(col + c.len_utf8(), &ctx)?;

        // Record change
        self.marks.record_change(self.cursor.position());
        if let Some(inserted) = self.current_insert.as_mut() {
            inserted.push(c);
        }

        Ok(())
    }

    /// Get word before cursor for abbreviation checking
    fn get_word_before_cursor(&self, line: &str, col: usize) -> Option<(String, usize)> {
        if col == 0 || col > line.len() {
            return None;
        }

        let text_before = &line[..col];
        // Find end of last non-keyword char
        // Keyword chars: alphanumeric + '_'
        let last_non_keyword = text_before.rfind(|c: char| !c.is_alphanumeric() && c != '_');
        
        let start = match last_non_keyword {
            Some(idx) => idx + 1, // Start after the non-keyword
            None => 0, // Start of line
        };

        if start >= col {
            return None; // No word
        }

        let word = text_before[start..col].to_string();
        if word.is_empty() {
            None
        } else {
            Some((word, start))
        }
    }

    /// Insert a character from a nearby line (Ctrl-Y/Ctrl-E behavior).
    pub fn insert_from_adjacent_line(&mut self, line_offset: i64) -> VimResult<()> {
        if !self.modes.mode().allows_insertion() {
            return Ok(());
        }

        let line_count = self.buffers.current().line_count() as i64;
        let current_idx = self.cursor.line().0 as i64 - 1;
        let target_idx = current_idx + line_offset;
        if target_idx < 0 || target_idx >= line_count {
            return Ok(());
        }

        let source_line = self
            .buffers
            .current()
            .get_line(target_idx)
            .unwrap_or_default();
        let col = self.cursor.col();
        if col >= source_line.len() {
            return Ok(());
        }

        if let Some(ch) = source_line[col..].chars().next() {
            self.insert_char_raw(ch)?;
        }

        Ok(())
    }

    /// Put the contents of a register into the current buffer.
    pub fn put_register(&mut self, reg: Register, after: bool) -> VimResult<()> {
        let Some(content) = self.registers.get(reg).cloned() else {
            return Ok(());
        };

        match content.reg_type {
            RegisterType::Linewise => {
                let line = self.cursor.line().0 as i64;
                let insert_at = if after { line } else { line.saturating_sub(1) };
                self.buffers
                    .current_mut()
                    .set_lines(insert_at, insert_at, false, content.text)?;
                self.sync_cursor_with_buffer();
                let new_line = if after { line + 1 } else { line.max(1) };
                let ctx = self.cursor_context();
                self.cursor.set_line(LineNr(new_line as usize), &ctx)?;
                self.cursor.set_col(0, &ctx)?;
            }
            RegisterType::Characterwise => {
                let line_idx = self.cursor.line().0 as i64 - 1;
                let line = self.current_line();
                let text = RegisterContent {
                    text: content.text,
                    reg_type: RegisterType::Characterwise,
                }
                .as_string();
                let mut insert_col = self.cursor.col();
                if after {
                    insert_col = insert_col.saturating_add(1);
                }
                insert_col = insert_col.min(line.len());
                let mut new_line = line.clone();
                new_line.insert_str(insert_col, &text);
                self.buffers
                    .current_mut()
                    .set_lines(line_idx, line_idx + 1, false, vec![new_line])?;
                self.sync_cursor_with_buffer();
                let ctx = self.cursor_context();
                self.cursor
                    .set_col(insert_col + text.len(), &ctx)?;
            }
            RegisterType::Blockwise { .. } => {
                return Err(VimError::Error(
                    1,
                    "Blockwise put not implemented".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Delete character at cursor position (like 'x' command)
    pub fn delete_char(&mut self) -> VimResult<()> {
        let line_idx = self.cursor.line().0 as i64 - 1;
        let col = self.cursor.col();
        let current_line = self.current_line();

        if col >= current_line.len() {
            return Ok(()); // Nothing to delete
        }

        // Find character boundary
        let char_end = col
            + current_line[col..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);

        // Build new line
        let new_line = format!("{}{}", &current_line[..col], &current_line[char_end..]);
        self.buffers
            .current_mut()
            .set_lines(line_idx, line_idx + 1, false, vec![new_line])?;

        self.sync_cursor_with_buffer();
        let ctx = self.cursor_context();
        self.cursor.check_cursor(&ctx);

        // Record change
        self.marks.record_change(self.cursor.position());

        Ok(())
    }

    /// Insert a new line (like pressing Enter in insert mode)
    pub fn insert_newline(&mut self) -> VimResult<()> {
        if !self.modes.mode().allows_insertion() {
            return Ok(());
        }

        let line_idx = self.cursor.line().0 as i64 - 1;
        let col = self.cursor.col();
        let current_line = self.current_line();

        // Split line at cursor
        let (before, after) = if col <= current_line.len() {
            (&current_line[..col], &current_line[col..])
        } else {
            (current_line.as_str(), "")
        };

        self.buffers.current_mut().set_lines(
            line_idx,
            line_idx + 1,
            false,
            vec![before.to_string(), after.to_string()],
        )?;

        // Move cursor to beginning of new line
        self.sync_cursor_with_buffer();
        let ctx = self.cursor_context();
        self.cursor.set_position(
            vxd::cursor::CursorPosition::new(LineNr(self.cursor.line().0 + 1), 0),
            &ctx,
        )?;
        if let Some(inserted) = self.current_insert.as_mut() {
            inserted.push('\n');
        }

        Ok(())
    }

    pub fn insert_text(&mut self, text: &str) -> VimResult<()> {
        for ch in text.chars() {
            if ch == '\n' {
                self.insert_newline()?;
            } else {
                self.insert_char(ch)?;
            }
        }
        Ok(())
    }

    pub fn insert_text_raw(&mut self, text: &str) -> VimResult<()> {
        for ch in text.chars() {
            if ch == '\n' {
                self.insert_newline()?;
            } else {
                self.insert_char_raw(ch)?;
            }
        }
        Ok(())
    }

    pub fn find_char(&mut self, motion: CharFindMotion) -> VimResult<bool> {
        let actual_motion = match motion {
            CharFindMotion::RepeatForward => match self.last_char_find {
                Some(last) => last,
                None => return Ok(false),
            },
            CharFindMotion::RepeatBackward => match self.last_char_find {
                Some(last) => invert_char_find(last),
                None => return Ok(false),
            },
            other => other,
        };

        let line = self.current_line();
        let col = self.cursor.col();
        let target_col = match char_find_target(&line, col, actual_motion) {
            Some(idx) => idx,
            None => return Ok(false),
        };

        let ctx = self.cursor_context();
        self.cursor.set_col(target_col, &ctx)?;
        self.cursor.update_curswant();
        self.last_char_find = Some(actual_motion);
        Ok(true)
    }

    pub fn match_bracket(&mut self) -> VimResult<bool> {
        let line = self.current_line();
        let col = self.cursor.col();
        let ch = match char_at(&line, col) {
            Some(ch) => ch,
            None => return Ok(false),
        };
        let (open, close, forward) = match ch {
            '(' => ('(', ')', true),
            '[' => ('[', ']', true),
            '{' => ('{', '}', true),
            ')' => ('(', ')', false),
            ']' => ('[', ']', false),
            '}' => ('{', '}', false),
            _ => return Ok(false),
        };

        let target_col = if forward {
            find_matching_forward(&line, col, open, close)
        } else {
            find_matching_backward(&line, col, open, close)
        };

        if let Some(target) = target_col {
            let ctx = self.cursor_context();
            self.cursor.set_col(target, &ctx)?;
            self.cursor.update_curswant();
            return Ok(true);
        }

        Ok(false)
    }

    // ========================================================================
    // Visual Mode Operations
    // ========================================================================

    fn delete_block_selection(&mut self) -> VimResult<(LineNr, LineNr, usize)> {
        let mode = self.modes.mode();
        let start = self.visual_anchor.ok_or(VimError::Error(1, "No selection".to_string()))?;
        let end = self.cursor.position();

        match mode {
            Mode::Visual(VisualMode::Block) => {
                let start_line = start.line.min(end.line);
                let end_line = start.line.max(end.line);
                let start_col = start.col.min(end.col);
                let end_col = start.col.max(end.col);

                let mut replacement = Vec::new();
                let buffer = self.buffers.current();
                
                // Get lines and modify them
                for i in start_line.0..=end_line.0 {
                    if let Ok(line) = buffer.get_line((i as i64) - 1) {
                        // Simple byte-based slicing for now (TODO: unicode/tabs)
                        let line_len = line.len();
                        if start_col < line_len {
                            let del_end = (end_col + 1).min(line_len);
                            let new_line = format!("{}{}", &line[..start_col], &line[del_end..]);
                            replacement.push((i, new_line));
                        } else {
                            // Selection starts after EOL, nothing to delete
                            replacement.push((i, line));
                        }
                    }
                }

                // Apply changes
                for (line_nr, content) in replacement {
                    self.buffers.current_mut().set_lines(
                        (line_nr as i64) - 1, 
                        (line_nr as i64) - 1 + 1, // exclusive end
                        false, 
                        vec![content]
                    )?;
                }
                
                Ok((start_line, end_line, start_col))
            }
            _ => Err(VimError::Error(1, "Only block delete implemented so far".to_string())),
        }
    }

    /// Delete the current visual selection (x/d)
    pub fn visual_delete(&mut self) -> VimResult<()> {
        let (start_line, _, start_col) = self.delete_block_selection()?;
        self.escape()?; // Exit visual mode
        // Position cursor at start
        let ctx = self.cursor_context();
        self.cursor.set_position(CursorPosition::new(start_line, start_col), &ctx)?;
        self.sync_cursor_with_buffer();
        Ok(())
    }

    /// Change the current visual selection (c)
    pub fn visual_change(&mut self) -> VimResult<()> {
        let (start_line, end_line, start_col) = self.delete_block_selection()?;
        self.escape()?; // To clear visual mode
        // Enter insert mode
        self.enter_insert()?;
        // Position cursor
        let ctx = self.cursor_context();
        self.cursor.set_position(CursorPosition::new(start_line, start_col), &ctx)?;
        self.sync_cursor_with_buffer();
        
        // Set context for repeat
        self.block_op_context = Some(BlockOpContext {
            start_line,
            end_line,
            col: start_col,
        });
        Ok(())
    }

    /// Insert at start of visual selection (I)
    pub fn visual_insert(&mut self) -> VimResult<()> {
        let mode = self.modes.mode();
        let start = self.visual_anchor.ok_or(VimError::Error(1, "No selection".to_string()))?;
        let end = self.cursor.position();

        match mode {
            Mode::Visual(VisualMode::Block) => {
                let start_line = start.line.min(end.line);
                let end_line = start.line.max(end.line);
                let start_col = start.col.min(end.col);
                
                self.escape()?; // Clear visual
                self.enter_insert()?;
                let ctx = self.cursor_context();
                self.cursor.set_position(CursorPosition::new(start_line, start_col), &ctx)?;
                self.sync_cursor_with_buffer();

                self.block_op_context = Some(BlockOpContext {
                    start_line,
                    end_line,
                    col: start_col,
                });
                Ok(())
            }
            _ => Err(VimError::Error(1, "Only block insert implemented".to_string())),
        }
    }

    /// Append at end of visual selection (A)
    pub fn visual_append(&mut self) -> VimResult<()> {
        let mode = self.modes.mode();
        let start = self.visual_anchor.ok_or(VimError::Error(1, "No selection".to_string()))?;
        let end = self.cursor.position();

        match mode {
            Mode::Visual(VisualMode::Block) => {
                let start_line = start.line.min(end.line);
                let end_line = start.line.max(end.line);
                let end_col = start.col.max(end.col);
                
                let insert_col = end_col + 1;

                self.escape()?; 
                self.enter_insert()?;
                let ctx = self.cursor_context();
                // Move cursor to start_line, insert_col.
                // Note: If line is shorter than insert_col, we might need padding?
                // For now, assume simple behavior.
                self.cursor.set_position(CursorPosition::new(start_line, insert_col), &ctx)?;
                self.sync_cursor_with_buffer();

                self.block_op_context = Some(BlockOpContext {
                    start_line,
                    end_line,
                    col: insert_col,
                });
                Ok(())
            }
            _ => Err(VimError::Error(1, "Only block append implemented".to_string())),
        }
    }

    /// Yank the current visual selection (y)
    pub fn visual_yank(&mut self) -> VimResult<()> {
        let mode = self.modes.mode();
        let start = self.visual_anchor.ok_or(VimError::Error(1, "No selection".to_string()))?;
        let end = self.cursor.position();

        match mode {
            Mode::Visual(VisualMode::Block) => {
                let start_line = start.line.min(end.line);
                let end_line = start.line.max(end.line);
                let start_col = start.col.min(end.col);
                let end_col = start.col.max(end.col);
                let width = end_col - start_col + 1;

                let mut text = Vec::new();
                let buffer = self.buffers.current();
                
                for i in start_line.0..=end_line.0 {
                    if let Ok(line) = buffer.get_line((i as i64) - 1) {
                        let line_len = line.len();
                        if start_col < line_len {
                            let end_slice = (end_col + 1).min(line_len);
                            text.push(line[start_col..end_slice].to_string());
                        } else {
                            text.push("".to_string());
                        }
                    }
                }

                let content = RegisterContent {
                    text,
                    reg_type: RegisterType::Blockwise { width },
                };
                self.registers.set(Register::Unnamed, content.clone())?;
                self.registers.set(Register::Named('0'), content)?;

                self.escape()?;
                // Cursor to start
                let ctx = self.cursor_context();
                self.cursor.set_position(CursorPosition::new(start_line, start_col), &ctx)?;
                Ok(())
            }
            Mode::Visual(VisualMode::Line) => {
                let start_line = start.line.min(end.line);
                let end_line = start.line.max(end.line);
                let buffer = self.buffers.current();
                
                let mut text = Vec::new();
                for i in start_line.0..=end_line.0 {
                    if let Ok(line) = buffer.get_line((i as i64) - 1) {
                         text.push(line);
                    }
                }
                
                let content = RegisterContent {
                    text,
                    reg_type: RegisterType::Linewise,
                };
                self.registers.set(Register::Unnamed, content.clone())?;
                self.registers.set(Register::Named('0'), content)?;
                
                self.escape()?;
                // Cursor to start line, col 0
                let ctx = self.cursor_context();
                self.cursor.set_position(CursorPosition::new(start_line, 0), &ctx)?;
                self.sync_cursor_with_buffer();
                Ok(())
            }
            Mode::Visual(VisualMode::Char) => {
                // Handle forward/backward selection
                let (start_pos, end_pos) = if start.line < end.line || (start.line == end.line && start.col <= end.col) {
                     (start, end)
                } else {
                     (end, start)
                };
                
                let buffer = self.buffers.current();
                let mut text = Vec::new();
                
                if start_pos.line == end_pos.line {
                     if let Ok(line) = buffer.get_line((start_pos.line.0 as i64) - 1) {
                         let len = line.len();
                         let s = start_pos.col.min(len);
                         let e = (end_pos.col + 1).min(len);
                         if s < e {
                             text.push(line[s..e].to_string());
                         } else {
                             text.push("".to_string());
                         }
                     }
                } else {
                    // Multi-line charwise
                    // First line
                    if let Ok(line) = buffer.get_line((start_pos.line.0 as i64) - 1) {
                        let len = line.len();
                        let s = start_pos.col.min(len);
                        text.push(line[s..].to_string());
                    }
                    
                    // Middle lines
                    for i in (start_pos.line.0 + 1)..end_pos.line.0 {
                         if let Ok(line) = buffer.get_line((i as i64) - 1) {
                             text.push(line);
                         }
                    }
                    
                    // Last line
                    if let Ok(line) = buffer.get_line((end_pos.line.0 as i64) - 1) {
                        let len = line.len();
                        let e = (end_pos.col + 1).min(len);
                        text.push(line[..e].to_string());
                    }
                }
                
                let content = RegisterContent {
                     text,
                     reg_type: RegisterType::Characterwise,
                };
                self.registers.set(Register::Unnamed, content.clone())?;
                self.registers.set(Register::Named('0'), content)?;
                
                self.escape()?;
                let ctx = self.cursor_context();
                self.cursor.set_position(start_pos, &ctx)?;
                self.sync_cursor_with_buffer();
                Ok(())
            }
            _ => Err(VimError::Error(1, "Only visual yank implemented".to_string())),
        }
    }
}

fn invert_char_find(motion: CharFindMotion) -> CharFindMotion {
    match motion {
        CharFindMotion::FindForward(c) => CharFindMotion::FindBackward(c),
        CharFindMotion::FindBackward(c) => CharFindMotion::FindForward(c),
        CharFindMotion::TillForward(c) => CharFindMotion::TillBackward(c),
        CharFindMotion::TillBackward(c) => CharFindMotion::TillForward(c),
        other => other,
    }
}

fn char_find_target(line: &str, col: usize, motion: CharFindMotion) -> Option<usize> {
    match motion {
        CharFindMotion::FindForward(target) => {
            let start = next_char_index(line, col)?;
            find_forward(line, start, target)
        }
        CharFindMotion::FindBackward(target) => find_backward(line, col, target),
        CharFindMotion::TillForward(target) => {
            let start = next_char_index(line, col)?;
            let idx = find_forward(line, start, target)?;
            previous_char_index(line, idx)
        }
        CharFindMotion::TillBackward(target) => {
            let idx = find_backward(line, col, target)?;
            let after = next_char_index(line, idx)?;
            if after >= line.len() {
                None
            } else {
                Some(after)
            }
        }
        CharFindMotion::RepeatForward | CharFindMotion::RepeatBackward => None,
    }
}

fn find_forward(line: &str, start: usize, target: char) -> Option<usize> {
    for (idx, ch) in line.char_indices() {
        if idx < start {
            continue;
        }
        if ch == target {
            return Some(idx);
        }
    }
    None
}

fn find_backward(line: &str, before: usize, target: char) -> Option<usize> {
    let mut found = None;
    for (idx, ch) in line.char_indices() {
        if idx >= before {
            break;
        }
        if ch == target {
            found = Some(idx);
        }
    }
    found
}

fn next_char_index(line: &str, idx: usize) -> Option<usize> {
    let ch = line.get(idx..)?.chars().next()?;
    Some(idx + ch.len_utf8())
}

fn previous_char_index(line: &str, idx: usize) -> Option<usize> {
    let mut prev = None;
    for (pos, _) in line.char_indices() {
        if pos >= idx {
            break;
        }
        prev = Some(pos);
    }
    prev
}

fn char_at(line: &str, col: usize) -> Option<char> {
    line.get(col..)?.chars().next()
}

fn find_matching_forward(line: &str, col: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in line.char_indices() {
        if idx < col {
            continue;
        }
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

fn find_matching_backward(line: &str, col: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in line.char_indices() {
        if idx > col {
            break;
        }
        if ch == close {
            depth += 1;
        } else if ch == open {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = Editor::new();
        assert_eq!(editor.mode(), Mode::Normal);
        assert_eq!(editor.buffers.list().len(), 1);
    }

    #[test]
    fn test_mode_switching() {
        let mut editor = Editor::new();

        assert_eq!(editor.mode(), Mode::Normal);

        editor.enter_insert().unwrap();
        assert_eq!(editor.mode(), Mode::Insert);

        editor.escape().unwrap();
        assert_eq!(editor.mode(), Mode::Normal);
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = Editor::new();

        // Add some content
        editor
            .buffers
            .current_mut()
            .set_lines(
                0,
                -1,
                false,
                vec!["line 1".into(), "line 2".into(), "line 3".into()],
            )
            .unwrap();
        editor.sync_cursor_with_buffer();

        // Move down
        editor.cursor_down(1).unwrap();
        assert_eq!(editor.cursor.line(), LineNr(2));

        // Move down again
        editor.cursor_down(1).unwrap();
        assert_eq!(editor.cursor.line(), LineNr(3));

        // Move up
        editor.cursor_up(1).unwrap();
        assert_eq!(editor.cursor.line(), LineNr(2));

        // Move right
        editor.cursor_right(3).unwrap();
        assert_eq!(editor.cursor.col(), 3);

        // Move left
        editor.cursor_left(1).unwrap();
        assert_eq!(editor.cursor.col(), 2);
    }

    #[test]
    fn test_insert_char() {
        let mut editor = Editor::new();

        // Enter insert mode
        editor.enter_insert().unwrap();

        // Insert characters
        editor.insert_char('h').unwrap();
        editor.insert_char('i').unwrap();

        // Check buffer content
        let lines = editor.buffers.current().get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["hi"]);
    }

    #[test]
    fn test_delete_char() {
        let mut editor = Editor::new();

        // Set content
        editor
            .buffers
            .current_mut()
            .set_lines(0, -1, false, vec!["hello".into()])
            .unwrap();
        editor.sync_cursor_with_buffer();

        // Delete first char
        editor.delete_char().unwrap();

        let lines = editor.buffers.current().get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["ello"]);
    }

    #[test]
    fn test_insert_newline() {
        let mut editor = Editor::new();

        // Set content and position cursor in middle
        editor
            .buffers
            .current_mut()
            .set_lines(0, -1, false, vec!["hello world".into()])
            .unwrap();
        editor.sync_cursor_with_buffer();

        // Enter insert mode and position cursor
        editor.enter_insert().unwrap();
        let ctx = editor.cursor_context();
        editor.cursor.set_col(5, &ctx).unwrap();

        // Insert newline
        editor.insert_newline().unwrap();

        let lines = editor.buffers.current().get_lines(0, -1, false).unwrap();
        assert_eq!(lines, vec!["hello", " world"]);
        assert_eq!(editor.cursor.line(), LineNr(2));
    }
}
