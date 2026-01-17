//! Movement commands (w, b, e, gg, G, etc).
//!
//! Motions are commands that move the cursor. They can be used standalone
//! or as the target of an operator (e.g., `dw` = delete word).
//!
//! # Key Behavioral Contracts
//!
//! - Motions have a type: characterwise, linewise, or blockwise
//! - Motions are inclusive or exclusive
//! - Motions respect count prefixes
//! - Some motions wrap across lines, others don't

use crate::cursor::CursorPosition;
use crate::types::*;

// ============================================================================
// Motion Result
// ============================================================================

/// The result of executing a motion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotionResult {
    /// New cursor position after motion
    pub position: CursorPosition,
    /// Type of motion (affects how operators work)
    pub motion_type: MotionType,
    /// Whether the motion is inclusive (includes the end position)
    pub inclusive: MotionInclusivity,
    /// Whether the motion failed (e.g., already at boundary)
    pub failed: bool,
}

impl MotionResult {
    /// Create a successful motion result
    pub fn success(
        position: CursorPosition,
        motion_type: MotionType,
        inclusive: MotionInclusivity,
    ) -> Self {
        MotionResult {
            position,
            motion_type,
            inclusive,
            failed: false,
        }
    }

    /// Create a failed motion result (cursor doesn't move)
    pub fn failed(position: CursorPosition) -> Self {
        MotionResult {
            position,
            motion_type: MotionType::Characterwise,
            inclusive: MotionInclusivity::Exclusive,
            failed: true,
        }
    }
}

// ============================================================================
// Motion Types
// ============================================================================

/// Categories of motions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MotionKind {
    /// Left-right motions (h, l, 0, ^, $, f, F, t, T, etc.)
    LeftRight,
    /// Up-down motions (j, k, +, -, G, gg, etc.)
    UpDown,
    /// Word motions (w, W, b, B, e, E, ge, gE)
    Word,
    /// Text object motions (sentence, paragraph)
    TextObject,
    /// Search motions (/, ?, n, N, *, #)
    Search,
    /// Mark motions (', `, g', g`)
    Mark,
    /// Various other motions (%, H, M, L)
    Various,
}

/// Word motion variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WordMotion {
    /// `w` - word forward
    WordForward,
    /// `W` - WORD forward
    WORDForward,
    /// `b` - word backward
    WordBackward,
    /// `B` - WORD backward
    WORDBackward,
    /// `e` - end of word forward
    EndForward,
    /// `E` - end of WORD forward
    EndWORDForward,
    /// `ge` - end of word backward
    EndBackward,
    /// `gE` - end of WORD backward
    EndWORDBackward,
}

/// Character find motions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharFindMotion {
    /// `f{char}` - find char forward (inclusive)
    FindForward(char),
    /// `F{char}` - find char backward (exclusive)
    FindBackward(char),
    /// `t{char}` - till char forward (inclusive)
    TillForward(char),
    /// `T{char}` - till char backward (exclusive)
    TillBackward(char),
    /// `;` - repeat last f/F/t/T
    RepeatForward,
    /// `,` - repeat last f/F/t/T in opposite direction
    RepeatBackward,
}

/// Line position motions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinePositionMotion {
    /// `0` - first column
    FirstColumn,
    /// `^` - first non-blank
    FirstNonBlank,
    /// `$` - end of line
    EndOfLine,
    /// `g0` - first screen column
    FirstScreenColumn,
    /// `g^` - first non-blank screen column
    FirstNonBlankScreen,
    /// `g$` - end of screen line
    EndOfScreenLine,
    /// `gm` - middle of screen line
    MiddleOfScreenLine,
    /// `gM` - middle of text line
    MiddleOfTextLine,
    /// `|` - to screen column (with count)
    ToColumn(usize),
}

/// Vertical line motions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerticalMotion {
    /// `j` - down
    Down,
    /// `k` - up
    Up,
    /// `gj` - down screen line
    ScreenDown,
    /// `gk` - up screen line
    ScreenUp,
    /// `+` or `<CR>` - down to first non-blank
    DownFirstNonBlank,
    /// `-` - up to first non-blank
    UpFirstNonBlank,
    /// `_` - down (count-1) to first non-blank
    CurrentFirstNonBlank,
}

/// Document position motions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentMotion {
    /// `gg` - go to line (default: first)
    GotoLine(Option<usize>),
    /// `G` - go to line (default: last)
    GotoLastLine(Option<usize>),
    /// `H` - top of window
    WindowTop,
    /// `M` - middle of window
    WindowMiddle,
    /// `L` - bottom of window
    WindowBottom,
    /// `%` with count - percentage through file
    Percentage(usize),
    /// `%` without count - matching bracket
    MatchingBracket,
}

// ============================================================================
// Motion Trait
// ============================================================================

/// Context needed for motion execution
pub struct MotionContext<'a> {
    /// Current line content
    pub line: &'a str,
    /// Total number of lines in buffer
    pub line_count: usize,
    /// Current cursor position
    pub cursor: CursorPosition,
    /// Count prefix (1 if not specified)
    pub count: usize,
    /// Tabstop setting (for screen column calculations)
    pub tabstop: usize,
    /// Whether virtualedit is enabled
    pub virtualedit: bool,
    /// Last character find motion (for ; and ,)
    pub last_char_find: Option<CharFindMotion>,
}

/// Trait for executing motions
pub trait Motion {
    /// Execute a word motion
    fn word_motion(
        &self,
        motion: WordMotion,
        ctx: &MotionContext,
        get_line: impl Fn(LineNr) -> Option<String>,
    ) -> MotionResult;

    /// Execute a character find motion
    fn char_find_motion(&self, motion: CharFindMotion, ctx: &MotionContext) -> MotionResult;

    /// Execute a line position motion
    fn line_position_motion(&self, motion: LinePositionMotion, ctx: &MotionContext)
        -> MotionResult;

    /// Execute a vertical motion
    fn vertical_motion(
        &self,
        motion: VerticalMotion,
        ctx: &MotionContext,
        get_line: impl Fn(LineNr) -> Option<String>,
    ) -> MotionResult;

    /// Execute a document motion
    fn document_motion(
        &self,
        motion: DocumentMotion,
        ctx: &MotionContext,
        get_line: impl Fn(LineNr) -> Option<String>,
    ) -> MotionResult;
}

// ============================================================================
// Word Definition
// ============================================================================

/// Character classification for word motions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    /// Whitespace characters
    Whitespace,
    /// Word characters (alphanumeric + underscore)
    Word,
    /// Punctuation/symbols (everything else)
    Punctuation,
}

impl CharClass {
    /// Classify a character for word motions
    pub fn classify(c: char) -> Self {
        if c.is_whitespace() {
            CharClass::Whitespace
        } else if c.is_alphanumeric() || c == '_' {
            CharClass::Word
        } else {
            CharClass::Punctuation
        }
    }

    /// Classify for WORD motions (only whitespace vs non-whitespace)
    pub fn classify_word(c: char) -> Self {
        if c.is_whitespace() {
            CharClass::Whitespace
        } else {
            CharClass::Word
        }
    }
}

// ============================================================================
// Motion Defaults
// ============================================================================

/// Default motion type for various motions
pub fn default_motion_type(kind: MotionKind) -> MotionType {
    match kind {
        MotionKind::LeftRight => MotionType::Characterwise,
        MotionKind::UpDown => MotionType::Linewise,
        MotionKind::Word => MotionType::Characterwise,
        MotionKind::TextObject => MotionType::Characterwise,
        MotionKind::Search => MotionType::Characterwise,
        MotionKind::Mark => MotionType::Characterwise, // ` is char, ' is line
        MotionKind::Various => MotionType::Characterwise,
    }
}

/// Default inclusivity for various motions
pub fn default_inclusivity(motion: &WordMotion) -> MotionInclusivity {
    match motion {
        // Forward motions to start of word are exclusive
        WordMotion::WordForward | WordMotion::WORDForward => MotionInclusivity::Exclusive,
        // Backward motions are exclusive
        WordMotion::WordBackward | WordMotion::WORDBackward => MotionInclusivity::Exclusive,
        // End-of-word motions are inclusive
        WordMotion::EndForward
        | WordMotion::EndWORDForward
        | WordMotion::EndBackward
        | WordMotion::EndWORDBackward => MotionInclusivity::Inclusive,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_classification() {
        assert_eq!(CharClass::classify(' '), CharClass::Whitespace);
        assert_eq!(CharClass::classify('\t'), CharClass::Whitespace);
        assert_eq!(CharClass::classify('a'), CharClass::Word);
        assert_eq!(CharClass::classify('Z'), CharClass::Word);
        assert_eq!(CharClass::classify('_'), CharClass::Word);
        assert_eq!(CharClass::classify('5'), CharClass::Word);
        assert_eq!(CharClass::classify('.'), CharClass::Punctuation);
        assert_eq!(CharClass::classify('('), CharClass::Punctuation);
    }

    #[test]
    fn test_word_classification() {
        // WORD only distinguishes whitespace
        assert_eq!(CharClass::classify_word(' '), CharClass::Whitespace);
        assert_eq!(CharClass::classify_word('.'), CharClass::Word);
        assert_eq!(CharClass::classify_word('a'), CharClass::Word);
    }

    /// Behavioral tests for motion implementations
    pub trait MotionBehaviorTests: Motion + Sized {
        // ====================================================================
        // Word Motion Tests
        // ====================================================================

        /// Test: `w` moves to start of next word
        fn test_w_basic(&self);

        /// Test: `w` at end of line moves to next line
        fn test_w_crosses_lines(&self);

        /// Test: `w` treats punctuation as separate words
        fn test_w_punctuation_boundary(&self);

        /// Test: `W` ignores punctuation
        fn test_big_w_ignores_punctuation(&self);

        /// Test: `b` moves to start of previous word
        fn test_b_basic(&self);

        /// Test: `e` moves to end of current/next word
        fn test_e_basic(&self);

        // ====================================================================
        // Character Find Tests
        // ====================================================================

        /// Test: `f{char}` finds character forward
        fn test_f_finds_char(&self);

        /// Test: `f{char}` is inclusive
        fn test_f_is_inclusive(&self);

        /// Test: `t{char}` stops before character
        fn test_t_stops_before(&self);

        /// Test: `;` repeats last find
        fn test_semicolon_repeats(&self);

        // ====================================================================
        // Line Position Tests
        // ====================================================================

        /// Test: `0` goes to column 0
        fn test_zero_first_column(&self);

        /// Test: `^` goes to first non-blank
        fn test_caret_first_nonblank(&self);

        /// Test: `$` goes to end of line
        fn test_dollar_end_of_line(&self);

        /// Test: `$` sets curswant to MAXCOL
        fn test_dollar_sets_maxcol(&self);

        // ====================================================================
        // Vertical Motion Tests
        // ====================================================================

        /// Test: `j` moves down preserving column
        fn test_j_preserves_column(&self);

        /// Test: `k` moves up preserving column
        fn test_k_preserves_column(&self);

        /// Test: `j` at last line does nothing
        fn test_j_at_last_line(&self);

        /// Test: `k` at first line does nothing
        fn test_k_at_first_line(&self);

        // ====================================================================
        // Document Motion Tests
        // ====================================================================

        /// Test: `gg` goes to first line
        fn test_gg_first_line(&self);

        /// Test: `G` goes to last line
        fn test_g_last_line(&self);

        /// Test: `{count}G` goes to specific line
        fn test_count_g_specific_line(&self);

        /// Test: `%` finds matching bracket
        fn test_percent_matching_bracket(&self);
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Motion Behavioral Tests
        //!
        //! Tests derived from Neovim's test suite:
        //! - test/functional/legacy/025_jump_spec.lua
        //! - test/functional/legacy/056_word_motion_spec.lua
        //!
        //! ## Key Quirks
        //!
        //! 1. **Word vs WORD**: `w` stops at punctuation, `W` only at whitespace.
        //!
        //! 2. **Inclusivity**: `e` and `f` are inclusive, `w` and `t` are exclusive.
        //!
        //! 3. **Empty lines**: `w` and `b` treat empty lines as word boundaries.
        //!
        //! 4. **Curswant**: `$` sets curswant to MAXCOL so subsequent j/k go to EOL.
        //!
        //! 5. **Count behavior**: `3w` moves 3 words, but `3$` moves down 2 lines then to EOL.
        //!
        //! 6. **Linewise motions**: `j`, `k`, `+`, `-`, `G`, `gg` are linewise.
    }
}
