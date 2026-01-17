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
// Case Operations
// ============================================================================

/// Case conversion operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseOp {
    /// Toggle ASCII case.
    Toggle,
    /// Force ASCII lowercase.
    Lower,
    /// Force ASCII uppercase.
    Upper,
}

/// Apply a case transformation to the given region.
pub fn apply_case(lines: &mut [String], region: &OperatorRegion, op: CaseOp) -> VimResult<()> {
    if lines.is_empty() {
        return Ok(());
    }

    let mut normalized = region.clone();
    normalized.normalize();
    let start_line = normalized.start.line.0.saturating_sub(1);
    let end_line = normalized.end.line.0.saturating_sub(1);
    if end_line >= lines.len() {
        return Err(VimError::InvalidRange("range out of bounds".into()));
    }

    match normalized.region_type {
        MotionType::Linewise => {
            for line in &mut lines[start_line..=end_line] {
                transform_bytes(line, 0, line.len(), op);
            }
        }
        MotionType::Blockwise => {
            let start_col = normalized.start.col.to_zero_indexed();
            let end_col = normalized.end.col.to_zero_indexed();
            let (col_min, col_max) = if start_col <= end_col {
                (start_col, end_col)
            } else {
                (end_col, start_col)
            };
            for line in &mut lines[start_line..=end_line] {
                let len = line.len();
                if col_min >= len {
                    continue;
                }
                let end = (col_max + 1).min(len);
                transform_bytes(line, col_min, end, op);
            }
        }
        MotionType::Characterwise => {
            let start_col = normalized.start.col.to_zero_indexed();
            let end_col = normalized.end.col.to_zero_indexed();
            for (idx, line) in lines[start_line..=end_line].iter_mut().enumerate() {
                let is_first = idx == 0;
                let is_last = start_line + idx == end_line;
                let line_len = line.len();
                let start = if is_first { start_col.min(line_len) } else { 0 };
                let mut end = if is_last { end_col.min(line_len) } else { line_len };
                if is_last && normalized.inclusive {
                    end = end.saturating_add(1).min(line_len);
                }
                if start >= end {
                    continue;
                }
                transform_bytes(line, start, end, op);
            }
        }
    }

    Ok(())
}

fn transform_bytes(line: &mut String, start: usize, end: usize, op: CaseOp) {
    let mut bytes = line.as_bytes().to_vec();
    let end = end.min(bytes.len());
    for b in &mut bytes[start..end] {
        *b = match op {
            CaseOp::Lower => b.to_ascii_lowercase(),
            CaseOp::Upper => b.to_ascii_uppercase(),
            CaseOp::Toggle => {
                if b.is_ascii_lowercase() {
                    b.to_ascii_uppercase()
                } else if b.is_ascii_uppercase() {
                    b.to_ascii_lowercase()
                } else {
                    *b
                }
            }
        };
    }
    if let Ok(updated) = String::from_utf8(bytes) {
        *line = updated;
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

    #[test]
    fn test_apply_case_charwise() {
        let mut lines = vec!["AbCdEf".to_string()];
        let region = OperatorRegion::characterwise(
            Position::new(LineNr(1), ColNr::from_zero_indexed(1)),
            Position::new(LineNr(1), ColNr::from_zero_indexed(4)),
            true,
        );
        apply_case(&mut lines, &region, CaseOp::Lower).unwrap();
        assert_eq!(lines[0], "Abcdef");
    }

    #[test]
    fn test_apply_case_linewise() {
        let mut lines = vec!["Hello".to_string(), "World".to_string()];
        let region = OperatorRegion::linewise(LineNr(1), LineNr(2));
        apply_case(&mut lines, &region, CaseOp::Upper).unwrap();
        assert_eq!(lines, vec!["HELLO", "WORLD"]);
    }

    #[test]
    fn test_apply_case_toggle_multiline() {
        let mut lines = vec!["AbC".to_string(), "dEf".to_string()];
        let region = OperatorRegion::characterwise(
            Position::new(LineNr(1), ColNr::from_zero_indexed(0)),
            Position::new(LineNr(2), ColNr::from_zero_indexed(2)),
            true,
        );
        apply_case(&mut lines, &region, CaseOp::Toggle).unwrap();
        assert_eq!(lines, vec!["aBc", "DeF"]);
    }

    #[test]
    fn test_apply_case_blockwise() {
        let mut lines = vec!["abCD".to_string(), "efGH".to_string()];
        let region = OperatorRegion::blockwise(
            Position::new(LineNr(1), ColNr::from_zero_indexed(2)),
            Position::new(LineNr(2), ColNr::from_zero_indexed(3)),
        );
        apply_case(&mut lines, &region, CaseOp::Lower).unwrap();
        assert_eq!(lines, vec!["abcd", "efgh"]);
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
