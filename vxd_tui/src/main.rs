//! VXD TUI - Interactive terminal-based Vim editor
//!
//! Run with: cargo run

use std::io::{self, stdout};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use vxd::buffer::{Buffer, BufferManager};
use vxd::cursor::Cursor;
use vxd::modes::Mode;
use vxd::registers::{Register, RegisterBank};
use vxd_tui::editor::Editor;

/// Application state
struct App {
    editor: Editor,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let mut editor = Editor::new();
        // Set some initial content
        editor
            .buffers
            .current_mut()
            .set_lines(
                0,
                -1,
                false,
                vec![
                    "Welcome to VXD - a Vim-compatible editor".to_string(),
                    "".to_string(),
                    "Commands:".to_string(),
                    "  i     - Enter insert mode".to_string(),
                    "  Esc   - Return to normal mode".to_string(),
                    "  h/j/k/l or arrows - Move cursor".to_string(),
                    "  x     - Delete character".to_string(),
                    "  q     - Quit (in normal mode)".to_string(),
                    "".to_string(),
                    "Start editing below:".to_string(),
                    "".to_string(),
                ],
            )
            .unwrap();
        editor.sync_cursor_with_buffer();

        App {
            editor,
            should_quit: false,
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match self.editor.mode() {
            Mode::Normal => self.handle_normal_key(code, modifiers),
            Mode::Insert => self.handle_insert_key(code, modifiers),
            _ => {}
        }
    }

    fn handle_normal_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) {
        match code {
            // Quit
            KeyCode::Char('q') => self.should_quit = true,

            // Enter insert mode
            KeyCode::Char('i') => {
                let _ = self.editor.enter_insert();
            }

            // Append after cursor
            KeyCode::Char('a') => {
                let _ = self.editor.cursor_right(1);
                let _ = self.editor.enter_insert();
            }

            // Insert at beginning of line
            KeyCode::Char('I') => {
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_col(0, &ctx);
                let _ = self.editor.enter_insert();
            }

            // Append at end of line
            KeyCode::Char('A') => {
                let line_len = self.editor.current_line().len();
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_col(line_len, &ctx);
                let _ = self.editor.enter_insert();
            }

            // Open line below
            KeyCode::Char('o') => {
                let line = self.editor.cursor.line().0 as i64;
                let _ = self.editor.buffers.current_mut().set_lines(
                    line,
                    line,
                    false,
                    vec!["".to_string()],
                );
                self.editor.sync_cursor_with_buffer();
                let _ = self.editor.cursor_down(1);
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_col(0, &ctx);
                let _ = self.editor.enter_insert();
            }

            // Open line above
            KeyCode::Char('O') => {
                let line = self.editor.cursor.line().0 as i64 - 1;
                let _ = self.editor.buffers.current_mut().set_lines(
                    line,
                    line,
                    false,
                    vec!["".to_string()],
                );
                self.editor.sync_cursor_with_buffer();
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_col(0, &ctx);
                let _ = self.editor.enter_insert();
            }

            // Movement
            KeyCode::Char('h') | KeyCode::Left => {
                let _ = self.editor.cursor_left(1);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let _ = self.editor.cursor_down(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let _ = self.editor.cursor_up(1);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                let _ = self.editor.cursor_right(1);
            }

            // Beginning/end of line
            KeyCode::Char('0') => {
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_col(0, &ctx);
            }
            KeyCode::Char('$') => {
                let line_len = self.editor.current_line().len();
                let ctx = self.editor.cursor_context();
                let col = if line_len > 0 { line_len - 1 } else { 0 };
                let _ = self.editor.cursor.set_col(col, &ctx);
            }

            // Word movement (simplified)
            KeyCode::Char('w') => {
                // Move to next word start
                let line = self.editor.current_line();
                let col = self.editor.cursor.col();
                if let Some(next_word) = find_next_word(&line, col) {
                    let ctx = self.editor.cursor_context();
                    let _ = self.editor.cursor.set_col(next_word, &ctx);
                }
            }
            KeyCode::Char('b') => {
                // Move to previous word start
                let line = self.editor.current_line();
                let col = self.editor.cursor.col();
                if let Some(prev_word) = find_prev_word(&line, col) {
                    let ctx = self.editor.cursor_context();
                    let _ = self.editor.cursor.set_col(prev_word, &ctx);
                }
            }

            // Delete character
            KeyCode::Char('x') => {
                let _ = self.editor.delete_char();
            }

            // Page up/down
            KeyCode::PageDown => {
                let _ = self.editor.cursor_down(20);
            }
            KeyCode::PageUp => {
                let _ = self.editor.cursor_up(20);
            }

            // Go to first/last line
            KeyCode::Char('g') => {
                // gg goes to first line (simplified, just 'g')
                let ctx = self.editor.cursor_context();
                let _ = self.editor.cursor.set_line(vxd::types::LineNr(1), &ctx);
            }
            KeyCode::Char('G') => {
                let line_count = self.editor.buffers.current().line_count();
                let ctx = self.editor.cursor_context();
                let _ = self
                    .editor
                    .cursor
                    .set_line(vxd::types::LineNr(line_count), &ctx);
            }

            _ => {}
        }
    }

    fn handle_insert_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            // Exit insert mode
            KeyCode::Esc => {
                let _ = self.editor.escape();
            }

            // Character input
            KeyCode::Char(c) => {
                // Handle Ctrl+C to exit
                if modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                    self.should_quit = true;
                    return;
                }
                if modifiers.contains(KeyModifiers::CONTROL) && c == 'a' {
                    if let Some(content) = self.editor.registers.get(Register::LastInserted) {
                        let _ = self.editor.insert_text(&content.as_string());
                    }
                    return;
                }
                let _ = self.editor.insert_char(c);
            }

            // Backspace
            KeyCode::Backspace => {
                let col = self.editor.cursor.col();
                if col > 0 {
                    let _ = self.editor.cursor_left(1);
                    let _ = self.editor.delete_char();
                } else {
                    // Join with previous line if at column 0
                    let line = self.editor.cursor.line().0;
                    if line > 1 {
                        let prev_line = self
                            .editor
                            .buffers
                            .current()
                            .get_line(line as i64 - 2)
                            .unwrap_or_default();
                        let curr_line = self.editor.current_line();
                        let prev_len = prev_line.len();
                        let joined = format!("{}{}", prev_line, curr_line);

                        let _ = self.editor.buffers.current_mut().set_lines(
                            line as i64 - 2,
                            line as i64,
                            false,
                            vec![joined],
                        );

                        self.editor.sync_cursor_with_buffer();
                        let ctx = self.editor.cursor_context();
                        let _ = self
                            .editor
                            .cursor
                            .set_line(vxd::types::LineNr(line - 1), &ctx);
                        let _ = self.editor.cursor.set_col(prev_len, &ctx);
                    }
                }
            }

            // Delete (forward)
            KeyCode::Delete => {
                let _ = self.editor.delete_char();
            }

            // Enter
            KeyCode::Enter => {
                let _ = self.editor.insert_newline();
            }

            // Tab
            KeyCode::Tab => {
                // Insert 4 spaces
                for _ in 0..4 {
                    let _ = self.editor.insert_char(' ');
                }
            }

            // Arrow keys
            KeyCode::Left => {
                let _ = self.editor.cursor_left(1);
            }
            KeyCode::Right => {
                let _ = self.editor.cursor_right(1);
            }
            KeyCode::Up => {
                let _ = self.editor.cursor_up(1);
            }
            KeyCode::Down => {
                let _ = self.editor.cursor_down(1);
            }

            _ => {}
        }
    }
}

/// Find the start of the next word
fn find_next_word(line: &str, col: usize) -> Option<usize> {
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();

    if col >= len {
        return None;
    }

    // Skip current word (non-space characters)
    let mut i = col;
    while i < len && !chars[i].is_whitespace() {
        i += 1;
    }

    // Skip whitespace
    while i < len && chars[i].is_whitespace() {
        i += 1;
    }

    if i < len {
        // Convert char index to byte offset
        Some(chars[..i].iter().map(|c| c.len_utf8()).sum())
    } else {
        None
    }
}

/// Find the start of the previous word
fn find_prev_word(line: &str, col: usize) -> Option<usize> {
    let chars: Vec<char> = line.chars().collect();

    if col == 0 {
        return None;
    }

    // Convert byte offset to char index
    let mut char_col = 0;
    let mut byte_count = 0;
    for (i, c) in chars.iter().enumerate() {
        if byte_count >= col {
            break;
        }
        char_col = i + 1;
        byte_count += c.len_utf8();
    }
    let mut i = char_col.saturating_sub(1);

    // Skip whitespace backwards
    while i > 0 && chars[i].is_whitespace() {
        i -= 1;
    }

    // Skip word backwards
    while i > 0 && !chars[i - 1].is_whitespace() {
        i -= 1;
    }

    // Convert back to byte offset
    Some(chars[..i].iter().map(|c| c.len_utf8()).sum())
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create app
    let mut app = App::new();

    // Main loop
    loop {
        // Render
        terminal.draw(|frame| render(frame, &app))?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code, key.modifiers);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Layout: main area + status line
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    // Render buffer content
    render_buffer(frame, chunks[0], app);

    // Render status line
    render_status(frame, chunks[1], app);

    // Set cursor position
    let cursor_line = app.editor.cursor.line().0 as u16;
    let cursor_col = app.editor.cursor.col() as u16;
    // Account for the border (1 row, 1 col offset)
    let cursor_x = chunks[0].x + 1 + cursor_col;
    let cursor_y = chunks[0].y + cursor_line; // line is 1-indexed, border adds 1

    // Ensure cursor is within bounds
    if cursor_x < chunks[0].x + chunks[0].width - 1 && cursor_y < chunks[0].y + chunks[0].height - 1
    {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render_buffer(frame: &mut Frame, area: Rect, app: &App) {
    let lines = app
        .editor
        .buffers
        .current()
        .get_lines(0, -1, false)
        .unwrap_or_default();

    // Calculate visible range based on cursor position
    let cursor_line = app.editor.cursor.line().0 as usize;
    let visible_height = (area.height as usize).saturating_sub(2); // Account for borders

    // Scroll so cursor is always visible
    let scroll_offset = if cursor_line > visible_height {
        cursor_line - visible_height
    } else {
        0
    };

    // Build display text with line numbers
    let display: Vec<Line> = lines
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, line)| {
            let line_num = format!("{:4} ", i + 1);
            Line::from(vec![
                Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                Span::raw(line.as_str()),
            ])
        })
        .collect();

    let buffer_widget =
        Paragraph::new(display).block(Block::default().borders(Borders::ALL).title(" VXD "));

    frame.render_widget(buffer_widget, area);
}

fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let mode = app.editor.mode();
    let mode_str = match mode {
        Mode::Normal => " NORMAL ",
        Mode::Insert => " INSERT ",
        Mode::Visual(_) => " VISUAL ",
        Mode::CommandLine(_) => " COMMAND ",
        _ => " OTHER ",
    };

    let mode_style = match mode {
        Mode::Normal => Style::default().bg(Color::Blue).fg(Color::White),
        Mode::Insert => Style::default().bg(Color::Green).fg(Color::Black),
        Mode::Visual(_) => Style::default().bg(Color::Magenta).fg(Color::White),
        _ => Style::default().bg(Color::Gray).fg(Color::Black),
    };

    let cursor_pos = format!(
        " {}:{} ",
        app.editor.cursor.line().0,
        app.editor.cursor.col() + 1
    );

    let status = Line::from(vec![
        Span::styled(mode_str, mode_style),
        Span::raw(" [No Name] "),
        Span::styled(
            if app.editor.buffers.current().is_modified() {
                "[+] "
            } else {
                ""
            },
            Style::default().fg(Color::Red),
        ),
        Span::raw(" ".repeat(area.width as usize - mode_str.len() - cursor_pos.len() - 15)),
        Span::styled(cursor_pos, Style::default().fg(Color::Cyan)),
    ]);

    let status_widget = Paragraph::new(status).style(Style::default().bg(Color::DarkGray));

    frame.render_widget(status_widget, area);
}
