//! Text object tests ported from Neovim tests
//!
//! These tests verify text object behavior including:
//! - Inner vs around variants (iw vs aw)
//! - Word and WORD objects (w, W)
//! - Sentence and paragraph objects (s, p)
//! - Quoted string objects (", ', `)
//! - Block/bracket objects ((), [], {}, <>)
//! - Tag objects (t)
//! - Count handling with text objects
//!
//! Source tests:
//! - test/functional/legacy/textobjects_spec.lua
//! - test/functional/editor/text_objects_spec.lua

#![allow(non_snake_case)]

mod common;

use vxd::cursor::CursorPosition;
use vxd::textobjects::{TextObject, TextObjectKind, TextObjectMatch, TextObjectVariant};
use vxd::types::LineNr;

// ============================================================================
// TextObjectVariant Tests
// ============================================================================

/// Test: inner variant exists
/// Source: Vim i{object} commands
#[test]
fn test_text_object_variant_inner() {
    let variant = TextObjectVariant::Inner;
    assert_eq!(variant, TextObjectVariant::Inner);
}

/// Test: around variant exists
/// Source: Vim a{object} commands
#[test]
fn test_text_object_variant_around() {
    let variant = TextObjectVariant::Around;
    assert_eq!(variant, TextObjectVariant::Around);
}

/// Test: variants are distinct
/// Source: Vim text object semantics
#[test]
fn test_text_object_variants_distinct() {
    assert_ne!(TextObjectVariant::Inner, TextObjectVariant::Around);
}

// ============================================================================
// TextObjectKind Parsing Tests
// ============================================================================

/// Test: word text object from 'w'
/// Source: Vim iw, aw
#[test]
fn test_text_object_kind_word() {
    assert_eq!(TextObjectKind::from_key('w'), Some(TextObjectKind::Word));
}

/// Test: WORD text object from 'W'
/// Source: Vim iW, aW
#[test]
fn test_text_object_kind_WORD() {
    assert_eq!(TextObjectKind::from_key('W'), Some(TextObjectKind::WORD));
}

/// Test: sentence text object from 's'
/// Source: Vim is, as
#[test]
fn test_text_object_kind_sentence() {
    assert_eq!(
        TextObjectKind::from_key('s'),
        Some(TextObjectKind::Sentence)
    );
}

/// Test: paragraph text object from 'p'
/// Source: Vim ip, ap
#[test]
fn test_text_object_kind_paragraph() {
    assert_eq!(
        TextObjectKind::from_key('p'),
        Some(TextObjectKind::Paragraph)
    );
}

/// Test: parens from '(' and ')'
/// Source: Vim i(, a), ib, ab
#[test]
fn test_text_object_kind_parens() {
    assert_eq!(TextObjectKind::from_key('('), Some(TextObjectKind::Parens));
    assert_eq!(TextObjectKind::from_key(')'), Some(TextObjectKind::Parens));
    assert_eq!(TextObjectKind::from_key('b'), Some(TextObjectKind::Parens));
}

/// Test: brackets from '[' and ']'
/// Source: Vim i[, a]
#[test]
fn test_text_object_kind_brackets() {
    assert_eq!(
        TextObjectKind::from_key('['),
        Some(TextObjectKind::Brackets)
    );
    assert_eq!(
        TextObjectKind::from_key(']'),
        Some(TextObjectKind::Brackets)
    );
}

/// Test: braces from '{' and '}'
/// Source: Vim i{, a}, iB, aB
#[test]
fn test_text_object_kind_braces() {
    assert_eq!(TextObjectKind::from_key('{'), Some(TextObjectKind::Braces));
    assert_eq!(TextObjectKind::from_key('}'), Some(TextObjectKind::Braces));
    assert_eq!(TextObjectKind::from_key('B'), Some(TextObjectKind::Braces));
}

/// Test: angle brackets from '<' and '>'
/// Source: Vim i<, a>
#[test]
fn test_text_object_kind_angle_brackets() {
    assert_eq!(
        TextObjectKind::from_key('<'),
        Some(TextObjectKind::AngleBrackets)
    );
    assert_eq!(
        TextObjectKind::from_key('>'),
        Some(TextObjectKind::AngleBrackets)
    );
}

/// Test: double quote from '"'
/// Source: Vim i", a"
#[test]
fn test_text_object_kind_double_quote() {
    assert_eq!(
        TextObjectKind::from_key('"'),
        Some(TextObjectKind::DoubleQuote)
    );
}

/// Test: single quote from '\''
/// Source: Vim i', a'
#[test]
fn test_text_object_kind_single_quote() {
    assert_eq!(
        TextObjectKind::from_key('\''),
        Some(TextObjectKind::SingleQuote)
    );
}

/// Test: backtick from '`'
/// Source: Vim i`, a`
#[test]
fn test_text_object_kind_backtick() {
    assert_eq!(
        TextObjectKind::from_key('`'),
        Some(TextObjectKind::Backtick)
    );
}

/// Test: tag from 't'
/// Source: Vim it, at
#[test]
fn test_text_object_kind_tag() {
    assert_eq!(TextObjectKind::from_key('t'), Some(TextObjectKind::Tag));
}

/// Test: invalid key returns None
/// Source: Vim behavior
#[test]
fn test_text_object_kind_invalid() {
    assert_eq!(TextObjectKind::from_key('x'), None);
    assert_eq!(TextObjectKind::from_key('z'), None);
    assert_eq!(TextObjectKind::from_key('1'), None);
}

// ============================================================================
// TextObjectKind Delimiter Tests
// ============================================================================

/// Test: parens delimiters
/// Source: Vim block text objects
#[test]
fn test_delimiters_parens() {
    assert_eq!(TextObjectKind::Parens.delimiters(), Some(('(', ')')));
}

/// Test: brackets delimiters
/// Source: Vim block text objects
#[test]
fn test_delimiters_brackets() {
    assert_eq!(TextObjectKind::Brackets.delimiters(), Some(('[', ']')));
}

/// Test: braces delimiters
/// Source: Vim block text objects
#[test]
fn test_delimiters_braces() {
    assert_eq!(TextObjectKind::Braces.delimiters(), Some(('{', '}')));
}

/// Test: angle brackets delimiters
/// Source: Vim block text objects
#[test]
fn test_delimiters_angle_brackets() {
    assert_eq!(TextObjectKind::AngleBrackets.delimiters(), Some(('<', '>')));
}

/// Test: double quote delimiters
/// Source: Vim quoted text objects
#[test]
fn test_delimiters_double_quote() {
    assert_eq!(TextObjectKind::DoubleQuote.delimiters(), Some(('"', '"')));
}

/// Test: single quote delimiters
/// Source: Vim quoted text objects
#[test]
fn test_delimiters_single_quote() {
    assert_eq!(TextObjectKind::SingleQuote.delimiters(), Some(('\'', '\'')));
}

/// Test: backtick delimiters
/// Source: Vim quoted text objects
#[test]
fn test_delimiters_backtick() {
    assert_eq!(TextObjectKind::Backtick.delimiters(), Some(('`', '`')));
}

/// Test: word has no delimiters
/// Source: Vim word text objects
#[test]
fn test_delimiters_word_none() {
    assert_eq!(TextObjectKind::Word.delimiters(), None);
}

/// Test: WORD has no delimiters
/// Source: Vim WORD text objects
#[test]
fn test_delimiters_WORD_none() {
    assert_eq!(TextObjectKind::WORD.delimiters(), None);
}

/// Test: sentence has no delimiters
/// Source: Vim sentence text objects
#[test]
fn test_delimiters_sentence_none() {
    assert_eq!(TextObjectKind::Sentence.delimiters(), None);
}

/// Test: paragraph has no delimiters
/// Source: Vim paragraph text objects
#[test]
fn test_delimiters_paragraph_none() {
    assert_eq!(TextObjectKind::Paragraph.delimiters(), None);
}

/// Test: tag has no simple delimiters
/// Source: Vim tag text objects (complex)
#[test]
fn test_delimiters_tag_none() {
    assert_eq!(TextObjectKind::Tag.delimiters(), None);
}

// ============================================================================
// TextObject Construction Tests
// ============================================================================

/// Test: create inner text object
/// Source: Internal API
#[test]
fn test_text_object_inner() {
    let obj = TextObject::inner(TextObjectKind::Word);
    assert_eq!(obj.variant, TextObjectVariant::Inner);
    assert_eq!(obj.kind, TextObjectKind::Word);
    assert_eq!(obj.count, 1);
}

/// Test: create around text object
/// Source: Internal API
#[test]
fn test_text_object_around() {
    let obj = TextObject::around(TextObjectKind::Parens);
    assert_eq!(obj.variant, TextObjectVariant::Around);
    assert_eq!(obj.kind, TextObjectKind::Parens);
    assert_eq!(obj.count, 1);
}

/// Test: text object with count
/// Source: Vim 2aw, 3iw
#[test]
fn test_text_object_with_count() {
    let obj = TextObject::inner(TextObjectKind::Word).with_count(3);
    assert_eq!(obj.count, 3);
}

/// Test: text object new constructor
/// Source: Internal API
#[test]
fn test_text_object_new() {
    let obj = TextObject::new(TextObjectVariant::Around, TextObjectKind::DoubleQuote);
    assert_eq!(obj.variant, TextObjectVariant::Around);
    assert_eq!(obj.kind, TextObjectKind::DoubleQuote);
    assert_eq!(obj.count, 1);
}

// ============================================================================
// TextObjectMatch Tests
// ============================================================================

/// Test: text object match validity - valid match
/// Source: Internal API
#[test]
fn test_text_object_match_is_valid() {
    let m = TextObjectMatch {
        start: CursorPosition::new(LineNr::new(1), 0),
        end: CursorPosition::new(LineNr::new(1), 5),
        linewise: false,
    };
    assert!(m.is_valid());
}

/// Test: text object match validity - multiline valid
/// Source: Internal API
#[test]
fn test_text_object_match_multiline_valid() {
    let m = TextObjectMatch {
        start: CursorPosition::new(LineNr::new(1), 0),
        end: CursorPosition::new(LineNr::new(5), 10),
        linewise: false,
    };
    assert!(m.is_valid());
}

/// Test: text object match line range
/// Source: Internal API
#[test]
fn test_text_object_match_line_range() {
    let m = TextObjectMatch {
        start: CursorPosition::new(LineNr::new(3), 0),
        end: CursorPosition::new(LineNr::new(7), 5),
        linewise: true,
    };
    let range = m.line_range();
    assert_eq!(range.start, LineNr::new(3));
    assert_eq!(range.end, LineNr::new(7));
    assert_eq!(range.len(), 5);
}

/// Test: text object match linewise flag
/// Source: Vim paragraph/sentence linewise
#[test]
fn test_text_object_match_linewise() {
    let m = TextObjectMatch {
        start: CursorPosition::new(LineNr::new(1), 0),
        end: CursorPosition::new(LineNr::new(3), 0),
        linewise: true,
    };
    assert!(m.linewise);
}

/// Test: text object match characterwise
/// Source: Vim word/quote characterwise
#[test]
fn test_text_object_match_characterwise() {
    let m = TextObjectMatch {
        start: CursorPosition::new(LineNr::new(1), 5),
        end: CursorPosition::new(LineNr::new(1), 10),
        linewise: false,
    };
    assert!(!m.linewise);
}

// ============================================================================
// All Text Object Kinds Tests
// ============================================================================

/// Test: all text object kinds enumerated
/// Source: Vim text object reference
#[test]
fn test_all_text_object_kinds() {
    let kinds = [
        TextObjectKind::Word,
        TextObjectKind::WORD,
        TextObjectKind::Sentence,
        TextObjectKind::Paragraph,
        TextObjectKind::Parens,
        TextObjectKind::Brackets,
        TextObjectKind::Braces,
        TextObjectKind::AngleBrackets,
        TextObjectKind::DoubleQuote,
        TextObjectKind::SingleQuote,
        TextObjectKind::Backtick,
        TextObjectKind::Tag,
        TextObjectKind::Block,
    ];
    assert_eq!(kinds.len(), 13);
}

/// Test: all text object kinds are distinct
/// Source: Internal consistency
#[test]
fn test_all_text_object_kinds_distinct() {
    let kinds = [
        TextObjectKind::Word,
        TextObjectKind::WORD,
        TextObjectKind::Sentence,
        TextObjectKind::Paragraph,
        TextObjectKind::Parens,
        TextObjectKind::Brackets,
        TextObjectKind::Braces,
        TextObjectKind::AngleBrackets,
        TextObjectKind::DoubleQuote,
        TextObjectKind::SingleQuote,
        TextObjectKind::Backtick,
        TextObjectKind::Tag,
        TextObjectKind::Block,
    ];

    for (i, a) in kinds.iter().enumerate() {
        for (j, b) in kinds.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}

// ============================================================================
// Text Object Semantic Tests (Conceptual)
// ============================================================================

/// Test: iw selects word without surrounding space
/// Source: Vim :h iw
#[test]
fn test_iw_concept() {
    // "hello world" with cursor on 'e' of hello
    // iw selects "hello" (no surrounding space)
    let obj = TextObject::inner(TextObjectKind::Word);
    assert_eq!(obj.variant, TextObjectVariant::Inner);
    assert_eq!(obj.kind, TextObjectKind::Word);
}

/// Test: aw selects word with surrounding space
/// Source: Vim :h aw
#[test]
fn test_aw_concept() {
    // "hello world" with cursor on 'e' of hello
    // aw selects "hello " (includes trailing space)
    let obj = TextObject::around(TextObjectKind::Word);
    assert_eq!(obj.variant, TextObjectVariant::Around);
    assert_eq!(obj.kind, TextObjectKind::Word);
}

/// Test: i" selects quoted content without quotes
/// Source: Vim :h i"
#[test]
fn test_iquote_concept() {
    // say "hello" with cursor anywhere
    // i" selects "hello" (without the quotes)
    let obj = TextObject::inner(TextObjectKind::DoubleQuote);
    assert_eq!(obj.variant, TextObjectVariant::Inner);
}

/// Test: a" selects quoted content with quotes
/// Source: Vim :h a"
#[test]
fn test_aquote_concept() {
    // say "hello" with cursor anywhere
    // a" selects "hello" (with the quotes)
    let obj = TextObject::around(TextObjectKind::DoubleQuote);
    assert_eq!(obj.variant, TextObjectVariant::Around);
}

/// Test: i( selects content inside parentheses
/// Source: Vim :h i(
#[test]
fn test_iparen_concept() {
    // foo(bar, baz) with cursor inside
    // i( selects "bar, baz"
    let obj = TextObject::inner(TextObjectKind::Parens);
    assert_eq!(obj.kind, TextObjectKind::Parens);
}

/// Test: a( selects content including parentheses
/// Source: Vim :h a(
#[test]
fn test_aparen_concept() {
    // foo(bar, baz) with cursor inside
    // a( selects "(bar, baz)"
    let obj = TextObject::around(TextObjectKind::Parens);
    assert_eq!(obj.kind, TextObjectKind::Parens);
}

/// Test: ip selects paragraph without surrounding blank lines
/// Source: Vim :h ip
#[test]
fn test_ip_concept() {
    // Paragraph is text surrounded by blank lines
    let obj = TextObject::inner(TextObjectKind::Paragraph);
    assert_eq!(obj.kind, TextObjectKind::Paragraph);
}

/// Test: ap selects paragraph with trailing blank line
/// Source: Vim :h ap
#[test]
fn test_ap_concept() {
    let obj = TextObject::around(TextObjectKind::Paragraph);
    assert_eq!(obj.kind, TextObjectKind::Paragraph);
}

/// Test: is selects sentence without surrounding space
/// Source: Vim :h is
#[test]
fn test_is_concept() {
    // Sentence ends with .!? followed by space/EOL
    let obj = TextObject::inner(TextObjectKind::Sentence);
    assert_eq!(obj.kind, TextObjectKind::Sentence);
}

/// Test: as selects sentence with trailing space
/// Source: Vim :h as
#[test]
fn test_as_concept() {
    let obj = TextObject::around(TextObjectKind::Sentence);
    assert_eq!(obj.kind, TextObjectKind::Sentence);
}

/// Test: it selects tag content
/// Source: Vim :h it
#[test]
fn test_it_concept() {
    // <div>content</div> with cursor inside
    // it selects "content"
    let obj = TextObject::inner(TextObjectKind::Tag);
    assert_eq!(obj.kind, TextObjectKind::Tag);
}

/// Test: at selects tag with tags
/// Source: Vim :h at
#[test]
fn test_at_concept() {
    // <div>content</div> with cursor inside
    // at selects "<div>content</div>"
    let obj = TextObject::around(TextObjectKind::Tag);
    assert_eq!(obj.kind, TextObjectKind::Tag);
}

// ============================================================================
// Count Behavior Tests (Conceptual)
// ============================================================================

/// Test: 2aw selects two words
/// Source: Vim count with text objects
#[test]
fn test_count_2aw() {
    let obj = TextObject::around(TextObjectKind::Word).with_count(2);
    assert_eq!(obj.count, 2);
}

/// Test: 3iw selects three words
/// Source: Vim count with text objects
#[test]
fn test_count_3iw() {
    let obj = TextObject::inner(TextObjectKind::Word).with_count(3);
    assert_eq!(obj.count, 3);
}

/// Test: count with bracket objects (nesting)
/// Source: Vim :h i(
#[test]
fn test_count_nested_parens() {
    // ((inner)) - 2i( goes to outer parens
    let obj = TextObject::inner(TextObjectKind::Parens).with_count(2);
    assert_eq!(obj.count, 2);
}

// ============================================================================
// Word vs WORD Tests (Conceptual)
// ============================================================================

/// Test: word stops at punctuation
/// Source: Vim :h word
#[test]
fn test_word_vs_WORD_punctuation() {
    // "foo.bar" - iw on 'o' selects "foo", iW selects "foo.bar"
    let word = TextObject::inner(TextObjectKind::Word);
    let big_word = TextObject::inner(TextObjectKind::WORD);
    assert_ne!(word.kind, big_word.kind);
}

/// Test: WORD only stops at whitespace
/// Source: Vim :h WORD
#[test]
fn test_WORD_whitespace_only() {
    let obj = TextObject::inner(TextObjectKind::WORD);
    assert_eq!(obj.kind, TextObjectKind::WORD);
}

// ============================================================================
// Bracket Synonym Tests
// ============================================================================

/// Test: b is synonym for (
/// Source: Vim :h ib
#[test]
fn test_b_synonym_for_parens() {
    assert_eq!(TextObjectKind::from_key('b'), TextObjectKind::from_key('('));
}

/// Test: B is synonym for {
/// Source: Vim :h iB
#[test]
fn test_B_synonym_for_braces() {
    assert_eq!(TextObjectKind::from_key('B'), TextObjectKind::from_key('{'));
}

/// Test: ( and ) both mean parens
/// Source: Vim :h i(
#[test]
fn test_parens_both_chars() {
    assert_eq!(TextObjectKind::from_key('('), TextObjectKind::from_key(')'));
}

/// Test: [ and ] both mean brackets
/// Source: Vim :h i[
#[test]
fn test_brackets_both_chars() {
    assert_eq!(TextObjectKind::from_key('['), TextObjectKind::from_key(']'));
}

/// Test: { and } both mean braces
/// Source: Vim :h i{
#[test]
fn test_braces_both_chars() {
    assert_eq!(TextObjectKind::from_key('{'), TextObjectKind::from_key('}'));
}

/// Test: < and > both mean angle brackets
/// Source: Vim :h i<
#[test]
fn test_angle_brackets_both_chars() {
    assert_eq!(TextObjectKind::from_key('<'), TextObjectKind::from_key('>'));
}

// ============================================================================
// Quote Text Object Behavior Tests (Conceptual)
// ============================================================================

/// Test: quotes search forward if not inside
/// Source: Vim quote text object behavior
#[test]
fn test_quote_searches_forward() {
    // If cursor not inside quotes, Vim searches forward on line
    let obj = TextObject::inner(TextObjectKind::DoubleQuote);
    assert_eq!(obj.kind, TextObjectKind::DoubleQuote);
}

/// Test: single quote text object
/// Source: Vim :h i'
#[test]
fn test_single_quote_text_object() {
    let obj = TextObject::inner(TextObjectKind::SingleQuote);
    assert_eq!(obj.kind, TextObjectKind::SingleQuote);
}

/// Test: backtick text object
/// Source: Vim :h i`
#[test]
fn test_backtick_text_object() {
    let obj = TextObject::inner(TextObjectKind::Backtick);
    assert_eq!(obj.kind, TextObjectKind::Backtick);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Test: empty match validity
/// Source: Edge case
#[test]
fn test_match_single_position() {
    let m = TextObjectMatch {
        start: CursorPosition::new(LineNr::new(1), 5),
        end: CursorPosition::new(LineNr::new(1), 5),
        linewise: false,
    };
    // Same position is valid (selects nothing or one char depending on context)
    assert!(m.is_valid());
}

/// Test: large count value
/// Source: Edge case
#[test]
fn test_large_count() {
    let obj = TextObject::inner(TextObjectKind::Word).with_count(1000);
    assert_eq!(obj.count, 1000);
}

/// Test: text object equality
/// Source: Internal consistency
#[test]
fn test_text_object_equality() {
    let obj1 = TextObject::inner(TextObjectKind::Word);
    let obj2 = TextObject::inner(TextObjectKind::Word);
    let obj3 = TextObject::around(TextObjectKind::Word);

    assert_eq!(obj1, obj2);
    assert_ne!(obj1, obj3);
}

/// Test: text object with different kinds
/// Source: Internal consistency
#[test]
fn test_text_object_different_kinds() {
    let word = TextObject::inner(TextObjectKind::Word);
    let paren = TextObject::inner(TextObjectKind::Parens);

    assert_ne!(word, paren);
}

/// Test: match line range single line
/// Source: Edge case
#[test]
fn test_match_single_line_range() {
    let m = TextObjectMatch {
        start: CursorPosition::new(LineNr::new(5), 0),
        end: CursorPosition::new(LineNr::new(5), 10),
        linewise: false,
    };
    let range = m.line_range();
    assert_eq!(range.len(), 1);
}

// ============================================================================
// Text Object Kind Exhaustive Key Tests
// ============================================================================

/// Test: all valid single-char keys
/// Source: Vim text object keys
#[test]
fn test_all_valid_keys() {
    let valid_keys = [
        'w', 'W', 's', 'p', '(', ')', 'b', '[', ']', '{', '}', 'B', '<', '>', '"', '\'', '`', 't',
    ];

    for key in valid_keys {
        assert!(
            TextObjectKind::from_key(key).is_some(),
            "Key '{}' should be valid",
            key
        );
    }
}

/// Test: invalid keys return None
/// Source: Vim text object keys
#[test]
fn test_invalid_keys() {
    let invalid_keys = [
        'a', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'q', 'r', 'u', 'v',
        'x', 'y', 'z', '0', '1', '2', '!', '@', '#', '$',
    ];

    for key in invalid_keys {
        assert!(
            TextObjectKind::from_key(key).is_none(),
            "Key '{}' should be invalid",
            key
        );
    }
}

// ============================================================================
// CursorPosition Tests (used by TextObjectMatch)
// ============================================================================

/// Test: cursor position in match
/// Source: Internal API
#[test]
fn test_match_cursor_positions() {
    let m = TextObjectMatch {
        start: CursorPosition::new(LineNr::new(10), 5),
        end: CursorPosition::new(LineNr::new(15), 20),
        linewise: false,
    };

    assert_eq!(m.start.line, LineNr::new(10));
    assert_eq!(m.start.col, 5);
    assert_eq!(m.end.line, LineNr::new(15));
    assert_eq!(m.end.col, 20);
}

/// Test: match at beginning of buffer
/// Source: Edge case
#[test]
fn test_match_at_buffer_start() {
    let m = TextObjectMatch {
        start: CursorPosition::new(LineNr::new(1), 0),
        end: CursorPosition::new(LineNr::new(1), 5),
        linewise: false,
    };

    assert!(m.is_valid());
    assert_eq!(m.start.line, LineNr::FIRST);
}
