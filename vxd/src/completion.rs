//! Completion system.
//!
//! Vim provides various types of completion for insert mode and command line.

use crate::types::*;

// ============================================================================
// Completion Types
// ============================================================================

/// Type of completion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionKind {
    /// Keyword completion (Ctrl-N, Ctrl-P)
    Keyword,
    /// Line completion (Ctrl-X Ctrl-L)
    Line,
    /// File path completion (Ctrl-X Ctrl-F)
    File,
    /// Dictionary completion (Ctrl-X Ctrl-K)
    Dictionary,
    /// Thesaurus completion (Ctrl-X Ctrl-T)
    Thesaurus,
    /// Tag completion (Ctrl-X Ctrl-])
    Tag,
    /// Include completion (Ctrl-X Ctrl-I)
    Include,
    /// Define completion (Ctrl-X Ctrl-D)
    Define,
    /// Vim command completion (Ctrl-X Ctrl-V)
    Command,
    /// User-defined completion (Ctrl-X Ctrl-U)
    User,
    /// Omni completion (Ctrl-X Ctrl-O)
    Omni,
    /// Spelling suggestions (Ctrl-X s)
    Spelling,
    /// Buffer completion
    Buffer,
}

/// A completion item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    /// The word to insert
    pub word: String,
    /// Abbreviation (shown in menu)
    pub abbr: Option<String>,
    /// Menu text (additional info)
    pub menu: Option<String>,
    /// Additional info (shown in preview)
    pub info: Option<String>,
    /// Kind indicator (v=variable, f=function, etc.)
    pub kind: Option<String>,
    /// Sort priority
    pub priority: i32,
    /// Whether this is a duplicate
    pub dup: bool,
    /// User data
    pub user_data: Option<String>,
}

impl CompletionItem {
    /// Create a simple completion item
    pub fn new(word: impl Into<String>) -> Self {
        CompletionItem {
            word: word.into(),
            abbr: None,
            menu: None,
            info: None,
            kind: None,
            priority: 0,
            dup: false,
            user_data: None,
        }
    }

    /// Set the menu text
    pub fn with_menu(mut self, menu: impl Into<String>) -> Self {
        self.menu = Some(menu.into());
        self
    }

    /// Set the kind
    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }
}

/// Completion menu state
#[derive(Debug, Clone, Default)]
pub struct CompletionState {
    /// Current completion items
    pub items: Vec<CompletionItem>,
    /// Currently selected index
    pub selected: Option<usize>,
    /// Original text before completion
    pub original: String,
    /// Start column of completion
    pub start_col: usize,
    /// Completion kind
    pub kind: Option<CompletionKind>,
}

// ============================================================================
// Completion Engine Trait
// ============================================================================

/// Trait for completion functionality
pub trait CompletionEngine {
    /// Get completions for the current context
    fn complete(
        &self,
        kind: CompletionKind,
        line: &str,
        col: usize,
    ) -> VimResult<Vec<CompletionItem>>;

    /// Get the completion state
    fn state(&self) -> &CompletionState;

    /// Start completion
    fn start(&mut self, kind: CompletionKind, line: &str, col: usize) -> VimResult<()>;

    /// Select next item
    fn select_next(&mut self) -> Option<&CompletionItem>;

    /// Select previous item
    fn select_prev(&mut self) -> Option<&CompletionItem>;

    /// Accept selected completion
    fn accept(&mut self) -> Option<CompletionItem>;

    /// Cancel completion
    fn cancel(&mut self);

    /// Check if completion is active
    fn is_active(&self) -> bool {
        !self.state().items.is_empty()
    }
}

// ============================================================================
// Basic Completion Engine
// ============================================================================

/// A simple completion engine backed by in-memory buffer lines.
#[derive(Debug, Default)]
pub struct BufferCompletionEngine {
    lines: Vec<String>,
    state: CompletionState,
}

impl BufferCompletionEngine {
    /// Create a completion engine with initial buffer lines.
    pub fn new(lines: Vec<String>) -> Self {
        BufferCompletionEngine {
            lines,
            state: CompletionState::default(),
        }
    }

    /// Replace the buffer lines used for completion.
    pub fn set_lines(&mut self, lines: Vec<String>) {
        self.lines = lines;
    }

    fn word_prefix(line: &str, col: usize) -> (usize, &str) {
        let col = col.min(line.len());
        let bytes = line.as_bytes();
        let mut start = col;
        while start > 0 {
            let byte = bytes[start - 1];
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                start -= 1;
            } else {
                break;
            }
        }
        (start, &line[start..col])
    }

    fn collect_words(&self, prefix: &str) -> Vec<CompletionItem> {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        let mut items = Vec::new();
        for line in &self.lines {
            let mut i = 0;
            let bytes = line.as_bytes();
            while i < bytes.len() {
                while i < bytes.len()
                    && !(bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
                {
                    i += 1;
                }
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
                {
                    i += 1;
                }
                if start < i {
                    let word = &line[start..i];
                    if word.starts_with(prefix) && seen.insert(word.to_string()) {
                        items.push(CompletionItem::new(word));
                    }
                }
            }
        }
        items
    }

    fn collect_lines(&self, prefix: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        for line in &self.lines {
            if line.starts_with(prefix) {
                items.push(CompletionItem::new(line.clone()));
            }
        }
        items
    }
}

impl CompletionEngine for BufferCompletionEngine {
    fn complete(
        &self,
        kind: CompletionKind,
        line: &str,
        col: usize,
    ) -> VimResult<Vec<CompletionItem>> {
        let items = match kind {
            CompletionKind::Keyword | CompletionKind::Buffer => {
                let (_, prefix) = Self::word_prefix(line, col);
                if prefix.is_empty() {
                    Vec::new()
                } else {
                    self.collect_words(prefix)
                }
            }
            CompletionKind::Line => {
                let col = col.min(line.len());
                let prefix = &line[..col];
                if prefix.is_empty() {
                    Vec::new()
                } else {
                    self.collect_lines(prefix)
                }
            }
            _ => Vec::new(),
        };
        Ok(items)
    }

    fn state(&self) -> &CompletionState {
        &self.state
    }

    fn start(&mut self, kind: CompletionKind, line: &str, col: usize) -> VimResult<()> {
        let start_col = match kind {
            CompletionKind::Keyword | CompletionKind::Buffer => Self::word_prefix(line, col).0,
            CompletionKind::Line => 0,
            _ => col.min(line.len()),
        };
        let items = self.complete(kind, line, col)?;
        self.state = CompletionState {
            items,
            selected: None,
            original: line.to_string(),
            start_col,
            kind: Some(kind),
        };
        if !self.state.items.is_empty() {
            self.state.selected = Some(0);
        }
        Ok(())
    }

    fn select_next(&mut self) -> Option<&CompletionItem> {
        let len = self.state.items.len();
        if len == 0 {
            return None;
        }
        let next = match self.state.selected {
            Some(idx) => (idx + 1) % len,
            None => 0,
        };
        self.state.selected = Some(next);
        self.state.items.get(next)
    }

    fn select_prev(&mut self) -> Option<&CompletionItem> {
        let len = self.state.items.len();
        if len == 0 {
            return None;
        }
        let prev = match self.state.selected {
            Some(idx) => (idx + len - 1) % len,
            None => 0,
        };
        self.state.selected = Some(prev);
        self.state.items.get(prev)
    }

    fn accept(&mut self) -> Option<CompletionItem> {
        let selected = self.state.selected.and_then(|idx| self.state.items.get(idx).cloned());
        self.state.items.clear();
        self.state.selected = None;
        self.state.kind = None;
        selected
    }

    fn cancel(&mut self) {
        self.state.items.clear();
        self.state.selected = None;
        self.state.kind = None;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_item() {
        let item = CompletionItem::new("hello")
            .with_menu("greeting")
            .with_kind("w");

        assert_eq!(item.word, "hello");
        assert_eq!(item.menu, Some("greeting".into()));
        assert_eq!(item.kind, Some("w".into()));
    }

    #[test]
    fn test_keyword_completion_from_buffer() {
        let mut engine = BufferCompletionEngine::new(vec![
            "alpha beta".to_string(),
            "alphabet soup".to_string(),
            "alpine trail".to_string(),
        ]);

        engine
            .start(CompletionKind::Keyword, "al", 2)
            .unwrap();
        let words: Vec<String> = engine
            .state()
            .items
            .iter()
            .map(|item| item.word.clone())
            .collect();
        assert_eq!(words, vec!["alpha", "alphabet", "alpine"]);
    }

    #[test]
    fn test_completion_selection_and_accept() {
        let mut engine = BufferCompletionEngine::new(vec![
            "hello world".to_string(),
            "helium".to_string(),
            "help".to_string(),
        ]);

        engine
            .start(CompletionKind::Keyword, "he", 2)
            .unwrap();
        assert_eq!(engine.state().selected, Some(0));
        let _ = engine.select_next();
        assert_eq!(engine.state().selected, Some(1));
        let accepted = engine.accept().unwrap();
        assert_eq!(accepted.word, "helium");
        assert!(engine.state().items.is_empty());
    }

    #[test]
    fn test_line_completion_matches_prefix() {
        let mut engine = BufferCompletionEngine::new(vec![
            "set number".to_string(),
            "set nowrap".to_string(),
            "syntax on".to_string(),
        ]);

        engine
            .start(CompletionKind::Line, "set ", 4)
            .unwrap();
        let lines: Vec<String> = engine
            .state()
            .items
            .iter()
            .map(|item| item.word.clone())
            .collect();
        assert_eq!(lines, vec!["set number", "set nowrap"]);
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Completion Behavioral Tests
        //!
        //! ## Key Quirks
        //!
        //! 1. **Completion direction**: Ctrl-N = next, Ctrl-P = previous.
        //!
        //! 2. **Multiple sources**: Different Ctrl-X sequences use different sources.
        //!
        //! 3. **Popup menu**: 'pumheight' controls max visible items.
        //!
        //! 4. **Completeopt**: Controls how completion menu behaves.
    }
}
