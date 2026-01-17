//! Moving text within a buffer (e.g., :move).

use crate::types::{LineNr, VimError, VimResult};

/// Move a line range to after a destination line.
///
/// `dest` is 1-based; `0` means before the first line.
pub fn move_line_range(
    lines: &mut Vec<String>,
    start: LineNr,
    end: LineNr,
    dest: usize,
) -> VimResult<()> {
    if lines.is_empty() {
        return Ok(());
    }

    if start.0 == 0 || end.0 == 0 {
        return Err(VimError::InvalidRange("line numbers are 1-based".into()));
    }

    if start.0 > end.0 {
        return Err(VimError::InvalidRange("start after end".into()));
    }

    let len = lines.len();
    let start_idx = start.0 - 1;
    let end_idx = end.0 - 1;
    if end_idx >= len {
        return Err(VimError::InvalidRange("range out of bounds".into()));
    }
    if dest > len {
        return Err(VimError::InvalidRange("destination out of bounds".into()));
    }
    if dest >= start.0 && dest <= end.0 {
        return Ok(());
    }

    let moved: Vec<String> = lines.drain(start_idx..=end_idx).collect();
    let range_len = moved.len();
    let mut dest_adjusted = dest;
    if dest > end.0 {
        dest_adjusted = dest.saturating_sub(range_len);
    }

    let insert_at = dest_adjusted.min(lines.len());
    lines.splice(insert_at..insert_at, moved);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_line_down() {
        let mut lines = vec!["a", "b", "c", "d"]
            .into_iter()
            .map(String::from)
            .collect();
        move_line_range(&mut lines, LineNr(2), LineNr(2), 3).unwrap();
        assert_eq!(lines, vec!["a", "c", "b", "d"]);
    }

    #[test]
    fn test_move_line_up() {
        let mut lines = vec!["a", "b", "c", "d"]
            .into_iter()
            .map(String::from)
            .collect();
        move_line_range(&mut lines, LineNr(3), LineNr(3), 1).unwrap();
        assert_eq!(lines, vec!["a", "c", "b", "d"]);
    }

    #[test]
    fn test_move_to_top() {
        let mut lines = vec!["a", "b", "c", "d"]
            .into_iter()
            .map(String::from)
            .collect();
        move_line_range(&mut lines, LineNr(3), LineNr(4), 0).unwrap();
        assert_eq!(lines, vec!["c", "d", "a", "b"]);
    }

    #[test]
    fn test_move_noop_when_dest_inside_range() {
        let mut lines = vec!["a", "b", "c", "d"]
            .into_iter()
            .map(String::from)
            .collect();
        move_line_range(&mut lines, LineNr(2), LineNr(3), 2).unwrap();
        assert_eq!(lines, vec!["a", "b", "c", "d"]);
    }
}
