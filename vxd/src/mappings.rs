//! Key mapping system.
//!
//! This module handles key mappings (map, noremap, etc.) across different modes.
//!
//! # Key Behavioral Contracts
//!
//! - Mappings are mode-specific (nmap, imap, etc.)
//! - Mappings can be recursive (map) or non-recursive (noremap)
//! - Mappings are matched against a prefix of pending input
//! - Longest match wins

use crate::modes::Mode;
use crate::types::*;
use std::collections::HashMap;

// ============================================================================
// Mapping Types
// ============================================================================

/// A key mapping definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapping {
    /// Left-hand side (trigger keys)
    pub lhs: String,
    /// Right-hand side (replacement keys)
    pub rhs: String,
    /// Whether the mapping is non-recursive
    pub noremap: bool,
    /// Whether the mapping is silent (no echo)
    pub silent: bool,
    /// Whether to wait for more keys (nowait=false means wait if ambiguity)
    pub nowait: bool,
}

/// Result of checking input against mappings
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappingCheckResult {
    /// Full match found (contains the mapping)
    FullMatch(Mapping),
    /// Input matches the prefix of one or more mappings (need more input)
    PartialMatch,
    /// No match found
    NoMatch,
}

// ============================================================================
// Mapping Manager Trait
// ============================================================================

/// Manages key mappings
pub trait MappingManager {
    /// Add a mapping
    fn add(&mut self, mode: Mode, lhs: &str, rhs: &str, noremap: bool) -> VimResult<()>;

    /// Remove a mapping
    fn remove(&mut self, mode: Mode, lhs: &str) -> VimResult<()>;

    /// Get a specific mapping
    fn get(&self, mode: Mode, lhs: &str) -> Option<&Mapping>;

    /// Check if input matches any mapping
    ///
    /// # Returns
    /// - `FullMatch(mapping)` if `input` exactly matches `lhs` of a mapping.
    ///   If multiple match (unlikely with unique LHS), one is returned.
    ///   Note: This doesn't handle the "longest match" logic if `input` is just a prefix
    ///   of a longer mapping. The caller typically accumulates input.
    ///   If `input` matches a mapping BUT is also a prefix of another mapping,
    ///   Vim typically waits (PartialMatch).
    ///   Here, we return `FullMatch` if it matches. The ambiguity handling (timeout vs match)
    ///   is often logic above this.
    ///   Actually, let's refine:
    ///   - If exact match AND no longer match possible -> FullMatch
    ///   - If exact match BUT longer match possible -> PartialMatch (or FullMatch with indication?)
    ///   Vim's logic: if match found, but could be longer, wait 'timeoutlen'.
    ///   To support this, we need to know if it's *also* a partial match.
    fn check(&self, mode: Mode, input: &str) -> MappingCheckResult;
}

// ============================================================================
// Simple Implementation
// ============================================================================

/// A HashMap-based mapping manager
#[derive(Debug, Default, Clone)]
pub struct SimpleMappingManager {
    /// Mappings storage: Mode -> LHS -> Mapping
    /// Note: Mode hash might be tricky if we want `vmap` to cover multiple Visual modes.
    /// For simplicity, we assume strict mode matching or caller handles normalization.
    mappings: HashMap<Mode, HashMap<String, Mapping>>,
}

impl SimpleMappingManager {
    /// Create a new empty mapping manager
    pub fn new() -> Self {
        SimpleMappingManager {
            mappings: HashMap::new(),
        }
    }

    fn get_mode_map(&self, mode: Mode) -> Option<&HashMap<String, Mapping>> {
        self.mappings.get(&mode)
    }

    fn get_mode_map_mut(&mut self, mode: Mode) -> &mut HashMap<String, Mapping> {
        self.mappings.entry(mode).or_default()
    }
}

impl MappingManager for SimpleMappingManager {
    fn add(&mut self, mode: Mode, lhs: &str, rhs: &str, noremap: bool) -> VimResult<()> {
        if lhs.is_empty() {
            return Err(VimError::Error(0, "Mapping LHS cannot be empty".to_string()));
        }
        
        let mapping = Mapping {
            lhs: lhs.to_string(),
            rhs: rhs.to_string(),
            noremap,
            silent: false,
            nowait: false,
        };

        self.get_mode_map_mut(mode).insert(lhs.to_string(), mapping);
        Ok(())
    }

    fn remove(&mut self, mode: Mode, lhs: &str) -> VimResult<()> {
        if let Some(map) = self.mappings.get_mut(&mode) {
            if map.remove(lhs).is_some() {
                Ok(())
            } else {
                Err(VimError::Error(0, format!("No such mapping: {}", lhs)))
            }
        } else {
            Err(VimError::Error(0, format!("No such mapping: {}", lhs)))
        }
    }

    fn get(&self, mode: Mode, lhs: &str) -> Option<&Mapping> {
        self.get_mode_map(mode).and_then(|m| m.get(lhs))
    }

    fn check(&self, mode: Mode, input: &str) -> MappingCheckResult {
        let Some(map) = self.get_mode_map(mode) else {
            return MappingCheckResult::NoMatch;
        };

        // Check for exact match
        let exact_match = map.get(input).cloned();

        // Check for partial matches (is `input` a prefix of any LHS?)
        let partial_match_exists = map.keys().any(|lhs| lhs.starts_with(input) && lhs != input);

        match (exact_match, partial_match_exists) {
            (Some(m), false) => MappingCheckResult::FullMatch(m),
            (Some(_), true) => MappingCheckResult::PartialMatch, // Ambiguous, wait for more
            (None, true) => MappingCheckResult::PartialMatch,
            (None, false) => MappingCheckResult::NoMatch,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_get_mapping() {
        let mut mgr = SimpleMappingManager::new();
        mgr.add(Mode::Normal, "lhs", "rhs", false).unwrap();
        
        let m = mgr.get(Mode::Normal, "lhs").unwrap();
        assert_eq!(m.rhs, "rhs");
        assert_eq!(m.noremap, false);
    }

    #[test]
    fn test_check_partial_match() {
        let mut mgr = SimpleMappingManager::new();
        mgr.add(Mode::Normal, "ab", "rhs", false).unwrap();
        
        assert!(matches!(mgr.check(Mode::Normal, "a"), MappingCheckResult::PartialMatch));
        assert!(matches!(mgr.check(Mode::Normal, "ab"), MappingCheckResult::FullMatch(_)));
        assert!(matches!(mgr.check(Mode::Normal, "abc"), MappingCheckResult::NoMatch));
    }

    #[test]
    fn test_check_ambiguous_match() {
        let mut mgr = SimpleMappingManager::new();
        mgr.add(Mode::Normal, "a", "short", false).unwrap();
        mgr.add(Mode::Normal, "ab", "long", false).unwrap();
        
        // "a" matches the first mapping, but is also a prefix of "ab".
        // Should return PartialMatch to indicate we should wait (timeout logic is external).
        assert!(matches!(mgr.check(Mode::Normal, "a"), MappingCheckResult::PartialMatch));
        
        // "ab" matches the second mapping exactly and is not a prefix of anything else.
        match mgr.check(Mode::Normal, "ab") {
            MappingCheckResult::FullMatch(m) => assert_eq!(m.rhs, "long"),
            _ => panic!("Expected FullMatch"),
        }
    }
    
    #[test]
    fn test_remove_mapping() {
        let mut mgr = SimpleMappingManager::new();
        mgr.add(Mode::Normal, "a", "b", false).unwrap();
        mgr.remove(Mode::Normal, "a").unwrap();
        assert!(mgr.get(Mode::Normal, "a").is_none());
    }
}
