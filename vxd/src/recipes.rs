//! Editing recipe helpers (usr_12).
//!
//! These are small, focused transformations used in common tasks.

use crate::types::VimResult;

/// Replace the first occurrence of `from` with `to` in the given line.
pub fn replace_word(line: &str, from: &str, to: &str) -> String {
    if let Some(pos) = line.find(from) {
        let mut out = String::new();
        out.push_str(&line[..pos]);
        out.push_str(to);
        out.push_str(&line[pos + from.len()..]);
        out
    } else {
        line.to_string()
    }
}

/// Swap "Last, First" into "First Last".
pub fn swap_last_first(line: &str) -> String {
    if let Some((last, first)) = line.split_once(',') {
        let first = first.trim();
        let last = last.trim();
        if first.is_empty() || last.is_empty() {
            return line.to_string();
        }
        format!("{} {}", first, last)
    } else {
        line.to_string()
    }
}

/// Sort lines lexicographically.
pub fn sort_lines(lines: &[String]) -> Vec<String> {
    let mut out = lines.to_vec();
    out.sort();
    out
}

/// Reverse the order of lines.
pub fn reverse_lines(lines: &[String]) -> Vec<String> {
    let mut out = lines.to_vec();
    out.reverse();
    out
}

/// Count words in text.
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Trim trailing blanks on each line.
pub fn trim_trailing_blanks(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .map(|line| line.trim_end_matches(|c| c == ' ' || c == '\t').to_string())
        .collect()
}

/// Find lines containing the given word, returning 1-based line numbers.
pub fn find_word_usage(lines: &[String], word: &str) -> Vec<usize> {
    if word.is_empty() {
        return Vec::new();
    }
    lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| {
            if line.contains(word) {
                Some(idx + 1)
            } else {
                None
            }
        })
        .collect()
}

/// Stub for man page lookup (returns the command name).
pub fn man_page_target(command: &str) -> VimResult<String> {
    if command.trim().is_empty() {
        return Err(crate::types::VimError::Error(1, "Empty command".to_string()));
    }
    Ok(command.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_word() {
        assert_eq!(replace_word("foo bar", "foo", "baz"), "baz bar");
        assert_eq!(replace_word("foo bar", "nope", "baz"), "foo bar");
    }

    #[test]
    fn test_swap_last_first() {
        assert_eq!(swap_last_first("Doe, Jane"), "Jane Doe");
        assert_eq!(swap_last_first("Doe Jane"), "Doe Jane");
    }

    #[test]
    fn test_sort_lines() {
        let lines = vec!["b".to_string(), "a".to_string()];
        assert_eq!(sort_lines(&lines), vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn test_reverse_lines() {
        let lines = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(
            reverse_lines(&lines),
            vec!["c".to_string(), "b".to_string(), "a".to_string()]
        );
    }

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("one two  three"), 3);
    }

    #[test]
    fn test_trim_trailing_blanks() {
        let lines = vec!["a  ".to_string(), "b\t".to_string(), "c".to_string()];
        assert_eq!(
            trim_trailing_blanks(&lines),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_find_word_usage() {
        let lines = vec!["foo".to_string(), "bar foo".to_string(), "baz".to_string()];
        assert_eq!(find_word_usage(&lines, "foo"), vec![1, 2]);
        assert!(find_word_usage(&lines, "").is_empty());
    }

    #[test]
    fn test_man_page_target() {
        assert_eq!(man_page_target("ls").unwrap(), "ls");
        assert!(man_page_target("").is_err());
    }
}
