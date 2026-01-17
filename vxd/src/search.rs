//! Search and pattern matching.
//!
//! Vim's search functionality uses its own regex dialect with options
//! for case sensitivity, magic mode, and incremental search.

use crate::cursor::CursorPosition;
use crate::types::*;

// ============================================================================
// Search Types
// ============================================================================

/// Direction of search
pub use crate::types::Direction;

/// Search options/flags
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchOptions {
    /// Ignore case in search
    pub ignorecase: bool,
    /// Override ignorecase if pattern has uppercase
    pub smartcase: bool,
    /// Use magic mode (special chars have special meaning)
    pub magic: bool,
    /// Wrap around end of file
    pub wrapscan: bool,
    /// Show incremental search results
    pub incsearch: bool,
    /// Highlight all matches
    pub hlsearch: bool,
}

/// A compiled search pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPattern {
    /// The original pattern string
    pub pattern: String,
    /// Direction of the search
    pub direction: Direction,
    /// Offset from match (e.g., /pattern/+2)
    pub offset: SearchOffset,
    /// Whether the pattern is valid
    pub valid: bool,
}

impl SearchPattern {
    /// Create a new forward search pattern
    pub fn forward(pattern: impl Into<String>) -> Self {
        SearchPattern {
            pattern: pattern.into(),
            direction: Direction::Forward,
            offset: SearchOffset::None,
            valid: true,
        }
    }

    /// Create a new backward search pattern
    pub fn backward(pattern: impl Into<String>) -> Self {
        SearchPattern {
            pattern: pattern.into(),
            direction: Direction::Backward,
            offset: SearchOffset::None,
            valid: true,
        }
    }
}

/// Offset applied after finding a match
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchOffset {
    /// No offset
    #[default]
    None,
    /// Line offset from match (+n or -n)
    Line(i32),
    /// End of match plus offset (e+n or e-n)
    End(i32),
    /// Start of match plus offset (s+n or s-n)
    Start(i32),
    /// Column offset (b+n or b-n)
    Column(i32),
}

/// A search match result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    /// Start position of match
    pub start: CursorPosition,
    /// End position of match
    pub end: CursorPosition,
    /// The matched text
    pub text: String,
    /// Capture groups (if any)
    pub groups: Vec<String>,
}

// ============================================================================
// Search State
// ============================================================================

/// State of the search system
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Last search pattern
    pub last_pattern: Option<SearchPattern>,
    /// Last search direction
    pub last_direction: Direction,
    /// Current match index (for n/N navigation)
    pub match_index: usize,
    /// Total number of matches
    pub match_count: usize,
    /// Whether search highlighting is active
    pub highlighting: bool,
}

// ============================================================================
// Search Engine Trait
// ============================================================================

/// Trait for search functionality
pub trait SearchEngine {
    /// Compile a pattern for searching
    fn compile(&self, pattern: &str, options: &SearchOptions) -> VimResult<SearchPattern>;

    /// Search from a position
    fn search(
        &self,
        pattern: &SearchPattern,
        from: CursorPosition,
        options: &SearchOptions,
    ) -> VimResult<Option<SearchMatch>>;

    /// Search for the next match
    fn search_next(
        &self,
        pattern: &SearchPattern,
        from: CursorPosition,
        options: &SearchOptions,
    ) -> VimResult<Option<SearchMatch>> {
        self.search(pattern, from, options)
    }

    /// Search for the previous match
    fn search_prev(
        &self,
        pattern: &SearchPattern,
        from: CursorPosition,
        options: &SearchOptions,
    ) -> VimResult<Option<SearchMatch>> {
        let mut reversed = pattern.clone();
        reversed.direction = pattern.direction.reverse();
        self.search(&reversed, from, options)
    }

    /// Find all matches in a range
    fn find_all(
        &self,
        pattern: &SearchPattern,
        start: CursorPosition,
        end: CursorPosition,
        options: &SearchOptions,
    ) -> VimResult<Vec<SearchMatch>>;

    /// Count matches in the buffer
    fn count_matches(&self, pattern: &SearchPattern, options: &SearchOptions) -> VimResult<usize>;

    /// Search for word under cursor (*, #)
    fn search_word(
        &self,
        word: &str,
        direction: Direction,
        whole_word: bool,
    ) -> VimResult<SearchPattern>;

    /// Get the search state
    fn state(&self) -> &SearchState;

    /// Get mutable search state
    fn state_mut(&mut self) -> &mut SearchState;

    /// Set the last search pattern
    fn set_last_pattern(&mut self, pattern: SearchPattern) {
        self.state_mut().last_pattern = Some(pattern);
    }

    /// Get the last search pattern
    fn last_pattern(&self) -> Option<&SearchPattern> {
        self.state().last_pattern.as_ref()
    }
}

// ============================================================================
// Substitute Command Types
// ============================================================================

/// Flags for the substitute command
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubstituteFlags {
    /// Replace all occurrences on each line
    pub global: bool,
    /// Confirm each substitution
    pub confirm: bool,
    /// Report number of substitutions
    pub report: bool,
    /// Ignore case
    pub ignore_case: bool,
    /// Don't ignore case
    pub no_ignore_case: bool,
    /// Use last search pattern
    pub use_last_pattern: bool,
    /// Print the last line where substitution occurred
    pub print: bool,
}

/// A substitute command specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstituteSpec {
    /// Pattern to match
    pub pattern: String,
    /// Replacement string
    pub replacement: String,
    /// Flags
    pub flags: SubstituteFlags,
    /// Range of lines to operate on
    pub range: Option<LineRange>,
}

/// Apply a substitute operation to the given lines.
pub fn apply_substitute(
    lines: &[String],
    spec: &SubstituteSpec,
    use_last_pattern: Option<&str>,
) -> VimResult<Vec<String>> {
    let pattern = if spec.flags.use_last_pattern {
        use_last_pattern.unwrap_or("").to_string()
    } else {
        spec.pattern.clone()
    };

    if pattern.is_empty() {
        return Err(VimError::Error(486, "Pattern not found".to_string()));
    }

    let start = spec.range.map(|r| r.start.0 - 1).unwrap_or(0);
    let end = spec
        .range
        .map(|r| r.end.0)
        .unwrap_or(lines.len())
        .min(lines.len());

    let mut out = lines.to_vec();
    for idx in start..end {
        if let Some(line) = out.get_mut(idx) {
            *line = substitute_line(line, &pattern, &spec.replacement, spec.flags.global);
        }
    }

    Ok(out)
}

fn substitute_line(line: &str, pattern: &str, replacement: &str, global: bool) -> String {
    if global {
        line.replace(pattern, replacement)
    } else if let Some(pos) = line.find(pattern) {
        let mut out = String::new();
        out.push_str(&line[..pos]);
        out.push_str(replacement);
        out.push_str(&line[pos + pattern.len()..]);
        out
    } else {
        line.to_string()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_pattern_creation() {
        let fwd = SearchPattern::forward("test");
        assert_eq!(fwd.direction, Direction::Forward);
        assert_eq!(fwd.pattern, "test");

        let bwd = SearchPattern::backward("test");
        assert_eq!(bwd.direction, Direction::Backward);
    }

    #[test]
    fn test_substitute_first_match_only() {
        let lines = vec!["one one".to_string()];
        let spec = SubstituteSpec {
            pattern: "one".to_string(),
            replacement: "two".to_string(),
            flags: SubstituteFlags::default(),
            range: None,
        };

        let out = apply_substitute(&lines, &spec, None).unwrap();
        assert_eq!(out, vec!["two one".to_string()]);
    }

    #[test]
    fn test_substitute_global() {
        let lines = vec!["one one".to_string()];
        let mut flags = SubstituteFlags::default();
        flags.global = true;
        let spec = SubstituteSpec {
            pattern: "one".to_string(),
            replacement: "two".to_string(),
            flags,
            range: None,
        };

        let out = apply_substitute(&lines, &spec, None).unwrap();
        assert_eq!(out, vec!["two two".to_string()]);
    }

    #[test]
    fn test_substitute_range() {
        let lines = vec![
            "one".to_string(),
            "one".to_string(),
            "one".to_string(),
        ];
        let spec = SubstituteSpec {
            pattern: "one".to_string(),
            replacement: "two".to_string(),
            flags: SubstituteFlags::default(),
            range: Some(LineRange::new(LineNr(2), LineNr(2))),
        };

        let out = apply_substitute(&lines, &spec, None).unwrap();
        assert_eq!(
            out,
            vec!["one".to_string(), "two".to_string(), "one".to_string()]
        );
    }

    #[test]
    fn test_substitute_uses_last_pattern() {
        let lines = vec!["alpha beta".to_string()];
        let mut flags = SubstituteFlags::default();
        flags.use_last_pattern = true;
        let spec = SubstituteSpec {
            pattern: "".to_string(),
            replacement: "omega".to_string(),
            flags,
            range: None,
        };

        let out = apply_substitute(&lines, &spec, Some("beta")).unwrap();
        assert_eq!(out, vec!["alpha omega".to_string()]);
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Search Behavioral Tests
        //!
        //! Tests derived from Neovim's test suite:
        //! - test/functional/editor/search_spec.lua
        //! - test/functional/legacy/search_spec.lua
        //!
        //! ## Key Quirks
        //!
        //! 1. **Magic mode**: By default, many special chars need escaping.
        //!    `\v` = very magic, `\m` = magic, `\M` = nomagic, `\V` = very nomagic.
        //!
        //! 2. **Smartcase**: If pattern has uppercase, ignorecase is disabled.
        //!
        //! 3. **Wrapscan**: By default, search wraps around file boundaries.
        //!
        //! 4. **Search offset**: `/pattern/+2` moves cursor 2 lines down from match.
        //!
        //! 5. **Star search**: `*` adds word boundaries automatically (`\<word\>`).
    }
}
