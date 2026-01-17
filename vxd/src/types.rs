//! Core types used across all Vim features.
//!
//! These types represent the fundamental data model of Vim, independent
//! of any specific feature implementation.

use std::fmt;

// ============================================================================
// Position Types
// ============================================================================

/// A 1-indexed line number (Vim uses 1-based indexing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LineNr(pub usize);

impl LineNr {
    /// The first line (line 1)
    pub const FIRST: LineNr = LineNr(1);

    /// Create a new line number. Panics if n == 0.
    pub fn new(n: usize) -> Self {
        assert!(n > 0, "Line numbers are 1-indexed");
        LineNr(n)
    }

    /// Convert to 0-indexed for internal array access
    pub fn to_zero_indexed(self) -> usize {
        self.0 - 1
    }

    /// Create from 0-indexed value
    pub fn from_zero_indexed(idx: usize) -> Self {
        LineNr(idx + 1)
    }
}

impl fmt::Display for LineNr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A 1-indexed column number (Vim uses 1-based indexing)
/// Note: In Vim, column is byte-offset based, not character based
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ColNr(pub usize);

impl ColNr {
    /// The first column (column 1)
    pub const FIRST: ColNr = ColNr(1);

    /// Create a new column number. Panics if n == 0.
    pub fn new(n: usize) -> Self {
        assert!(n > 0, "Column numbers are 1-indexed");
        ColNr(n)
    }

    /// Convert to 0-indexed for internal array access
    pub fn to_zero_indexed(self) -> usize {
        self.0 - 1
    }

    /// Create from 0-indexed value
    pub fn from_zero_indexed(idx: usize) -> Self {
        ColNr(idx + 1)
    }
}

impl fmt::Display for ColNr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A position in a buffer (line, column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: LineNr,
    /// Column number (1-indexed, byte offset)
    pub col: ColNr,
}

impl Position {
    /// Create a new position
    pub fn new(line: LineNr, col: ColNr) -> Self {
        Position { line, col }
    }

    /// Create position from raw 1-indexed values
    pub fn from_1indexed(line: usize, col: usize) -> Self {
        Position {
            line: LineNr::new(line),
            col: ColNr::new(col),
        }
    }

    /// The origin position (1, 1)
    pub const ORIGIN: Position = Position {
        line: LineNr(1),
        col: ColNr(1),
    };
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// Virtual column number (accounts for tabs, wide characters)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct VirtColNr(pub usize);

// ============================================================================
// Range Types
// ============================================================================

/// A range of lines (inclusive on both ends, Vim-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineRange {
    /// Start line (inclusive)
    pub start: LineNr,
    /// End line (inclusive)
    pub end: LineNr,
}

impl LineRange {
    /// Create a new line range
    pub fn new(start: LineNr, end: LineNr) -> Self {
        LineRange { start, end }
    }

    /// Create a single-line range
    pub fn single(line: LineNr) -> Self {
        LineRange {
            start: line,
            end: line,
        }
    }

    /// Number of lines in the range
    pub fn len(&self) -> usize {
        if self.end.0 >= self.start.0 {
            self.end.0 - self.start.0 + 1
        } else {
            0
        }
    }

    /// Check if range is empty
    pub fn is_empty(&self) -> bool {
        self.end.0 < self.start.0
    }
}

/// A range between two positions (for characterwise operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CharRange {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (inclusive in Vim's model)
    pub end: Position,
}

impl CharRange {
    /// Create a new character range
    pub fn new(start: Position, end: Position) -> Self {
        CharRange { start, end }
    }
}

// ============================================================================
// Buffer Identification
// ============================================================================

/// Unique identifier for a buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId(pub usize);

/// Unique identifier for a window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub usize);

/// Unique identifier for a tab page
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub usize);

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during Vim operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VimError {
    /// Invalid line number
    InvalidLine(LineNr),
    /// Invalid column number
    InvalidColumn(ColNr),
    /// Invalid position
    InvalidPosition(Position),
    /// Invalid range
    InvalidRange(String),
    /// Buffer not found
    BufferNotFound(BufferId),
    /// Window not found
    WindowNotFound(WindowId),
    /// Tab not found
    TabNotFound(TabId),
    /// Invalid register name
    InvalidRegister(char),
    /// Invalid mark name
    InvalidMark(char),
    /// Mark not set
    MarkNotSet(char),
    /// Pattern not found
    PatternNotFound(String),
    /// Invalid pattern/regex
    InvalidPattern(String),
    /// Command failed
    CommandFailed(String),
    /// File not found
    FileNotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Read-only buffer/option
    ReadOnly(String),
    /// Operation not allowed in current mode
    NotAllowedInMode(String),
    /// Argument required
    ArgumentRequired,
    /// Trailing characters after command
    TrailingCharacters,
    /// Not an editor command
    NotEditorCommand(String),
    /// Generic error message (E followed by number)
    Error(u32, String),
}

impl fmt::Display for VimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VimError::InvalidLine(l) => write!(f, "E16: Invalid line number: {}", l),
            VimError::InvalidColumn(c) => write!(f, "E964: Invalid column number: {}", c),
            VimError::InvalidPosition(p) => write!(f, "Invalid position: {}", p),
            VimError::InvalidRange(r) => write!(f, "E16: Invalid range: {}", r),
            VimError::BufferNotFound(id) => write!(f, "E86: Buffer {} does not exist", id.0),
            VimError::WindowNotFound(id) => write!(f, "E957: Invalid window number: {}", id.0),
            VimError::TabNotFound(id) => write!(f, "E16: Invalid tab number: {}", id.0),
            VimError::InvalidRegister(c) => write!(f, "E354: Invalid register name: '{}'", c),
            VimError::InvalidMark(c) => write!(f, "E78: Unknown mark: {}", c),
            VimError::MarkNotSet(c) => write!(f, "E20: Mark not set: {}", c),
            VimError::PatternNotFound(p) => write!(f, "E486: Pattern not found: {}", p),
            VimError::InvalidPattern(p) => write!(f, "E383: Invalid search string: {}", p),
            VimError::CommandFailed(s) => write!(f, "E492: {}", s),
            VimError::FileNotFound(s) => write!(f, "E484: Can't open file {}", s),
            VimError::PermissionDenied(s) => write!(f, "E212: Can't open file for writing: {}", s),
            VimError::ReadOnly(s) => write!(f, "E45: 'readonly' option is set: {}", s),
            VimError::NotAllowedInMode(s) => write!(f, "E523: Not allowed here: {}", s),
            VimError::ArgumentRequired => write!(f, "E471: Argument required"),
            VimError::TrailingCharacters => write!(f, "E488: Trailing characters"),
            VimError::NotEditorCommand(s) => write!(f, "E492: Not an editor command: {}", s),
            VimError::Error(n, s) => write!(f, "E{}: {}", n, s),
        }
    }
}

impl std::error::Error for VimError {}

/// Result type for Vim operations
pub type VimResult<T> = Result<T, VimError>;

// ============================================================================
// Direction and Motion Types
// ============================================================================

/// Direction of movement or search
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    /// Forward (down, right, towards end of file)
    #[default]
    Forward,
    /// Backward (up, left, towards beginning of file)
    Backward,
}

impl Direction {
    /// Reverse the direction
    pub fn reverse(self) -> Self {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }
}

/// Type of motion (characterwise, linewise, blockwise)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MotionType {
    /// Character-wise motion (e.g., `w`, `e`, `f`)
    Characterwise,
    /// Line-wise motion (e.g., `j`, `k`, `dd`)
    Linewise,
    /// Block-wise motion (visual block mode)
    Blockwise,
}

/// Whether a motion is inclusive or exclusive
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MotionInclusivity {
    /// Motion includes the final character
    Inclusive,
    /// Motion excludes the final character
    Exclusive,
}

// ============================================================================
// Count and Repeat
// ============================================================================

/// A count value (the number prefix before commands, e.g., `3dd`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Count(pub Option<usize>);

impl Count {
    /// No count specified
    pub const NONE: Count = Count(None);

    /// Create a count with a specific value
    pub fn new(n: usize) -> Self {
        Count(Some(n))
    }

    /// Get the count value, defaulting to 1 if not specified
    pub fn value_or_default(self) -> usize {
        self.0.unwrap_or(1)
    }

    /// Get the count value, defaulting to the given value if not specified
    pub fn value_or(self, default: usize) -> usize {
        self.0.unwrap_or(default)
    }

    /// Check if a count was explicitly specified
    pub fn is_specified(self) -> bool {
        self.0.is_some()
    }
}

impl Default for Count {
    fn default() -> Self {
        Count::NONE
    }
}
