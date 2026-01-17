//! Cursor position reporting helpers (e.g., Ctrl-G).

use crate::cursor::CursorPosition;
use crate::types::LineNr;

/// Information about the current cursor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionInfo {
    /// Current 1-based line number.
    pub line: LineNr,
    /// Current 1-based column number.
    pub col: usize,
    /// Total number of lines in the buffer.
    pub line_count: usize,
    /// Percentage through the file (1..=100).
    pub percent: usize,
}

/// Compute position information from the cursor and line count.
pub fn position_info(cursor: CursorPosition, line_count: usize) -> PositionInfo {
    let line_count = line_count.max(1);
    let line = LineNr(cursor.line.0.clamp(1, line_count));
    let col = cursor.col.saturating_add(1);
    let percent = line.0.saturating_mul(100) / line_count;

    PositionInfo {
        line,
        col,
        line_count,
        percent: percent.clamp(1, 100),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cursor::CursorPosition;

    #[test]
    fn test_position_info_basic() {
        let cursor = CursorPosition::new(LineNr(5), 3);
        let info = position_info(cursor, 10);
        assert_eq!(info.line, LineNr(5));
        assert_eq!(info.col, 4);
        assert_eq!(info.line_count, 10);
        assert_eq!(info.percent, 50);
    }

    #[test]
    fn test_position_info_single_line_is_100_percent() {
        let cursor = CursorPosition::new(LineNr(1), 0);
        let info = position_info(cursor, 1);
        assert_eq!(info.percent, 100);
    }

    #[test]
    fn test_position_info_clamps_line() {
        let cursor = CursorPosition::new(LineNr(5), 0);
        let info = position_info(cursor, 3);
        assert_eq!(info.line, LineNr(3));
        assert_eq!(info.percent, 100);
    }
}
