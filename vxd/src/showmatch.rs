//! Bracket matching helpers for 'showmatch'.

/// Find the matching bracket on a single line.
pub fn find_matching_bracket(line: &str, col: usize) -> Option<usize> {
    let ch = line.get(col..)?.chars().next()?;
    let (open, close, forward) = match ch {
        '(' => ('(', ')', true),
        '[' => ('[', ']', true),
        '{' => ('{', '}', true),
        ')' => ('(', ')', false),
        ']' => ('[', ']', false),
        '}' => ('{', '}', false),
        _ => return None,
    };

    if forward {
        find_forward(line, col, open, close)
    } else {
        find_backward(line, col, open, close)
    }
}

fn find_forward(line: &str, col: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in line.char_indices() {
        if idx < col {
            continue;
        }
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

fn find_backward(line: &str, col: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in line.char_indices() {
        if idx > col {
            break;
        }
        if ch == close {
            depth += 1;
        } else if ch == open {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_forward() {
        let line = "(a [b {c} d] e)";
        assert_eq!(find_matching_bracket(line, 0), Some(14));
    }

    #[test]
    fn test_match_backward() {
        let line = "(a [b {c} d] e)";
        assert_eq!(find_matching_bracket(line, 14), Some(0));
    }

    #[test]
    fn test_match_none_when_not_on_bracket() {
        let line = "(ab)";
        assert_eq!(find_matching_bracket(line, 1), None);
    }
}
