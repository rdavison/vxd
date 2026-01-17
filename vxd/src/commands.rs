//! Ex command system.
//!
//! Ex commands are the colon commands in Vim (`:w`, `:q`, `:s`, etc.).
//! They have their own syntax for ranges, arguments, and flags.

use crate::types::*;

// ============================================================================
// Command Range
// ============================================================================

/// A line specifier in a range
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineSpec {
    /// Absolute line number
    Absolute(LineNr),
    /// Current line (.)
    Current,
    /// Last line ($)
    Last,
    /// Mark position ('a)
    Mark(char),
    /// Search forward (/pattern/)
    SearchForward(String),
    /// Search backward (?pattern?)
    SearchBackward(String),
    /// Relative offset (+n or -n)
    Relative(i32),
    /// Visual selection start ('<)
    VisualStart,
    /// Visual selection end ('>)
    VisualEnd,
}

/// A command range
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandRange {
    /// Start line specification
    pub start: Option<LineSpec>,
    /// End line specification (if range)
    pub end: Option<LineSpec>,
    /// Whether this is the whole file (%)
    pub whole_file: bool,
}

impl CommandRange {
    /// Create a range for the whole file
    pub fn whole_file() -> Self {
        CommandRange {
            start: None,
            end: None,
            whole_file: true,
        }
    }

    /// Create a range for the current line
    pub fn current_line() -> Self {
        CommandRange {
            start: Some(LineSpec::Current),
            end: None,
            whole_file: false,
        }
    }

    /// Create a range from absolute line numbers
    pub fn lines(start: LineNr, end: LineNr) -> Self {
        CommandRange {
            start: Some(LineSpec::Absolute(start)),
            end: Some(LineSpec::Absolute(end)),
            whole_file: false,
        }
    }
}

// ============================================================================
// Command Types
// ============================================================================

/// A parsed ex command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExCommand {
    /// Command name
    pub name: String,
    /// Range specification
    pub range: CommandRange,
    /// Whether ! was appended (force)
    pub bang: bool,
    /// Command arguments
    pub args: String,
    /// Count (if applicable)
    pub count: Option<usize>,
    /// Register (if applicable)
    pub register: Option<char>,
}

/// Parse a leading command range from a command line.
pub fn parse_command_range(input: &str) -> VimResult<(CommandRange, &str)> {
    let mut idx = 0usize;
    skip_whitespace(input, &mut idx);
    if idx >= input.len() {
        return Ok((CommandRange::default(), input));
    }

    if input[idx..].starts_with('%') {
        idx += 1;
        return Ok((CommandRange::whole_file(), &input[idx..]));
    }

    let start = match parse_line_spec(input, &mut idx) {
        Some(spec) => spec,
        None => return Ok((CommandRange::default(), input)),
    };

    skip_whitespace(input, &mut idx);
    let end = if idx < input.len() && input[idx..].starts_with(',') {
        idx += 1;
        skip_whitespace(input, &mut idx);
        parse_line_spec(input, &mut idx)
            .ok_or_else(|| VimError::InvalidRange("missing range end".into()))?
    } else {
        return Ok((
            CommandRange {
                start: Some(start),
                end: None,
                whole_file: false,
            },
            &input[idx..],
        ));
    };

    Ok((
        CommandRange {
            start: Some(start),
            end: Some(end),
            whole_file: false,
        },
        &input[idx..],
    ))
}

fn parse_line_spec(input: &str, idx: &mut usize) -> Option<LineSpec> {
    if *idx >= input.len() {
        return None;
    }
    let rest = &input[*idx..];
    let mut chars = rest.chars();
    match chars.next()? {
        '.' => {
            *idx += 1;
            Some(LineSpec::Current)
        }
        '$' => {
            *idx += 1;
            Some(LineSpec::Last)
        }
        ch if ch.is_ascii_digit() => {
            let mut end = *idx + 1;
            while end < input.len() && input.as_bytes()[end].is_ascii_digit() {
                end += 1;
            }
            let number = input[*idx..end].parse::<usize>().ok()?;
            *idx = end;
            Some(LineSpec::Absolute(LineNr(number)))
        }
        _ => None,
    }
}

fn skip_whitespace(input: &str, idx: &mut usize) {
    while *idx < input.len() && input.as_bytes()[*idx].is_ascii_whitespace() {
        *idx += 1;
    }
}

/// Result of command execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    /// Whether command succeeded
    pub success: bool,
    /// Output message (if any)
    pub message: Option<String>,
    /// Error (if failed)
    pub error: Option<VimError>,
}

impl CommandResult {
    /// Create a success result
    pub fn success() -> Self {
        CommandResult {
            success: true,
            message: None,
            error: None,
        }
    }

    /// Create a success result with message
    pub fn with_message(message: impl Into<String>) -> Self {
        CommandResult {
            success: true,
            message: Some(message.into()),
            error: None,
        }
    }

    /// Create an error result
    pub fn error(err: VimError) -> Self {
        CommandResult {
            success: false,
            message: None,
            error: Some(err),
        }
    }
}

// ============================================================================
// Command Definition
// ============================================================================

/// Flags for command definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommandFlags {
    /// Command accepts a range
    pub range: bool,
    /// Command accepts a count
    pub count: bool,
    /// Command accepts a register
    pub register: bool,
    /// Command accepts a bang (!)
    pub bang: bool,
    /// Command accepts arguments
    pub args: bool,
    /// Command modifies buffer
    pub modify: bool,
}

/// Definition of an ex command
#[derive(Debug, Clone)]
pub struct CommandDef {
    /// Full command name
    pub name: String,
    /// Minimum abbreviation length
    pub min_abbrev: usize,
    /// Command flags
    pub flags: CommandFlags,
    /// Help description
    pub description: String,
}

// ============================================================================
// Command Executor Trait
// ============================================================================

/// Trait for executing ex commands
pub trait CommandExecutor {
    /// Parse a command line string
    fn parse(&self, cmdline: &str) -> VimResult<ExCommand>;

    /// Execute a parsed command
    fn execute(&mut self, cmd: &ExCommand) -> CommandResult;

    /// Parse and execute a command line
    fn run(&mut self, cmdline: &str) -> CommandResult {
        match self.parse(cmdline) {
            Ok(cmd) => self.execute(&cmd),
            Err(e) => CommandResult::error(e),
        }
    }

    /// Get available commands
    fn commands(&self) -> Vec<&CommandDef>;

    /// Complete command name
    fn complete_command(&self, prefix: &str) -> Vec<String>;

    /// Complete command arguments
    fn complete_args(&self, cmd: &str, args: &str) -> Vec<String>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_range() {
        let whole = CommandRange::whole_file();
        assert!(whole.whole_file);

        let current = CommandRange::current_line();
        assert_eq!(current.start, Some(LineSpec::Current));
    }

    #[test]
    fn test_parse_command_range_whole_file() {
        let (range, rest) = parse_command_range("%s/foo/bar/").unwrap();
        assert!(range.whole_file);
        assert_eq!(rest, "s/foo/bar/");
    }

    #[test]
    fn test_parse_command_range_lines() {
        let (range, rest) = parse_command_range("1,5delete").unwrap();
        assert_eq!(
            range,
            CommandRange {
                start: Some(LineSpec::Absolute(LineNr(1))),
                end: Some(LineSpec::Absolute(LineNr(5))),
                whole_file: false,
            }
        );
        assert_eq!(rest, "delete");
    }

    #[test]
    fn test_parse_command_range_current_to_last() {
        let (range, rest) = parse_command_range(".,$p").unwrap();
        assert_eq!(
            range,
            CommandRange {
                start: Some(LineSpec::Current),
                end: Some(LineSpec::Last),
                whole_file: false,
            }
        );
        assert_eq!(rest, "p");
    }

    #[test]
    fn test_parse_command_range_single_line() {
        let (range, rest) = parse_command_range("10y").unwrap();
        assert_eq!(
            range,
            CommandRange {
                start: Some(LineSpec::Absolute(LineNr(10))),
                end: None,
                whole_file: false,
            }
        );
        assert_eq!(rest, "y");
    }

    #[test]
    fn test_parse_command_range_none() {
        let (range, rest) = parse_command_range("write").unwrap();
        assert_eq!(range, CommandRange::default());
        assert_eq!(rest, "write");
    }

    #[test]
    fn test_parse_command_range_missing_end_errors() {
        let err = parse_command_range("1,").unwrap_err();
        assert_eq!(err, VimError::InvalidRange("missing range end".into()));
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Command Behavioral Tests
        //!
        //! Tests derived from Neovim's test suite:
        //! - test/functional/ex_cmds/ - various command tests
        //!
        //! ## Key Quirks
        //!
        //! 1. **Command abbreviation**: `:w` = `:write`, `:q` = `:quit`.
        //!
        //! 2. **Range parsing**: `:1,5d` deletes lines 1-5, `:%s` = whole file.
        //!
        //! 3. **Bang behavior**: `:q!` = force quit, `:w!` = force write.
        //!
        //! 4. **Vertical bar**: `|` separates commands (`:w | q`).
        //!
        //! 5. **Special characters**: `%` = current file, `#` = alternate file.
    }
}
