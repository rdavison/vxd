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
// Simple Search Engine
// ============================================================================

/// A minimal search engine over in-memory lines.
#[derive(Debug, Clone, Default)]
pub struct SimpleSearchEngine {
    lines: Vec<String>,
    state: SearchState,
}

impl SimpleSearchEngine {
    /// Create a new search engine with the given lines.
    pub fn new(lines: Vec<String>) -> Self {
        SimpleSearchEngine {
            lines,
            state: SearchState::default(),
        }
    }

    /// Replace the engine's lines.
    pub fn set_lines(&mut self, lines: Vec<String>) {
        self.lines = lines;
    }

    fn case_sensitive(options: &SearchOptions, pattern: &str) -> bool {
        if options.ignorecase {
            if options.smartcase && pattern.chars().any(|ch| ch.is_uppercase()) {
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    fn match_text(line: &str, start_col: usize, needle: &str, case_sensitive: bool) -> Option<usize> {
        if start_col > line.len() {
            return None;
        }
        let search_line = if case_sensitive {
            std::borrow::Cow::Borrowed(line)
        } else {
            std::borrow::Cow::Owned(line.to_lowercase())
        };
        let needle = if case_sensitive {
            std::borrow::Cow::Borrowed(needle)
        } else {
            std::borrow::Cow::Owned(needle.to_lowercase())
        };
        let haystack = &search_line[start_col..];
        haystack.find(needle.as_ref()).map(|pos| start_col + pos)
    }

    fn match_text_backward(
        line: &str,
        end_col: usize,
        needle: &str,
        case_sensitive: bool,
    ) -> Option<usize> {
        let end_col = end_col.min(line.len());
        let search_line = if case_sensitive {
            std::borrow::Cow::Borrowed(line)
        } else {
            std::borrow::Cow::Owned(line.to_lowercase())
        };
        let needle = if case_sensitive {
            std::borrow::Cow::Borrowed(needle)
        } else {
            std::borrow::Cow::Owned(needle.to_lowercase())
        };
        let haystack = &search_line[..end_col];
        haystack.rfind(needle.as_ref())
    }

    fn build_match(
        line_idx: usize,
        match_col: usize,
        needle_len: usize,
        line: &str,
    ) -> SearchMatch {
        SearchMatch {
            start: CursorPosition::new(LineNr(line_idx + 1), match_col),
            end: CursorPosition::new(LineNr(line_idx + 1), match_col + needle_len),
            text: line[match_col..match_col + needle_len].to_string(),
            groups: Vec::new(),
        }
    }

    fn search_forward(
        &self,
        pattern: &SearchPattern,
        from: CursorPosition,
        options: &SearchOptions,
    ) -> Option<SearchMatch> {
        let case_sensitive = Self::case_sensitive(options, &pattern.pattern);
        let start_line = from.line.0.saturating_sub(1);
        for (idx, line) in self.lines.iter().enumerate().skip(start_line) {
            let start_col = if idx == start_line { from.col } else { 0 };
            if let Some(match_col) =
                Self::match_text(line, start_col, &pattern.pattern, case_sensitive)
            {
                return Some(Self::build_match(idx, match_col, pattern.pattern.len(), line));
            }
        }
        None
    }

    fn search_backward(
        &self,
        pattern: &SearchPattern,
        from: CursorPosition,
        options: &SearchOptions,
    ) -> Option<SearchMatch> {
        let case_sensitive = Self::case_sensitive(options, &pattern.pattern);
        let start_line = from.line.0.saturating_sub(1);
        for (idx, line) in self.lines.iter().enumerate().take(start_line + 1).rev() {
            let end_col = if idx == start_line { from.col } else { line.len() };
            if let Some(match_col) =
                Self::match_text_backward(line, end_col, &pattern.pattern, case_sensitive)
            {
                return Some(Self::build_match(idx, match_col, pattern.pattern.len(), line));
            }
        }
        None
    }
}

impl SearchEngine for SimpleSearchEngine {
    fn compile(&self, pattern: &str, _options: &SearchOptions) -> VimResult<SearchPattern> {
        if pattern.is_empty() {
            return Err(VimError::InvalidPattern("empty pattern".to_string()));
        }
        Ok(SearchPattern::forward(pattern))
    }

    fn search(
        &self,
        pattern: &SearchPattern,
        from: CursorPosition,
        options: &SearchOptions,
    ) -> VimResult<Option<SearchMatch>> {
        if pattern.pattern.is_empty() {
            return Err(VimError::PatternNotFound(pattern.pattern.clone()));
        }
        let found = match pattern.direction {
            Direction::Forward => self.search_forward(pattern, from, options),
            Direction::Backward => self.search_backward(pattern, from, options),
        };
        if found.is_none() && options.wrapscan {
            let wrap_from = match pattern.direction {
                Direction::Forward => CursorPosition::ORIGIN,
                Direction::Backward => {
                    let last_line = self.lines.len().max(1);
                    CursorPosition::new(LineNr(last_line), usize::MAX)
                }
            };
            let wrapped = match pattern.direction {
                Direction::Forward => self.search_forward(pattern, wrap_from, options),
                Direction::Backward => self.search_backward(pattern, wrap_from, options),
            };
            return Ok(wrapped);
        }
        Ok(found)
    }

    fn find_all(
        &self,
        pattern: &SearchPattern,
        start: CursorPosition,
        end: CursorPosition,
        options: &SearchOptions,
    ) -> VimResult<Vec<SearchMatch>> {
        if pattern.pattern.is_empty() {
            return Err(VimError::PatternNotFound(pattern.pattern.clone()));
        }
        let case_sensitive = Self::case_sensitive(options, &pattern.pattern);
        let mut matches = Vec::new();
        let start_line = start.line.0.saturating_sub(1);
        let end_line = end.line.0.saturating_sub(1).min(self.lines.len().saturating_sub(1));

        for idx in start_line..=end_line {
            let line = &self.lines[idx];
            let line_start = if idx == start_line { start.col } else { 0 };
            let mut line_end = if idx == end_line { end.col } else { line.len() };
            line_end = line_end.min(line.len());
            let mut offset = line_start;
            while offset <= line_end {
                let search_line = if case_sensitive {
                    std::borrow::Cow::Borrowed(line)
                } else {
                    std::borrow::Cow::Owned(line.to_lowercase())
                };
                let needle = if case_sensitive {
                    std::borrow::Cow::Borrowed(pattern.pattern.as_str())
                } else {
                    std::borrow::Cow::Owned(pattern.pattern.to_lowercase())
                };
                let haystack = &search_line[offset..line_end];
                if let Some(pos) = haystack.find(needle.as_ref()) {
                    let match_col = offset + pos;
                    matches.push(Self::build_match(
                        idx,
                        match_col,
                        pattern.pattern.len(),
                        line,
                    ));
                    let advance = pattern.pattern.len().max(1);
                    offset = match_col + advance;
                } else {
                    break;
                }
            }
        }

        Ok(matches)
    }

    fn count_matches(&self, pattern: &SearchPattern, options: &SearchOptions) -> VimResult<usize> {
        let end_line = self.lines.len().max(1);
        let start = CursorPosition::ORIGIN;
        let end = CursorPosition::new(LineNr(end_line), usize::MAX);
        Ok(self.find_all(pattern, start, end, options)?.len())
    }

    fn search_word(
        &self,
        word: &str,
        direction: Direction,
        _whole_word: bool,
    ) -> VimResult<SearchPattern> {
        Ok(SearchPattern {
            pattern: word.to_string(),
            direction,
            offset: SearchOffset::None,
            valid: !word.is_empty(),
        })
    }

    fn state(&self) -> &SearchState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut SearchState {
        &mut self.state
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

    #[test]
    fn test_simple_search_ignorecase_matches() {
        let engine = SimpleSearchEngine::new(vec!["Alpha beta".to_string()]);
        let options = SearchOptions {
            ignorecase: true,
            ..Default::default()
        };
        let pattern = SearchPattern::forward("alpha");

        let found = engine
            .search(&pattern, CursorPosition::ORIGIN, &options)
            .unwrap()
            .expect("expected match");

        assert_eq!(found.start, CursorPosition::new(LineNr(1), 0));
        assert_eq!(found.text, "Alpha");
    }

    #[test]
    fn test_simple_search_smartcase_disables_ignorecase() {
        let engine = SimpleSearchEngine::new(vec!["alpha beta".to_string()]);
        let options = SearchOptions {
            ignorecase: true,
            smartcase: true,
            ..Default::default()
        };

        let lower = SearchPattern::forward("beta");
        let upper = SearchPattern::forward("BETA");

        let lower_match = engine
            .search(&lower, CursorPosition::ORIGIN, &options)
            .unwrap();
        let upper_match = engine
            .search(&upper, CursorPosition::ORIGIN, &options)
            .unwrap();

        assert!(lower_match.is_some());
        assert!(upper_match.is_none());
    }

    #[test]
    fn test_simple_search_case_sensitive_default() {
        let engine = SimpleSearchEngine::new(vec!["Alpha beta".to_string()]);
        let options = SearchOptions::default();
        let pattern = SearchPattern::forward("alpha");

        let found = engine.search(&pattern, CursorPosition::ORIGIN, &options).unwrap();
        assert!(found.is_none());
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
