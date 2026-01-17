//! Operator commands (d, c, y, etc).
//!
//! Operators are commands that act on a region of text defined by a motion
//! or text object. The formula is: operator + motion = action.
//!
//! # Key Behavioral Contracts
//!
//! - Operators wait for a motion or text object
//! - The region is determined by the motion's start/end and type
//! - Operators respect the motion's inclusivity
//! - Double operator (dd, yy) acts on whole line(s)

use crate::registers::{Register, RegisterContent};
use crate::types::*;

// ============================================================================
// Operator Types
// ============================================================================

/// An operator command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    /// `d` - delete
    Delete,
    /// `c` - change (delete and enter insert mode)
    Change,
    /// `y` - yank (copy)
    Yank,
    /// `>` - indent (shift right)
    Indent,
    /// `<` - dedent (shift left)
    Dedent,
    /// `=` - auto-indent/format
    Format,
    /// `g~` - toggle case
    ToggleCase,
    /// `gu` - lowercase
    Lowercase,
    /// `gU` - uppercase
    Uppercase,
    /// `gq` - format text (wrap)
    FormatText,
    /// `gw` - format text (wrap, cursor stays)
    FormatTextKeepCursor,
    /// `!` - filter through external command
    Filter,
    /// `zf` - create fold
    CreateFold,
    /// `g@` - call 'operatorfunc'
    CallOperatorFunc,
}

impl Operator {
    /// Parse an operator from its key sequence
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "d" => Some(Operator::Delete),
            "c" => Some(Operator::Change),
            "y" => Some(Operator::Yank),
            ">" => Some(Operator::Indent),
            "<" => Some(Operator::Dedent),
            "=" => Some(Operator::Format),
            "g~" => Some(Operator::ToggleCase),
            "gu" => Some(Operator::Lowercase),
            "gU" => Some(Operator::Uppercase),
            "gq" => Some(Operator::FormatText),
            "gw" => Some(Operator::FormatTextKeepCursor),
            "!" => Some(Operator::Filter),
            "zf" => Some(Operator::CreateFold),
            "g@" => Some(Operator::CallOperatorFunc),
            _ => None,
        }
    }

    /// Get the key sequence for this operator
    pub fn key(&self) -> &'static str {
        match self {
            Operator::Delete => "d",
            Operator::Change => "c",
            Operator::Yank => "y",
            Operator::Indent => ">",
            Operator::Dedent => "<",
            Operator::Format => "=",
            Operator::ToggleCase => "g~",
            Operator::Lowercase => "gu",
            Operator::Uppercase => "gU",
            Operator::FormatText => "gq",
            Operator::FormatTextKeepCursor => "gw",
            Operator::Filter => "!",
            Operator::CreateFold => "zf",
            Operator::CallOperatorFunc => "g@",
        }
    }

    /// Check if this operator modifies the buffer
    pub fn modifies_buffer(&self) -> bool {
        match self {
            Operator::Yank | Operator::CreateFold | Operator::CallOperatorFunc => false,
            _ => true,
        }
    }

    /// Check if this operator enters insert mode after
    pub fn enters_insert(&self) -> bool {
        matches!(self, Operator::Change)
    }

    /// Check if this operator stores text in a register
    pub fn uses_register(&self) -> bool {
        matches!(self, Operator::Delete | Operator::Change | Operator::Yank)
    }
}

// ============================================================================
// Operation Region
// ============================================================================

/// Defines the region an operator will act on
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorRegion {
    /// Start position
    pub start: Position,
    /// End position
    pub end: Position,
    /// Type of region (characterwise, linewise, blockwise)
    pub region_type: MotionType,
    /// Whether the end position is inclusive
    pub inclusive: bool,
}

impl OperatorRegion {
    /// Create a characterwise region
    pub fn characterwise(start: Position, end: Position, inclusive: bool) -> Self {
        OperatorRegion {
            start,
            end,
            region_type: MotionType::Characterwise,
            inclusive,
        }
    }

    /// Create a linewise region
    pub fn linewise(start_line: LineNr, end_line: LineNr) -> Self {
        OperatorRegion {
            start: Position::new(start_line, ColNr::FIRST),
            end: Position::new(end_line, ColNr::FIRST),
            region_type: MotionType::Linewise,
            inclusive: true,
        }
    }

    /// Create a blockwise region
    pub fn blockwise(start: Position, end: Position) -> Self {
        OperatorRegion {
            start,
            end,
            region_type: MotionType::Blockwise,
            inclusive: true,
        }
    }

    /// Normalize the region so start <= end
    pub fn normalize(&mut self) {
        if self.start.line > self.end.line
            || (self.start.line == self.end.line && self.start.col > self.end.col)
        {
            std::mem::swap(&mut self.start, &mut self.end);
        }
    }

    /// Get the line range of this region
    pub fn line_range(&self) -> LineRange {
        LineRange::new(self.start.line, self.end.line)
    }
}

// ============================================================================
// Operation Result
// ============================================================================

/// Result of executing an operator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorResult {
    /// Content that was affected (for delete/change/yank)
    pub content: Option<RegisterContent>,
    /// New cursor position after operation
    pub cursor: Position,
    /// Whether to enter insert mode
    pub enter_insert: bool,
    /// Whether the operation succeeded
    pub success: bool,
}

// ============================================================================
// Operator Executor Trait
// ============================================================================

/// Context for operator execution
pub struct OperatorContext {
    /// Target register for yank/delete
    pub register: Register,
    /// Count prefix (multiplied with motion count)
    pub count: Count,
    /// Whether this is a double operator (dd, yy, etc.)
    pub is_double: bool,
}

impl Default for OperatorContext {
    fn default() -> Self {
        OperatorContext {
            register: Register::Unnamed,
            count: Count::NONE,
            is_double: false,
        }
    }
}

/// Trait for executing operators
pub trait OperatorExecutor {
    /// Execute a delete operation
    fn delete(
        &mut self,
        region: OperatorRegion,
        ctx: &OperatorContext,
    ) -> VimResult<OperatorResult>;

    /// Execute a change operation
    fn change(
        &mut self,
        region: OperatorRegion,
        ctx: &OperatorContext,
    ) -> VimResult<OperatorResult>;

    /// Execute a yank operation
    fn yank(&mut self, region: OperatorRegion, ctx: &OperatorContext) -> VimResult<OperatorResult>;

    /// Execute an indent operation
    fn indent(
        &mut self,
        region: OperatorRegion,
        ctx: &OperatorContext,
    ) -> VimResult<OperatorResult>;

    /// Execute a dedent operation
    fn dedent(
        &mut self,
        region: OperatorRegion,
        ctx: &OperatorContext,
    ) -> VimResult<OperatorResult>;

    /// Execute a format operation
    fn format(
        &mut self,
        region: OperatorRegion,
        ctx: &OperatorContext,
    ) -> VimResult<OperatorResult>;

    /// Execute a case toggle operation
    fn toggle_case(
        &mut self,
        region: OperatorRegion,
        ctx: &OperatorContext,
    ) -> VimResult<OperatorResult>;

    /// Execute a lowercase operation
    fn lowercase(
        &mut self,
        region: OperatorRegion,
        ctx: &OperatorContext,
    ) -> VimResult<OperatorResult>;

    /// Execute an uppercase operation
    fn uppercase(
        &mut self,
        region: OperatorRegion,
        ctx: &OperatorContext,
    ) -> VimResult<OperatorResult>;

    /// Execute any operator
    fn execute(
        &mut self,
        op: Operator,
        region: OperatorRegion,
        ctx: &OperatorContext,
    ) -> VimResult<OperatorResult> {
        match op {
            Operator::Delete => self.delete(region, ctx),
            Operator::Change => self.change(region, ctx),
            Operator::Yank => self.yank(region, ctx),
            Operator::Indent => self.indent(region, ctx),
            Operator::Dedent => self.dedent(region, ctx),
            Operator::Format => self.format(region, ctx),
            Operator::ToggleCase => self.toggle_case(region, ctx),
            Operator::Lowercase => self.lowercase(region, ctx),
            Operator::Uppercase => self.uppercase(region, ctx),
            _ => Err(VimError::CommandFailed(format!(
                "Operator {:?} not implemented",
                op
            ))),
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
    fn test_operator_parsing() {
        assert_eq!(Operator::from_key("d"), Some(Operator::Delete));
        assert_eq!(Operator::from_key("c"), Some(Operator::Change));
        assert_eq!(Operator::from_key("y"), Some(Operator::Yank));
        assert_eq!(Operator::from_key("gu"), Some(Operator::Lowercase));
        assert_eq!(Operator::from_key("gU"), Some(Operator::Uppercase));
        assert_eq!(Operator::from_key("x"), None);
    }

    #[test]
    fn test_operator_properties() {
        assert!(Operator::Delete.modifies_buffer());
        assert!(!Operator::Yank.modifies_buffer());
        assert!(Operator::Change.enters_insert());
        assert!(!Operator::Delete.enters_insert());
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Operator Behavioral Tests
        //!
        //! Tests derived from Neovim's test suite:
        //! - test/functional/legacy/094_visual_mode_operators_spec.lua
        //! - test/functional/editor/put_spec.lua
        //!
        //! ## Key Quirks
        //!
        //! 1. **Exclusive adjustment**: If an exclusive motion ends at column 0,
        //!    the region is adjusted to be linewise.
        //!
        //! 2. **Linewise yank**: `yy` and `Y` are always linewise, even in visual
        //!    charwise mode.
        //!
        //! 3. **Change vs delete**: `c` enters insert mode, `d` stays in normal.
        //!
        //! 4. **Register behavior**: Delete stores in "1, yank stores in "0.
        //!
        //! 5. **Dot repeat**: The last operator+motion is remembered for `.`.
    }
}
