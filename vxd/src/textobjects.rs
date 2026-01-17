//! Text objects (iw, aw, ip, etc).
//!
//! Text objects define regions of text for use with operators or visual mode.
//! They come in "inner" (i) and "a" (around/outer) variants.

use crate::cursor::CursorPosition;
use crate::types::*;

// ============================================================================
// Text Object Types
// ============================================================================

/// Text object variant (inner vs around)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextObjectVariant {
    /// Inner - just the content (iw, i", etc.)
    Inner,
    /// Around - includes surrounding whitespace/delimiters (aw, a", etc.)
    Around,
}

/// Text object kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextObjectKind {
    /// Word (w)
    Word,
    /// WORD (W)
    WORD,
    /// Sentence (s)
    Sentence,
    /// Paragraph (p)
    Paragraph,
    /// Parentheses block (( or ))
    Parens,
    /// Brackets block ([ or ])
    Brackets,
    /// Braces block ({ or })
    Braces,
    /// Angle brackets block (< or >)
    AngleBrackets,
    /// Double quotes (")
    DoubleQuote,
    /// Single quotes (')
    SingleQuote,
    /// Backticks (`)
    Backtick,
    /// Tag (<tag>...</tag>)
    Tag,
    /// Block (covers all bracket types)
    Block,
}

impl TextObjectKind {
    /// Get the opening/closing delimiters for this text object
    pub fn delimiters(&self) -> Option<(char, char)> {
        match self {
            TextObjectKind::Parens => Some(('(', ')')),
            TextObjectKind::Brackets => Some(('[', ']')),
            TextObjectKind::Braces => Some(('{', '}')),
            TextObjectKind::AngleBrackets => Some(('<', '>')),
            TextObjectKind::DoubleQuote => Some(('"', '"')),
            TextObjectKind::SingleQuote => Some(('\'', '\'')),
            TextObjectKind::Backtick => Some(('`', '`')),
            _ => None,
        }
    }

    /// Parse a text object from its key
    pub fn from_key(key: char) -> Option<Self> {
        match key {
            'w' => Some(TextObjectKind::Word),
            'W' => Some(TextObjectKind::WORD),
            's' => Some(TextObjectKind::Sentence),
            'p' => Some(TextObjectKind::Paragraph),
            '(' | ')' | 'b' => Some(TextObjectKind::Parens),
            '[' | ']' => Some(TextObjectKind::Brackets),
            '{' | '}' | 'B' => Some(TextObjectKind::Braces),
            '<' | '>' => Some(TextObjectKind::AngleBrackets),
            '"' => Some(TextObjectKind::DoubleQuote),
            '\'' => Some(TextObjectKind::SingleQuote),
            '`' => Some(TextObjectKind::Backtick),
            't' => Some(TextObjectKind::Tag),
            _ => None,
        }
    }
}

/// A complete text object specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextObject {
    /// The variant (inner or around)
    pub variant: TextObjectVariant,
    /// The kind of text object
    pub kind: TextObjectKind,
    /// Count (e.g., 2aw = two words)
    pub count: usize,
}

impl TextObject {
    /// Create a new text object
    pub fn new(variant: TextObjectVariant, kind: TextObjectKind) -> Self {
        TextObject {
            variant,
            kind,
            count: 1,
        }
    }

    /// Create an inner text object
    pub fn inner(kind: TextObjectKind) -> Self {
        TextObject::new(TextObjectVariant::Inner, kind)
    }

    /// Create an around text object
    pub fn around(kind: TextObjectKind) -> Self {
        TextObject::new(TextObjectVariant::Around, kind)
    }

    /// Set the count
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }
}

/// Result of finding a text object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextObjectMatch {
    /// Start position
    pub start: CursorPosition,
    /// End position
    pub end: CursorPosition,
    /// Whether the match is linewise
    pub linewise: bool,
}

impl TextObjectMatch {
    /// Check if the match is valid (non-empty)
    pub fn is_valid(&self) -> bool {
        self.start.line.0 <= self.end.line.0
    }

    /// Get as a line range
    pub fn line_range(&self) -> LineRange {
        LineRange::new(self.start.line, self.end.line)
    }
}

// ============================================================================
// Text Object Finder Trait
// ============================================================================

/// Context for finding text objects
pub struct TextObjectContext<'a> {
    /// Current cursor position
    pub cursor: CursorPosition,
    /// Function to get a line's content
    pub get_line: &'a dyn Fn(LineNr) -> Option<String>,
    /// Total line count
    pub line_count: usize,
}

/// Trait for finding text objects
pub trait TextObjectFinder {
    /// Find a text object from the current position
    fn find(&self, obj: TextObject, ctx: &TextObjectContext) -> Option<TextObjectMatch>;

    /// Find a word text object
    fn find_word(
        &self,
        variant: TextObjectVariant,
        big_word: bool,
        ctx: &TextObjectContext,
    ) -> Option<TextObjectMatch>;

    /// Find a sentence text object
    fn find_sentence(
        &self,
        variant: TextObjectVariant,
        ctx: &TextObjectContext,
    ) -> Option<TextObjectMatch>;

    /// Find a paragraph text object
    fn find_paragraph(
        &self,
        variant: TextObjectVariant,
        ctx: &TextObjectContext,
    ) -> Option<TextObjectMatch>;

    /// Find a quoted string text object
    fn find_quoted(
        &self,
        variant: TextObjectVariant,
        quote: char,
        ctx: &TextObjectContext,
    ) -> Option<TextObjectMatch>;

    /// Find a bracketed block text object
    fn find_block(
        &self,
        variant: TextObjectVariant,
        open: char,
        close: char,
        ctx: &TextObjectContext,
    ) -> Option<TextObjectMatch>;

    /// Find an XML/HTML tag text object
    fn find_tag(
        &self,
        variant: TextObjectVariant,
        ctx: &TextObjectContext,
    ) -> Option<TextObjectMatch>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_object_parsing() {
        assert_eq!(TextObjectKind::from_key('w'), Some(TextObjectKind::Word));
        assert_eq!(TextObjectKind::from_key('('), Some(TextObjectKind::Parens));
        assert_eq!(
            TextObjectKind::from_key('"'),
            Some(TextObjectKind::DoubleQuote)
        );
        assert_eq!(TextObjectKind::from_key('t'), Some(TextObjectKind::Tag));
    }

    #[test]
    fn test_delimiters() {
        assert_eq!(TextObjectKind::Parens.delimiters(), Some(('(', ')')));
        assert_eq!(TextObjectKind::DoubleQuote.delimiters(), Some(('"', '"')));
        assert_eq!(TextObjectKind::Word.delimiters(), None);
    }

    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Text Object Behavioral Tests
        //!
        //! ## Key Quirks
        //!
        //! 1. **Inner vs around**: `iw` selects word, `aw` includes surrounding space.
        //!    For quoted strings, `i"` excludes quotes, `a"` includes them.
        //!
        //! 2. **Cursor position**: The cursor doesn't need to be on the object.
        //!    `ci"` with cursor anywhere on line finds the quoted string.
        //!
        //! 3. **Count behavior**: `2aw` selects two words. `2i(` is not commonly used.
        //!
        //! 4. **Nesting**: `i(` finds the innermost parens containing cursor.
        //!    With count, `2i(` goes up one level.
        //!
        //! 5. **Quote searching**: Quote text objects search forward on the line if
        //!    cursor isn't inside quotes.
        //!
        //! 6. **Paragraph definition**: Paragraphs are separated by blank lines.
        //!
        //! 7. **Sentence definition**: Sentences end with .!? followed by space/EOL.
    }
}
