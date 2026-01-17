//! Binary mode handling.
//!
//! This module models simple binary mode transformations for file contents.

/// Convert text to binary mode (strip carriage returns).
pub fn to_binary(text: &str) -> String {
    text.replace('\r', "")
}

/// Convert text from binary mode (ensure trailing newline).
pub fn from_binary(text: &str) -> String {
    if text.ends_with('\n') {
        text.to_string()
    } else {
        let mut out = text.to_string();
        out.push('\n');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_binary_strips_cr() {
        let text = "one\r\ntwo\rthree\n";
        assert_eq!(to_binary(text), "one\ntwothree\n");
    }

    #[test]
    fn test_from_binary_adds_trailing_newline() {
        assert_eq!(from_binary("one\n"), "one\n");
        assert_eq!(from_binary("two"), "two\n");
    }
}
