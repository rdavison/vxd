use std::fmt;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Parsed key representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    Escape,
    Enter,
    Backspace,
    Delete,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Ctrl(char),
    Alt(char),
    F(u8),
    Null, // For unknown keys
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Char(' ') => write!(f, " "), // Or <Space>? Vim allows both, but literals are simpler
            Key::Char('<') => write!(f, "<lt>"), // Special handling for <
            Key::Char(c) => write!(f, "{}", c),
            Key::Escape => write!(f, "<Esc>"),
            Key::Enter => write!(f, "<CR>"),
            Key::Backspace => write!(f, "<BS>"),
            Key::Delete => write!(f, "<Del>"),
            Key::Tab => write!(f, "<Tab>"),
            Key::Left => write!(f, "<Left>"),
            Key::Right => write!(f, "<Right>"),
            Key::Up => write!(f, "<Up>"),
            Key::Down => write!(f, "<Down>"),
            Key::Home => write!(f, "<Home>"),
            Key::End => write!(f, "<End>"),
            Key::PageUp => write!(f, "<PageUp>"),
            Key::PageDown => write!(f, "<PageDown>"),
            Key::Ctrl(c) => write!(f, "<C-{}>", c),
            Key::Alt(c) => write!(f, "<A-{}>", c),
            Key::F(n) => write!(f, "<F{}>", n),
            Key::Null => write!(f, ""),
        }
    }
}

impl From<KeyEvent> for Key {
    fn from(event: KeyEvent) -> Self {
        match event.code {
            KeyCode::Char(c) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    Key::Ctrl(c)
                } else if event.modifiers.contains(KeyModifiers::ALT) {
                    Key::Alt(c)
                } else {
                    Key::Char(c)
                }
            }
            KeyCode::Esc => Key::Escape,
            KeyCode::Enter => Key::Enter,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Delete => Key::Delete,
            KeyCode::Tab => Key::Tab,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::F(n) => Key::F(n),
            _ => Key::Null,
        }
    }
}

/// Parse a Vim-style key sequence into individual keys.
pub fn parse_keys(input: &str) -> Vec<Key> {
    let mut keys = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            // Parse special key notation
            let mut special = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '>' {
                    chars.next();
                    break;
                }
                special.push(chars.next().unwrap());
            }

            // Handle empty <> or incomplete
            if special.is_empty() {
                keys.push(Key::Char('<'));
                continue;
            }

            let key = match special.to_lowercase().as_str() {
                "esc" | "escape" => Key::Escape,
                "cr" | "enter" | "return" => Key::Enter,
                "bs" | "backspace" => Key::Backspace,
                "del" | "delete" => Key::Delete,
                "tab" => Key::Tab,
                "left" => Key::Left,
                "right" => Key::Right,
                "up" => Key::Up,
                "down" => Key::Down,
                "home" => Key::Home,
                "end" => Key::End,
                "pageup" => Key::PageUp,
                "pagedown" => Key::PageDown,
                "lt" => Key::Char('<'),
                "space" => Key::Char(' '),
                s if s.starts_with("c-") => {
                    let ch = s.chars().nth(2).unwrap_or('?');
                    Key::Ctrl(ch)
                }
                s if s.starts_with("a-") || s.starts_with("m-") => {
                    let ch = s.chars().nth(2).unwrap_or('?');
                    Key::Alt(ch)
                }
                _ => {
                    // Fallback for unknown special keys or literal <foo>
                    // Actually, if it's not recognized, it might be literal text wrapped in <>.
                    // But standard Vim notation expects <Esc>.
                    // For now, treat unknown as Null or handle gracefully?
                    // Let's treat as Null for now to avoid complexity.
                    Key::Null 
                }
            };
            if key != Key::Null {
                keys.push(key);
            }
        } else {
            keys.push(Key::Char(c));
        }
    }

    keys
}
