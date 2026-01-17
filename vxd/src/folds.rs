//! Folding system.
//!
//! Folds allow hiding sections of text. Vim supports multiple fold methods.

use crate::types::*;

// ============================================================================
// Fold Types
// ============================================================================

/// Fold method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FoldMethod {
    /// Manual folds created by user
    #[default]
    Manual,
    /// Folds based on indentation
    Indent,
    /// Folds based on expression
    Expr,
    /// Folds based on markers in text
    Marker,
    /// Folds based on syntax highlighting
    Syntax,
    /// Folds based on diff blocks
    Diff,
}

/// State of a fold
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FoldState {
    /// Fold is open (content visible)
    Open,
    /// Fold is closed (content hidden)
    Closed,
}

/// A fold in the buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fold {
    /// Start line (1-indexed)
    pub start: LineNr,
    /// End line (1-indexed, inclusive)
    pub end: LineNr,
    /// Fold level (1 = top level)
    pub level: usize,
    /// Current state
    pub state: FoldState,
    /// Nested folds
    pub nested: Vec<Fold>,
}

impl Fold {
    /// Get the number of lines in this fold
    pub fn line_count(&self) -> usize {
        self.end.0 - self.start.0 + 1
    }

    /// Check if a line is within this fold
    pub fn contains(&self, line: LineNr) -> bool {
        line.0 >= self.start.0 && line.0 <= self.end.0
    }
}

// ============================================================================
// Fold Manager Trait
// ============================================================================

/// Manages folds for a buffer
pub trait FoldManager {
    /// Get the fold method
    fn method(&self) -> FoldMethod;

    /// Set the fold method
    fn set_method(&mut self, method: FoldMethod);

    /// Create a fold (manual method)
    fn create(&mut self, start: LineNr, end: LineNr) -> VimResult<()>;

    /// Delete a fold at a line
    fn delete(&mut self, line: LineNr) -> VimResult<()>;

    /// Delete all folds
    fn delete_all(&mut self);

    /// Open a fold at a line
    fn open(&mut self, line: LineNr) -> VimResult<()>;

    /// Close a fold at a line
    fn close(&mut self, line: LineNr) -> VimResult<()>;

    /// Toggle fold at a line
    fn toggle(&mut self, line: LineNr) -> VimResult<()>;

    /// Open all folds
    fn open_all(&mut self);

    /// Close all folds
    fn close_all(&mut self);

    /// Get fold at a line
    fn get(&self, line: LineNr) -> Option<&Fold>;

    /// Get fold level at a line
    fn level(&self, line: LineNr) -> usize;

    /// Check if a line is folded (inside a closed fold)
    fn is_folded(&self, line: LineNr) -> bool;

    /// Get the fold text (displayed when closed)
    fn fold_text(&self, fold: &Fold) -> String;

    /// Recompute folds (after buffer change or method change)
    fn recompute(&mut self);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_contains() {
        let fold = Fold {
            start: LineNr(5),
            end: LineNr(10),
            level: 1,
            state: FoldState::Closed,
            nested: vec![],
        };

        assert!(fold.contains(LineNr(5)));
        assert!(fold.contains(LineNr(7)));
        assert!(fold.contains(LineNr(10)));
        assert!(!fold.contains(LineNr(4)));
        assert!(!fold.contains(LineNr(11)));
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Fold Behavioral Tests
        //!
        //! ## Key Quirks
        //!
        //! 1. **Nested folds**: Folds can contain other folds.
        //!
        //! 2. **Fold levels**: 'foldlevel' option controls which folds are open.
        //!
        //! 3. **Fold markers**: Default markers are {{{ and }}}.
        //!
        //! 4. **Fold text**: Customizable via 'foldtext' option.
    }
}
