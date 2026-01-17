//! Modeline parsing for per-file option settings.
//!
//! This module parses Vim modelines of the form:
//!   "vim:set {option}={value} ... :"
//! It extracts option assignments for higher-level application.

use crate::options::OptionValue;

/// Parsed modeline setting (name/value).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelineSetting {
    /// Option name.
    pub name: String,
    /// Parsed option value.
    pub value: OptionValue,
}

/// Parse a single line for a Vim modeline.
///
/// Returns None if no modeline is found.
pub fn parse_modeline_line(line: &str) -> Option<Vec<ModelineSetting>> {
    let marker = "vim:";
    let mut start = None;
    let mut search = line;
    let mut offset = 0usize;

    while let Some(pos) = search.find(marker) {
        let absolute = offset + pos;
        let ok_prefix = absolute == 0 || line[..absolute].chars().last().map(|c| c.is_whitespace()).unwrap_or(false);
        if ok_prefix {
            start = Some(absolute + marker.len());
            break;
        }
        offset = absolute + marker.len();
        search = &line[offset..];
    }

    let start = start?;
    let mut rest = line[start..].trim_start();
    if !rest.starts_with("set") {
        return None;
    }
    rest = rest["set".len()..].trim_start();

    let mut cmd = String::new();
    let mut chars = rest.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == ':' {
            break;
        }
        if ch == '\\' {
            if let Some(next) = chars.next() {
                if next == ':' {
                    cmd.push(':');
                } else {
                    cmd.push('\\');
                    cmd.push(next);
                }
            } else {
                cmd.push('\\');
            }
        } else {
            cmd.push(ch);
        }
    }

    let mut settings = Vec::new();
    for token in cmd.split_whitespace() {
        if let Some((name, value)) = token.split_once('=') {
            settings.push(ModelineSetting {
                name: name.to_string(),
                value: parse_option_value(value),
            });
        } else if token.starts_with("no") && token.len() > 2 {
            settings.push(ModelineSetting {
                name: token[2..].to_string(),
                value: OptionValue::Boolean(false),
            });
        } else {
            settings.push(ModelineSetting {
                name: token.to_string(),
                value: OptionValue::Boolean(true),
            });
        }
    }

    if settings.is_empty() {
        None
    } else {
        Some(settings)
    }
}

/// Extract modelines from the first and last `count` lines of a file.
pub fn extract_modelines(lines: &[String], count: usize) -> Vec<ModelineSetting> {
    if count == 0 || lines.is_empty() {
        return Vec::new();
    }

    let mut settings = Vec::new();
    let len = lines.len();
    let head = count.min(len);
    let tail_start = len.saturating_sub(count);

    for line in &lines[..head] {
        if let Some(found) = parse_modeline_line(line) {
            settings.extend(found);
        }
    }

    for line in &lines[tail_start..] {
        if let Some(found) = parse_modeline_line(line) {
            settings.extend(found);
        }
    }

    settings
}

fn parse_option_value(value: &str) -> OptionValue {
    if let Ok(num) = value.parse::<i64>() {
        OptionValue::Number(num)
    } else {
        OptionValue::String(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modeline_simple() {
        let line = "/* vim:set shiftwidth=4: */";
        let settings = parse_modeline_line(line).unwrap();
        assert_eq!(settings.len(), 1);
        assert_eq!(settings[0].name, "shiftwidth");
        assert_eq!(settings[0].value, OptionValue::Number(4));
    }

    #[test]
    fn test_parse_modeline_multiple_options() {
        let line = "// vim:set ts=4 sw=2:";
        let settings = parse_modeline_line(line).unwrap();
        assert_eq!(settings.len(), 2);
        assert_eq!(settings[0].name, "ts");
        assert_eq!(settings[0].value, OptionValue::Number(4));
        assert_eq!(settings[1].name, "sw");
        assert_eq!(settings[1].value, OptionValue::Number(2));
    }

    #[test]
    fn test_parse_modeline_requires_whitespace_or_start() {
        let line = "gvim:set ts=4:";
        assert!(parse_modeline_line(line).is_none());
    }

    #[test]
    fn test_parse_modeline_escaped_colon() {
        let line = "// vim:set dir=c\\:\\tmp:";
        let settings = parse_modeline_line(line).unwrap();
        assert_eq!(settings.len(), 1);
        assert_eq!(settings[0].name, "dir");
        assert_eq!(settings[0].value, OptionValue::String("c:\\tmp".to_string()));
    }

    #[test]
    fn test_parse_modeline_boolean_options() {
        let line = "vim:set number nonumber norelativenumber:";
        let settings = parse_modeline_line(line).unwrap();
        assert_eq!(settings.len(), 3);
        assert_eq!(settings[0].value, OptionValue::Boolean(true));
        assert_eq!(settings[1].value, OptionValue::Boolean(false));
        assert_eq!(settings[2].value, OptionValue::Boolean(false));
    }

    #[test]
    fn test_extract_modelines_head_and_tail() {
        let lines = vec![
            "vim:set ts=4:".to_string(),
            "middle".to_string(),
            "vim:set sw=2:".to_string(),
        ];

        let settings = extract_modelines(&lines, 1);
        assert_eq!(settings.len(), 2);
        assert_eq!(settings[0].name, "ts");
        assert_eq!(settings[1].name, "sw");
    }
}
