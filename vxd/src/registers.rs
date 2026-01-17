//! Register system (yank/paste/named registers).
//!
//! Vim registers store text for later retrieval. There are many types
//! of registers with different behaviors.
//!
//! # Key Behavioral Contracts
//!
//! - Unnamed register ("") is the default for yank/delete/put
//! - Named registers (a-z) can be appended to with uppercase (A-Z)
//! - Numbered registers (0-9) hold recent deletes/yanks
//! - Some registers are read-only
//! - Register content can be linewise or characterwise

use crate::types::*;

// ============================================================================
// Register Names
// ============================================================================

/// A register identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    /// Unnamed register ("") - default for yank/delete/put
    Unnamed,
    /// Named register (a-z)
    Named(char),
    /// Numbered register (0-9)
    /// 0 = most recent yank
    /// 1-9 = recent deletes (1 = most recent)
    Numbered(u8),
    /// Small delete register (-) - deletes less than one line
    SmallDelete,
    /// Read-only: last inserted text (.)
    LastInserted,
    /// Read-only: current file name (%)
    CurrentFile,
    /// Read-only: alternate file name (#)
    AlternateFile,
    /// Read-only: last command (:)
    LastCommand,
    /// Read-only: last search pattern (/)
    LastSearch,
    /// Expression register (=) - evaluates expression on paste
    Expression,
    /// Selection register (*) - system selection (X11 PRIMARY)
    Selection,
    /// Clipboard register (+) - system clipboard
    Clipboard,
    /// Black hole register (_) - discards writes
    BlackHole,
    /// Last dropped text (~)
    LastDrop,
}

impl Register {
    /// Parse a register name from a character
    pub fn from_char(c: char) -> Result<Self, VimError> {
        match c {
            '"' => Ok(Register::Unnamed),
            'a'..='z' => Ok(Register::Named(c)),
            'A'..='Z' => Ok(Register::Named(c.to_ascii_lowercase())),
            '0'..='9' => Ok(Register::Numbered(c as u8 - b'0')),
            '-' => Ok(Register::SmallDelete),
            '.' => Ok(Register::LastInserted),
            '%' => Ok(Register::CurrentFile),
            '#' => Ok(Register::AlternateFile),
            ':' => Ok(Register::LastCommand),
            '/' => Ok(Register::LastSearch),
            '=' => Ok(Register::Expression),
            '*' => Ok(Register::Selection),
            '+' => Ok(Register::Clipboard),
            '_' => Ok(Register::BlackHole),
            '~' => Ok(Register::LastDrop),
            _ => Err(VimError::InvalidRegister(c)),
        }
    }

    /// Get the character representation of this register
    pub fn to_char(&self) -> char {
        match self {
            Register::Unnamed => '"',
            Register::Named(c) => *c,
            Register::Numbered(n) => (b'0' + n) as char,
            Register::SmallDelete => '-',
            Register::LastInserted => '.',
            Register::CurrentFile => '%',
            Register::AlternateFile => '#',
            Register::LastCommand => ':',
            Register::LastSearch => '/',
            Register::Expression => '=',
            Register::Selection => '*',
            Register::Clipboard => '+',
            Register::BlackHole => '_',
            Register::LastDrop => '~',
        }
    }

    /// Check if this register is read-only
    pub fn is_readonly(&self) -> bool {
        matches!(
            self,
            Register::LastInserted
                | Register::CurrentFile
                | Register::AlternateFile
                | Register::LastCommand
                | Register::LastSearch
        )
    }

    /// Check if this is an append operation (A-Z registers)
    pub fn is_append(c: char) -> bool {
        c.is_ascii_uppercase()
    }

    /// Check if this register interfaces with system clipboard
    pub fn is_clipboard(&self) -> bool {
        matches!(self, Register::Selection | Register::Clipboard)
    }
}

// ============================================================================
// Register Content
// ============================================================================

/// Type of content stored in a register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegisterType {
    /// Character-wise content (no trailing newline implied)
    #[default]
    Characterwise,
    /// Line-wise content (each line is complete)
    Linewise,
    /// Block-wise content (rectangular selection)
    Blockwise {
        /// Width of the block
        width: usize,
    },
}

/// Content stored in a register
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegisterContent {
    /// The text content (lines)
    pub text: Vec<String>,
    /// Type of the content
    pub reg_type: RegisterType,
}

impl RegisterContent {
    /// Create new characterwise content
    pub fn characterwise(text: impl Into<String>) -> Self {
        RegisterContent {
            text: vec![text.into()],
            reg_type: RegisterType::Characterwise,
        }
    }

    /// Create new linewise content
    pub fn linewise(lines: Vec<String>) -> Self {
        RegisterContent {
            text: lines,
            reg_type: RegisterType::Linewise,
        }
    }

    /// Create new blockwise content
    pub fn blockwise(lines: Vec<String>, width: usize) -> Self {
        RegisterContent {
            text: lines,
            reg_type: RegisterType::Blockwise { width },
        }
    }

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() || (self.text.len() == 1 && self.text[0].is_empty())
    }

    /// Get the content as a single string
    pub fn as_string(&self) -> String {
        match self.reg_type {
            RegisterType::Linewise => {
                let mut s = self.text.join("\n");
                s.push('\n');
                s
            }
            RegisterType::Characterwise | RegisterType::Blockwise { .. } => self.text.join("\n"),
        }
    }

    /// Append content to this register
    pub fn append(&mut self, other: &RegisterContent) {
        // When appending, linewise wins if either is linewise
        if other.reg_type == RegisterType::Linewise {
            self.reg_type = RegisterType::Linewise;
        }
        self.text.extend(other.text.iter().cloned());
    }
}

// ============================================================================
// Register Bank Trait
// ============================================================================

/// Manages all registers
pub trait RegisterBank {
    /// Get the content of a register
    fn get(&self, reg: Register) -> Option<&RegisterContent>;

    /// Set the content of a register
    ///
    /// Returns error if register is read-only.
    fn set(&mut self, reg: Register, content: RegisterContent) -> VimResult<()>;

    /// Append to a named register (A-Z behavior)
    fn append(&mut self, reg: Register, content: RegisterContent) -> VimResult<()>;

    /// Get the unnamed register
    fn unnamed(&self) -> Option<&RegisterContent> {
        self.get(Register::Unnamed)
    }

    /// Set the unnamed register
    fn set_unnamed(&mut self, content: RegisterContent) -> VimResult<()> {
        self.set(Register::Unnamed, content)
    }

    /// Record a yank to register 0 and unnamed
    fn record_yank(&mut self, content: RegisterContent) -> VimResult<()>;

    /// Record a delete to numbered registers (shifts 1-9)
    fn record_delete(&mut self, content: RegisterContent, is_small: bool) -> VimResult<()>;

    /// Rotate numbered registers (for delete history)
    fn rotate_numbered(&mut self);

    /// Clear a register
    fn clear(&mut self, reg: Register) -> VimResult<()> {
        self.set(reg, RegisterContent::default())
    }

    /// Get the last search pattern register
    fn last_search(&self) -> Option<&str>;

    /// Set the last search pattern
    fn set_last_search(&mut self, pattern: &str);

    /// Get the last command register
    fn last_command(&self) -> Option<&str>;

    /// Set the last command
    fn set_last_command(&mut self, cmd: &str);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_from_char() {
        assert_eq!(Register::from_char('"').unwrap(), Register::Unnamed);
        assert_eq!(Register::from_char('a').unwrap(), Register::Named('a'));
        assert_eq!(Register::from_char('A').unwrap(), Register::Named('a')); // Uppercase = append
        assert_eq!(Register::from_char('0').unwrap(), Register::Numbered(0));
        assert_eq!(Register::from_char('9').unwrap(), Register::Numbered(9));
        assert_eq!(Register::from_char('*').unwrap(), Register::Selection);
        assert_eq!(Register::from_char('+').unwrap(), Register::Clipboard);
        assert_eq!(Register::from_char('_').unwrap(), Register::BlackHole);
        assert!(Register::from_char('!').is_err());
    }

    #[test]
    fn test_register_readonly() {
        assert!(!Register::Unnamed.is_readonly());
        assert!(!Register::Named('a').is_readonly());
        assert!(Register::LastInserted.is_readonly());
        assert!(Register::CurrentFile.is_readonly());
        assert!(Register::LastCommand.is_readonly());
    }

    #[test]
    fn test_register_content_string() {
        let char_content = RegisterContent::characterwise("hello");
        assert_eq!(char_content.as_string(), "hello");

        let line_content = RegisterContent::linewise(vec!["line1".into(), "line2".into()]);
        assert_eq!(line_content.as_string(), "line1\nline2\n");
    }

    /// Behavioral tests for register implementations
    pub trait RegisterBehaviorTests: RegisterBank + Sized {
        // ====================================================================
        // Basic Register Tests
        // ====================================================================

        /// Test: Unnamed register is default target
        fn test_unnamed_default(&mut self) {
            let content = RegisterContent::characterwise("test");
            self.set_unnamed(content.clone()).unwrap();
            assert_eq!(self.unnamed(), Some(&content));
        }

        /// Test: Named registers store content
        fn test_named_registers(&mut self) {
            let content = RegisterContent::characterwise("hello");
            self.set(Register::Named('a'), content.clone()).unwrap();
            assert_eq!(self.get(Register::Named('a')), Some(&content));
        }

        /// Test: Uppercase appends to lowercase register
        fn test_uppercase_appends(&mut self) {
            let content1 = RegisterContent::characterwise("hello");
            let content2 = RegisterContent::characterwise(" world");

            self.set(Register::Named('a'), content1).unwrap();
            self.append(Register::Named('a'), content2).unwrap();

            let result = self.get(Register::Named('a')).unwrap();
            assert!(result.as_string().contains("hello"));
            assert!(result.as_string().contains("world"));
        }

        /// Test: Black hole register discards content
        fn test_black_hole(&mut self) {
            let content = RegisterContent::characterwise("discarded");
            self.set(Register::BlackHole, content).unwrap();
            // Black hole should return empty or None
            let result = self.get(Register::BlackHole);
            assert!(result.is_none() || result.unwrap().is_empty());
        }

        /// Test: Read-only registers reject writes
        fn test_readonly_rejects_write(&mut self) {
            let content = RegisterContent::characterwise("test");
            let result = self.set(Register::LastInserted, content);
            assert!(result.is_err());
        }

        // ====================================================================
        // Numbered Register Tests
        // ====================================================================

        /// Test: Register 0 holds most recent yank
        fn test_register_0_yank(&mut self) {
            let content = RegisterContent::characterwise("yanked");
            self.record_yank(content.clone()).unwrap();
            assert_eq!(self.get(Register::Numbered(0)), Some(&content));
        }

        /// Test: Registers 1-9 hold delete history
        fn test_numbered_delete_history(&mut self) {
            let content1 = RegisterContent::linewise(vec!["delete1".into()]);
            let content2 = RegisterContent::linewise(vec!["delete2".into()]);

            self.record_delete(content1.clone(), false).unwrap();
            assert_eq!(self.get(Register::Numbered(1)), Some(&content1));

            self.record_delete(content2.clone(), false).unwrap();
            assert_eq!(self.get(Register::Numbered(1)), Some(&content2));
            assert_eq!(self.get(Register::Numbered(2)), Some(&content1));
        }

        /// Test: Small delete register for less than one line
        fn test_small_delete_register(&mut self) {
            let content = RegisterContent::characterwise("small");
            self.record_delete(content.clone(), true).unwrap();
            assert_eq!(self.get(Register::SmallDelete), Some(&content));
        }
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Register Behavioral Tests
        //!
        //! Tests derived from Neovim's test suite:
        //! - test/functional/vimscript/functions_spec.lua (getreg, setreg)
        //! - test/functional/editor/put_spec.lua
        //!
        //! ## Key Quirks
        //!
        //! 1. **Delete rotation**: Deletes shift 1→2→3...→9, but yanks only go to "0".
        //!
        //! 2. **Small delete**: Deletes less than one line go to "-", not numbered.
        //!
        //! 3. **Append behavior**: "Ay appends to "a, preserving type if compatible.
        //!
        //! 4. **Type promotion**: When appending, linewise + charwise = linewise.
        //!
        //! 5. **Expression register**: "=expr<CR> evaluates expr and uses result.
        //!
        //! 6. **Clipboard registers**: "*" and "+" may be the same on some systems.
    }
}
