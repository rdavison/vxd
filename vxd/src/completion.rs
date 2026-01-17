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
