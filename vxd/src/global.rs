//! Global command implementation (:g, :v).

use crate::types::*;
use crate::buffer::Buffer;
use crate::cursor::CursorPosition;
use crate::search::{SearchEngine, SearchOptions};

/// A parsed global command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalCommand {
    /// Pattern to match
    pub pattern: String,
    /// Command string to execute
    pub command: String,
    /// Whether to invert the match (:v or :g!)
    pub inverse: bool,
}

impl GlobalCommand {
    /// Parse a global command string (e.g. "/pat/cmd")
    /// The leading ":g" or ":v" is assumed to be handled by the caller to set `inverse`.
    pub fn parse(args: &str, inverse: bool) -> VimResult<Self> {
        if args.is_empty() {
            return Err(VimError::ArgumentRequired);
        }

        let mut chars = args.chars();
        let delim = chars.next().unwrap();
        
        if delim.is_alphanumeric() || delim == '"' || delim == '|' {
             return Err(VimError::InvalidPattern("Regular expression missing from global".to_string()));
        }

        let mut pattern = String::new();
        let mut command = String::new();
        let mut escaped = false;
        let mut found_delim = false;

        while let Some(c) = chars.next() {
            if escaped {
                pattern.push(c);
                escaped = false;
            } else if c == '\\' {
                pattern.push(c);
                escaped = true;
            } else if c == delim {
                found_delim = true;
                break;
            } else {
                pattern.push(c);
            }
        }

        // The rest is the command
        if found_delim {
            command = chars.as_str().to_string();
        }

        Ok(GlobalCommand {
            pattern,
            command,
            inverse,
        })
    }

        /// Find all lines matching the global command pattern
        pub fn match_lines<B, S>(
            &self,
            buffer: &B,
            search_engine: &S,
        ) -> VimResult<Vec<LineNr>>
        where
            B: Buffer + ?Sized,
            S: SearchEngine + ?Sized,
        {
            // 1. Compile pattern
            let pattern_str = if self.pattern.is_empty() {
                if let Some(last) = search_engine.last_pattern() {
                    last.pattern.clone()
                } else {
                    return Err(VimError::PatternNotFound("".to_string()));
                }
            } else {
                self.pattern.clone()
            };
    
            let search_pattern = search_engine.compile(&pattern_str, &SearchOptions::default())?;
    
            // 2. Mark lines
            let line_count = buffer.line_count();
            let mut marked_lines = Vec::new();
    
            for i in 1..=line_count {
                let line_nr = LineNr(i);
                let start_pos = CursorPosition::new(line_nr, 0);
                
                // Search on this line specifically
                // We check if the pattern matches anywhere on the line.
                let match_result = search_engine.search(&search_pattern, start_pos, &SearchOptions::default())?;
                
                let mut matched = false;
                if let Some(m) = match_result {
                    if m.start.line == line_nr {
                        matched = true;
                    }
                }
                
                if matched != self.inverse {
                    marked_lines.push(line_nr);
                }
            }
    
            Ok(marked_lines)
        }
    }
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_global() {
        let cmd = GlobalCommand::parse("/foo/d", false).unwrap();
        assert_eq!(cmd.pattern, "foo");
        assert_eq!(cmd.command, "d");
        assert!(!cmd.inverse);
    }

    #[test]
    fn test_parse_global_with_escape() {
        let cmd = GlobalCommand::parse(r"/foo\/bar/p", false).unwrap();
        assert_eq!(cmd.pattern, "foo/bar");
        assert_eq!(cmd.command, "p");
    }

    #[test]
    fn test_parse_vglobal() {
        let cmd = GlobalCommand::parse("/bar/d", true).unwrap();
        assert_eq!(cmd.pattern, "bar");
        assert!(cmd.inverse);
    }
    
    #[test]
    fn test_parse_global_no_command() {
        let cmd = GlobalCommand::parse("/foo/", false).unwrap();
        assert_eq!(cmd.pattern, "foo");
        assert_eq!(cmd.command, "");
    }
}
