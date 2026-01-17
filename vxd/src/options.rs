//! Option/setting system.
//!
//! Vim has three scopes for options: global, window-local, and buffer-local.
//! Options can be boolean, number, or string types.

use crate::types::*;

// ============================================================================
// Option Types
// ============================================================================

/// Scope of an option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionScope {
    /// Global option
    Global,
    /// Window-local option
    Window,
    /// Buffer-local option
    Buffer,
}

/// Type of an option value
#[derive(Debug, Clone, PartialEq)]
pub enum OptionValue {
    /// Boolean option (on/off)
    Boolean(bool),
    /// Number option
    Number(i64),
    /// String option
    String(String),
}

impl OptionValue {
    /// Get as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            OptionValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as number
    pub fn as_number(&self) -> Option<i64> {
        match self {
            OptionValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            OptionValue::String(s) => Some(s),
            _ => None,
        }
    }
}

/// Definition of an option
#[derive(Debug, Clone)]
pub struct OptionDef {
    /// Option name
    pub name: String,
    /// Short name/abbreviation
    pub short_name: Option<String>,
    /// Scope of the option
    pub scope: OptionScope,
    /// Default value
    pub default: OptionValue,
    /// Whether option is hidden
    pub hidden: bool,
    /// Description
    pub description: String,
}

// ============================================================================
// Common Options
// ============================================================================

/// Well-known option names
pub mod options {
    /// Tab stop width
    pub const TABSTOP: &str = "tabstop";
    /// Shift width for indentation
    pub const SHIFTWIDTH: &str = "shiftwidth";
    /// Expand tabs to spaces
    pub const EXPANDTAB: &str = "expandtab";
    /// Auto-indent new lines
    pub const AUTOINDENT: &str = "autoindent";
    /// Smart indent
    pub const SMARTINDENT: &str = "smartindent";
    /// Line numbers
    pub const NUMBER: &str = "number";
    /// Relative line numbers
    pub const RELATIVENUMBER: &str = "relativenumber";
    /// Wrap long lines
    pub const WRAP: &str = "wrap";
    /// Ignore case in search
    pub const IGNORECASE: &str = "ignorecase";
    /// Smart case in search
    pub const SMARTCASE: &str = "smartcase";
    /// Incremental search
    pub const INCSEARCH: &str = "incsearch";
    /// Highlight search
    pub const HLSEARCH: &str = "hlsearch";
    /// Show matching brackets
    pub const SHOWMATCH: &str = "showmatch";
    /// Cursor line highlighting
    pub const CURSORLINE: &str = "cursorline";
    /// Cursor column highlighting
    pub const CURSORCOLUMN: &str = "cursorcolumn";
    /// Scroll offset
    pub const SCROLLOFF: &str = "scrolloff";
    /// Side scroll offset
    pub const SIDESCROLLOFF: &str = "sidescrolloff";
    /// Hidden buffers
    pub const HIDDEN: &str = "hidden";
    /// File encoding
    pub const FILEENCODING: &str = "fileencoding";
    /// File format
    pub const FILEFORMAT: &str = "fileformat";
    /// Modified flag
    pub const MODIFIED: &str = "modified";
    /// Read-only flag
    pub const READONLY: &str = "readonly";
    /// Modifiable flag
    pub const MODIFIABLE: &str = "modifiable";
    /// Spell checking
    pub const SPELL: &str = "spell";
    /// List mode (show tabs, spaces)
    pub const LIST: &str = "list";
    /// Timeout for mappings
    pub const TIMEOUTLEN: &str = "timeoutlen";
    /// Update time
    pub const UPDATETIME: &str = "updatetime";
    /// Sign column
    pub const SIGNCOLUMN: &str = "signcolumn";
    /// Color column
    pub const COLORCOLUMN: &str = "colorcolumn";
    /// Text width
    pub const TEXTWIDTH: &str = "textwidth";
    /// Virtual edit mode
    pub const VIRTUALEDIT: &str = "virtualedit";
}

// ============================================================================
// Option Manager Trait
// ============================================================================

/// Trait for managing options
pub trait OptionManager {
    /// Get an option value
    fn get(&self, name: &str) -> Option<&OptionValue>;

    /// Set an option value
    fn set(&mut self, name: &str, value: OptionValue) -> VimResult<()>;

    /// Get a boolean option
    fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(|v| v.as_bool())
    }

    /// Get a number option
    fn get_number(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(|v| v.as_number())
    }

    /// Get a string option
    fn get_string(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(|v| v.as_str())
    }

    /// Set a boolean option
    fn set_bool(&mut self, name: &str, value: bool) -> VimResult<()> {
        self.set(name, OptionValue::Boolean(value))
    }

    /// Set a number option
    fn set_number(&mut self, name: &str, value: i64) -> VimResult<()> {
        self.set(name, OptionValue::Number(value))
    }

    /// Set a string option
    fn set_string(&mut self, name: &str, value: impl Into<String>) -> VimResult<()> {
        self.set(name, OptionValue::String(value.into()))
    }

    /// Toggle a boolean option
    fn toggle(&mut self, name: &str) -> VimResult<bool> {
        let current = self.get_bool(name).unwrap_or(false);
        self.set_bool(name, !current)?;
        Ok(!current)
    }

    /// Reset an option to its default value
    fn reset(&mut self, name: &str) -> VimResult<()>;

    /// Get all option definitions
    fn definitions(&self) -> Vec<&OptionDef>;

    /// Get the definition for a specific option
    fn definition(&self, name: &str) -> Option<&OptionDef>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_value_types() {
        let bool_val = OptionValue::Boolean(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.as_number(), None);

        let num_val = OptionValue::Number(42);
        assert_eq!(num_val.as_number(), Some(42));
        assert_eq!(num_val.as_bool(), None);

        let str_val = OptionValue::String("test".into());
        assert_eq!(str_val.as_str(), Some("test"));
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Option Behavioral Tests
        //!
        //! Tests derived from Neovim's test suite:
        //! - test/functional/options/ - option tests
        //!
        //! ## Key Quirks
        //!
        //! 1. **Local options**: Window/buffer options have local and global values.
        //!
        //! 2. **Option scoping**: `:set` affects all, `:setlocal` affects local only.
        //!
        //! 3. **Number options**: Can be incremented/decremented (`:set ts+=4`).
        //!
        //! 4. **String options**: Can be appended/prepended (`:set path+=dir`).
        //!
        //! 5. **Option callbacks**: Some options trigger side effects when changed.
    }
}
