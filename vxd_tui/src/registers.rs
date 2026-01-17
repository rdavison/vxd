//! Register implementation.
//!
//! This module provides a concrete implementation of Vim's register system.

use std::collections::HashMap;
use vxd::registers::{Register, RegisterBank, RegisterContent};
use vxd::types::{VimError, VimResult};

/// Concrete register bank implementation
#[derive(Debug, Clone, Default)]
pub struct TuiRegisterBank {
    /// Named registers (a-z)
    named: HashMap<char, RegisterContent>,
    /// Numbered registers (0-9)
    numbered: [Option<RegisterContent>; 10],
    /// Unnamed register ("")
    unnamed: Option<RegisterContent>,
    /// Small delete register (-)
    small_delete: Option<RegisterContent>,
    /// Last search pattern (/)
    last_search: Option<String>,
    /// Last command (:)
    last_command: Option<String>,
    /// Selection register (*)
    selection: Option<RegisterContent>,
    /// Clipboard register (+)
    clipboard: Option<RegisterContent>,
}

impl TuiRegisterBank {
    /// Create a new empty register bank
    pub fn new() -> Self {
        TuiRegisterBank::default()
    }
}

impl RegisterBank for TuiRegisterBank {
    fn get(&self, reg: Register) -> Option<&RegisterContent> {
        match reg {
            Register::Unnamed => self.unnamed.as_ref(),
            Register::Named(c) => self.named.get(&c.to_ascii_lowercase()),
            Register::Numbered(n) => self.numbered.get(n as usize).and_then(|r| r.as_ref()),
            Register::SmallDelete => self.small_delete.as_ref(),
            Register::Selection => self.selection.as_ref(),
            Register::Clipboard => self.clipboard.as_ref(),
            Register::BlackHole => None, // Black hole always returns nothing
            Register::LastSearch => {
                // Return as register content
                None // Would need to wrap string
            }
            Register::LastCommand => None,
            _ => None, // Read-only registers not stored
        }
    }

    fn set(&mut self, reg: Register, content: RegisterContent) -> VimResult<()> {
        // Check for read-only registers
        if reg.is_readonly() {
            return Err(VimError::Error(
                1,
                format!("Cannot write to read-only register '{}'", reg.to_char()),
            ));
        }

        match reg {
            Register::Unnamed => {
                self.unnamed = Some(content);
            }
            Register::Named(c) => {
                self.named.insert(c.to_ascii_lowercase(), content);
            }
            Register::Numbered(n) => {
                if (n as usize) < 10 {
                    self.numbered[n as usize] = Some(content);
                }
            }
            Register::SmallDelete => {
                self.small_delete = Some(content);
            }
            Register::Selection => {
                self.selection = Some(content);
            }
            Register::Clipboard => {
                self.clipboard = Some(content);
            }
            Register::BlackHole => {
                // Silently discard
            }
            _ => {
                return Err(VimError::Error(
                    1,
                    format!("Cannot write to register '{}'", reg.to_char()),
                ));
            }
        }

        Ok(())
    }

    fn append(&mut self, reg: Register, content: RegisterContent) -> VimResult<()> {
        match reg {
            Register::Named(c) => {
                let key = c.to_ascii_lowercase();
                if let Some(existing) = self.named.get_mut(&key) {
                    existing.append(&content);
                } else {
                    self.named.insert(key, content);
                }
                Ok(())
            }
            _ => {
                // For non-named registers, just set
                self.set(reg, content)
            }
        }
    }

    fn record_yank(&mut self, content: RegisterContent) -> VimResult<()> {
        // Yank goes to "" and "0
        self.unnamed = Some(content.clone());
        self.numbered[0] = Some(content);
        Ok(())
    }

    fn record_delete(&mut self, content: RegisterContent, is_small: bool) -> VimResult<()> {
        // Delete goes to ""
        self.unnamed = Some(content.clone());

        if is_small {
            // Small delete (less than one line) goes to "-"
            self.small_delete = Some(content);
        } else {
            // Large delete rotates through 1-9
            self.rotate_numbered();
            self.numbered[1] = Some(content);
        }

        Ok(())
    }

    fn rotate_numbered(&mut self) {
        // Shift 1->2, 2->3, ..., 8->9 (9 is lost)
        for i in (2..=9).rev() {
            self.numbered[i] = self.numbered[i - 1].take();
        }
    }

    fn last_search(&self) -> Option<&str> {
        self.last_search.as_deref()
    }

    fn set_last_search(&mut self, pattern: &str) {
        self.last_search = Some(pattern.to_string());
    }

    fn last_command(&self) -> Option<&str> {
        self.last_command.as_deref()
    }

    fn set_last_command(&mut self, cmd: &str) {
        self.last_command = Some(cmd.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unnamed_default() {
        let mut bank = TuiRegisterBank::new();
        let content = RegisterContent::characterwise("test");
        bank.set_unnamed(content.clone()).unwrap();
        assert_eq!(bank.unnamed(), Some(&content));
    }

    #[test]
    fn test_named_registers() {
        let mut bank = TuiRegisterBank::new();
        let content = RegisterContent::characterwise("hello");
        bank.set(Register::Named('a'), content.clone()).unwrap();
        assert_eq!(bank.get(Register::Named('a')), Some(&content));
    }

    #[test]
    fn test_uppercase_appends() {
        let mut bank = TuiRegisterBank::new();
        let content1 = RegisterContent::characterwise("hello");
        let content2 = RegisterContent::characterwise(" world");

        bank.set(Register::Named('a'), content1).unwrap();
        bank.append(Register::Named('a'), content2).unwrap();

        let result = bank.get(Register::Named('a')).unwrap();
        assert!(result.as_string().contains("hello"));
        assert!(result.as_string().contains("world"));
    }

    #[test]
    fn test_black_hole() {
        let mut bank = TuiRegisterBank::new();
        let content = RegisterContent::characterwise("discarded");
        bank.set(Register::BlackHole, content).unwrap();
        assert!(bank.get(Register::BlackHole).is_none());
    }

    #[test]
    fn test_readonly_rejects_write() {
        let mut bank = TuiRegisterBank::new();
        let content = RegisterContent::characterwise("test");
        let result = bank.set(Register::LastInserted, content);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_0_yank() {
        let mut bank = TuiRegisterBank::new();
        let content = RegisterContent::characterwise("yanked");
        bank.record_yank(content.clone()).unwrap();
        assert_eq!(bank.get(Register::Numbered(0)), Some(&content));
    }

    #[test]
    fn test_numbered_delete_history() {
        let mut bank = TuiRegisterBank::new();
        let content1 = RegisterContent::linewise(vec!["delete1".into()]);
        let content2 = RegisterContent::linewise(vec!["delete2".into()]);

        bank.record_delete(content1.clone(), false).unwrap();
        assert_eq!(bank.get(Register::Numbered(1)), Some(&content1));

        bank.record_delete(content2.clone(), false).unwrap();
        assert_eq!(bank.get(Register::Numbered(1)), Some(&content2));
        assert_eq!(bank.get(Register::Numbered(2)), Some(&content1));
    }

    #[test]
    fn test_small_delete_register() {
        let mut bank = TuiRegisterBank::new();
        let content = RegisterContent::characterwise("small");
        bank.record_delete(content.clone(), true).unwrap();
        assert_eq!(bank.get(Register::SmallDelete), Some(&content));
    }
}
