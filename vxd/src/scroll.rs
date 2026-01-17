//! Scrolling helpers for window viewports.

use crate::types::LineNr;

/// Scroll direction for adjusting a window topline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    /// Scroll window down (content moves up).
    Down,
    /// Scroll window up (content moves down).
    Up,
}

/// Compute a new topline when scrolling a window.
pub fn scroll_topline(
    topline: LineNr,
    line_count: usize,
    window_height: usize,
    count: usize,
    direction: ScrollDirection,
) -> LineNr {
    let line_count = line_count.max(1);
    let window_height = window_height.max(1);
    let max_topline = if line_count > window_height {
        line_count - window_height + 1
    } else {
        1
    };
    let current = topline.0.clamp(1, max_topline);
    let new_topline = match direction {
        ScrollDirection::Down => (current + count).min(max_topline),
        ScrollDirection::Up => current.saturating_sub(count).max(1),
    };
    LineNr(new_topline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_down_clamps_to_max() {
        let topline = LineNr(1);
        let new_top = scroll_topline(topline, 100, 20, 200, ScrollDirection::Down);
        assert_eq!(new_top, LineNr(81));
    }

    #[test]
    fn test_scroll_up_clamps_to_one() {
        let topline = LineNr(5);
        let new_top = scroll_topline(topline, 100, 20, 10, ScrollDirection::Up);
        assert_eq!(new_top, LineNr(1));
    }

    #[test]
    fn test_scroll_when_buffer_shorter_than_window() {
        let topline = LineNr(1);
        let new_top = scroll_topline(topline, 5, 20, 3, ScrollDirection::Down);
        assert_eq!(new_top, LineNr(1));
    }
}
