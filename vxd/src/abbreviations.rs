//! Abbreviation system.
//!
//! Abbreviations are similar to mappings but trigger only in Insert, Replace, and Command-line modes,
//! and only when a non-keyword character is typed after the abbreviation.
//!
//! # Key Behavioral Contracts
//!
//! - Abbreviations are mode-specific (iabbrev, cabbrev).
//! - Abbreviations are triggered by non-keyword characters.
//! - Abbreviations replace the typed word with the expansion.

use crate::modes::Mode;
use crate::types::*;
use std::collections::HashMap;

// ============================================================================
// Abbreviation Types
// ============================================================================

/// An abbreviation definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Abbreviation {
    /// Left-hand side (short form)
    pub lhs: String,
    /// Right-hand side (expansion)
    pub rhs: String,
    /// Whether the abbreviation is non-recursive
    pub noremap: bool,
    /// Whether the abbreviation is silent (no echo)
    pub silent: bool,
    /// Whether it's buffer-local
    pub buffer_local: bool,
}

// ============================================================================
// Abbreviation Manager Trait
// ============================================================================

/// Manages abbreviations
pub trait AbbreviationManager {
    /// Add an abbreviation
    fn add(&mut self, mode: Mode, lhs: &str, rhs: &str, noremap: bool, buffer_local: bool) -> VimResult<()>;

    /// Remove an abbreviation
    fn remove(&mut self, mode: Mode, lhs: &str) -> VimResult<()>;

    /// Get a specific abbreviation
    fn get(&self, mode: Mode, lhs: &str) -> Option<&Abbreviation>;

    /// Check if the word before cursor triggers an abbreviation.
    ///
    /// # Arguments
    /// * `mode` - Current mode (Insert or Cmdline)
    /// * `word` - The word immediately before the cursor (excluding trigger char)
    /// * `trigger_char` - The character just typed that might trigger expansion
    ///
    /// # Returns
    /// * `Some(Abbreviation)` if expansion should occur
    /// * `None` otherwise
    fn check(&self, mode: Mode, word: &str) -> Option<&Abbreviation>;
}

// ============================================================================
// Simple Implementation
// ============================================================================

/// A HashMap-based abbreviation manager
#[derive(Debug, Default, Clone)]
pub struct SimpleAbbreviationManager {
    /// Abbreviations storage: Mode -> LHS -> Abbreviation
    abbrevs: HashMap<Mode, HashMap<String, Abbreviation>>,
}

impl SimpleAbbreviationManager {
    /// Create a new empty abbreviation manager
    pub fn new() -> Self {
        SimpleAbbreviationManager {
            abbrevs: HashMap::new(),
        }
    }

    fn get_mode_map(&self, mode: Mode) -> Option<&HashMap<String, Abbreviation>> {
        self.abbrevs.get(&mode)
    }

    fn get_mode_map_mut(&mut self, mode: Mode) -> &mut HashMap<String, Abbreviation> {
        self.abbrevs.entry(mode).or_default()
    }
}

impl AbbreviationManager for SimpleAbbreviationManager {
    fn add(&mut self, mode: Mode, lhs: &str, rhs: &str, noremap: bool, buffer_local: bool) -> VimResult<()> {
        if lhs.is_empty() {
            return Err(VimError::Error(0, "Abbreviation LHS cannot be empty".to_string()));
        }
        
        let abbrev = Abbreviation {
            lhs: lhs.to_string(),
            rhs: rhs.to_string(),
            noremap,
            silent: false,
            buffer_local,
        };

        self.get_mode_map_mut(mode).insert(lhs.to_string(), abbrev);
        Ok(())
    }

    fn remove(&mut self, mode: Mode, lhs: &str) -> VimResult<()> {
        if let Some(map) = self.abbrevs.get_mut(&mode) {
            if map.remove(lhs).is_some() {
                Ok(())
            } else {
                Err(VimError::Error(0, format!("No such abbreviation: {}", lhs)))
            }
        } else {
            Err(VimError::Error(0, format!("No such abbreviation: {}", lhs)))
        }
    }

    fn get(&self, mode: Mode, lhs: &str) -> Option<&Abbreviation> {
        self.get_mode_map(mode).and_then(|m| m.get(lhs))
    }

    fn check(&self, mode: Mode, word: &str) -> Option<&Abbreviation> {
        // Vim's abbreviation logic:
        // 1. The typed character must be a non-keyword character (handled by caller usually)
        // 2. The word before the cursor must match the lhs of an abbreviation
        // 3. The character *before* the word must be a non-keyword character (or start of line)
        //    (This ensures we don't expand "foobar" when "bar" is the abbreviation)
        //
        // However, `word` passed here is assumed to be the candidate string.
        // The caller is responsible for extracting the "word" candidates.
        // Vim checks: is there an abbrev for `word`?
        
        // Note: Insert mode and Replace mode share abbreviations.
        let check_mode = if mode == Mode::Replace { Mode::Insert } else { mode };
        
        self.get_mode_map(check_mode).and_then(|m| m.get(word))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_get_abbrev() {
        let mut mgr = SimpleAbbreviationManager::new();
        mgr.add(Mode::Insert, "ad", "advertisement", false, false).unwrap();
        
        let a = mgr.get(Mode::Insert, "ad").unwrap();
        assert_eq!(a.rhs, "advertisement");
    }

    #[test]
    fn test_check_abbrev() {
        let mut mgr = SimpleAbbreviationManager::new();
        mgr.add(Mode::Insert, "ad", "advertisement", false, false).unwrap();
        
        assert!(mgr.check(Mode::Insert, "ad").is_some());
        assert!(mgr.check(Mode::Insert, "bad").is_none());
        assert!(mgr.check(Mode::Normal, "ad").is_none()); // Wrong mode
    }
}
