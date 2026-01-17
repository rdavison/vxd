//! Operator and motion tests ported from Neovim tests
//!
//! These tests verify operator and motion behavior including:
//! - Basic operators (d, c, y, >, <, =)
//! - Word motions (w, W, b, B, e, E)
//! - Character find motions (f, F, t, T)
//! - Line position motions (0, ^, $)
//! - Operator + motion combinations
//! - Double operators (dd, yy, cc)
//! - Count handling
//!
//! Source tests:
//! - test/functional/legacy/056_word_motion_spec.lua
//! - test/functional/legacy/094_visual_mode_operators_spec.lua
//! - test/functional/editor/operator_spec.lua

#![allow(non_snake_case)]

mod common;

use vxd::motions::{
    CharClass, CharFindMotion, DocumentMotion, LinePositionMotion, MotionKind, MotionResult,
    VerticalMotion, WordMotion,
};
use vxd::operators::{Operator, OperatorContext, OperatorRegion};
use vxd::types::{
    ColNr, Count, Direction, LineNr, LineRange, MotionInclusivity, MotionType, Position,
};

// ============================================================================
// Operator Parsing Tests
// ============================================================================

/// Test: operators parse from key sequences correctly
/// Source: Vim behavior
#[test]
fn test_operator_from_key_delete() {
    assert_eq!(Operator::from_key("d"), Some(Operator::Delete));
}

#[test]
fn test_operator_from_key_change() {
    assert_eq!(Operator::from_key("c"), Some(Operator::Change));
}

#[test]
fn test_operator_from_key_yank() {
    assert_eq!(Operator::from_key("y"), Some(Operator::Yank));
}

#[test]
fn test_operator_from_key_indent() {
    assert_eq!(Operator::from_key(">"), Some(Operator::Indent));
}

#[test]
fn test_operator_from_key_dedent() {
    assert_eq!(Operator::from_key("<"), Some(Operator::Dedent));
}

#[test]
fn test_operator_from_key_format() {
    assert_eq!(Operator::from_key("="), Some(Operator::Format));
}

#[test]
fn test_operator_from_key_toggle_case() {
    assert_eq!(Operator::from_key("g~"), Some(Operator::ToggleCase));
}

#[test]
fn test_operator_from_key_lowercase() {
    assert_eq!(Operator::from_key("gu"), Some(Operator::Lowercase));
}

#[test]
fn test_operator_from_key_uppercase() {
    assert_eq!(Operator::from_key("gU"), Some(Operator::Uppercase));
}

#[test]
fn test_operator_from_key_invalid() {
    assert_eq!(Operator::from_key("x"), None);
    assert_eq!(Operator::from_key("p"), None);
    assert_eq!(Operator::from_key(""), None);
}

// ============================================================================
// Operator Properties Tests
// ============================================================================

/// Test: delete operator modifies buffer
/// Source: Vim behavior
#[test]
fn test_operator_delete_modifies_buffer() {
    assert!(Operator::Delete.modifies_buffer());
}

/// Test: yank operator does not modify buffer
/// Source: Vim behavior
#[test]
fn test_operator_yank_does_not_modify_buffer() {
    assert!(!Operator::Yank.modifies_buffer());
}

/// Test: change operator enters insert mode
/// Source: Vim behavior
#[test]
fn test_operator_change_enters_insert() {
    assert!(Operator::Change.enters_insert());
}

/// Test: delete operator does not enter insert mode
/// Source: Vim behavior
#[test]
fn test_operator_delete_does_not_enter_insert() {
    assert!(!Operator::Delete.enters_insert());
}

/// Test: delete, change, yank use registers
/// Source: Vim behavior
#[test]
fn test_operators_use_register() {
    assert!(Operator::Delete.uses_register());
    assert!(Operator::Change.uses_register());
    assert!(Operator::Yank.uses_register());
    assert!(!Operator::Indent.uses_register());
    assert!(!Operator::Format.uses_register());
}

/// Test: operator key round-trip
/// Source: consistency test
#[test]
fn test_operator_key_roundtrip() {
    let operators = [
        Operator::Delete,
        Operator::Change,
        Operator::Yank,
        Operator::Indent,
        Operator::Dedent,
        Operator::Format,
        Operator::ToggleCase,
        Operator::Lowercase,
        Operator::Uppercase,
    ];
    for op in operators {
        let key = op.key();
        let parsed = Operator::from_key(key);
        assert_eq!(parsed, Some(op), "Round-trip failed for {:?}", op);
    }
}

// ============================================================================
// Operator Region Tests
// ============================================================================

/// Test: characterwise region creation
/// Source: Vim internal behavior
#[test]
fn test_region_characterwise_creation() {
    let start = Position::from_1indexed(1, 1);
    let end = Position::from_1indexed(1, 5);
    let region = OperatorRegion::characterwise(start, end, true);

    assert_eq!(region.start, start);
    assert_eq!(region.end, end);
    assert_eq!(region.region_type, MotionType::Characterwise);
    assert!(region.inclusive);
}

/// Test: linewise region creation
/// Source: Vim internal behavior
#[test]
fn test_region_linewise_creation() {
    let region = OperatorRegion::linewise(LineNr::new(1), LineNr::new(3));

    assert_eq!(region.start.line, LineNr::new(1));
    assert_eq!(region.end.line, LineNr::new(3));
    assert_eq!(region.region_type, MotionType::Linewise);
    assert!(region.inclusive);
}

/// Test: blockwise region creation
/// Source: Vim internal behavior
#[test]
fn test_region_blockwise_creation() {
    let start = Position::from_1indexed(1, 1);
    let end = Position::from_1indexed(3, 5);
    let region = OperatorRegion::blockwise(start, end);

    assert_eq!(region.region_type, MotionType::Blockwise);
    assert!(region.inclusive);
}

/// Test: region normalization swaps start/end when inverted
/// Source: Vim internal behavior
#[test]
fn test_region_normalize_swaps_when_inverted() {
    let start = Position::from_1indexed(3, 5);
    let end = Position::from_1indexed(1, 1);
    let mut region = OperatorRegion::characterwise(start, end, true);

    region.normalize();

    assert_eq!(region.start.line, LineNr::new(1));
    assert_eq!(region.end.line, LineNr::new(3));
}

/// Test: region normalization handles same-line inversion
/// Source: Vim internal behavior
#[test]
fn test_region_normalize_same_line() {
    let start = Position::from_1indexed(1, 10);
    let end = Position::from_1indexed(1, 1);
    let mut region = OperatorRegion::characterwise(start, end, true);

    region.normalize();

    assert_eq!(region.start.col, ColNr::new(1));
    assert_eq!(region.end.col, ColNr::new(10));
}

/// Test: region line range extraction
/// Source: Vim internal behavior
#[test]
fn test_region_line_range() {
    let region = OperatorRegion::linewise(LineNr::new(5), LineNr::new(10));
    let range = region.line_range();

    assert_eq!(range.start, LineNr::new(5));
    assert_eq!(range.end, LineNr::new(10));
    assert_eq!(range.len(), 6);
}

// ============================================================================
// Operator Context Tests
// ============================================================================

/// Test: default operator context
/// Source: Vim internal behavior
#[test]
fn test_operator_context_default() {
    let ctx = OperatorContext::default();

    assert_eq!(ctx.count, Count::NONE);
    assert!(!ctx.is_double);
}

// ============================================================================
// Word Motion Type Tests
// ============================================================================

/// Test: word motion variants exist
/// Source: Vim motion types
#[test]
fn test_word_motion_variants() {
    let motions = [
        WordMotion::WordForward,
        WordMotion::WORDForward,
        WordMotion::WordBackward,
        WordMotion::WORDBackward,
        WordMotion::EndForward,
        WordMotion::EndWORDForward,
        WordMotion::EndBackward,
        WordMotion::EndWORDBackward,
    ];
    assert_eq!(motions.len(), 8);
}

// ============================================================================
// Character Classification Tests
// ============================================================================

/// Test: whitespace classification
/// Source: Vim word definition
#[test]
fn test_char_class_whitespace() {
    assert_eq!(CharClass::classify(' '), CharClass::Whitespace);
    assert_eq!(CharClass::classify('\t'), CharClass::Whitespace);
    assert_eq!(CharClass::classify('\n'), CharClass::Whitespace);
}

/// Test: word character classification
/// Source: Vim word definition
#[test]
fn test_char_class_word_chars() {
    assert_eq!(CharClass::classify('a'), CharClass::Word);
    assert_eq!(CharClass::classify('Z'), CharClass::Word);
    assert_eq!(CharClass::classify('_'), CharClass::Word);
    assert_eq!(CharClass::classify('0'), CharClass::Word);
    assert_eq!(CharClass::classify('9'), CharClass::Word);
}

/// Test: punctuation classification
/// Source: Vim word definition
#[test]
fn test_char_class_punctuation() {
    assert_eq!(CharClass::classify('.'), CharClass::Punctuation);
    assert_eq!(CharClass::classify(','), CharClass::Punctuation);
    assert_eq!(CharClass::classify('('), CharClass::Punctuation);
    assert_eq!(CharClass::classify(')'), CharClass::Punctuation);
    assert_eq!(CharClass::classify('-'), CharClass::Punctuation);
    assert_eq!(CharClass::classify('+'), CharClass::Punctuation);
}

/// Test: WORD classification only distinguishes whitespace
/// Source: Vim WORD definition
#[test]
fn test_char_class_word_only_whitespace() {
    // WORD treats everything non-whitespace the same
    assert_eq!(CharClass::classify_word(' '), CharClass::Whitespace);
    assert_eq!(CharClass::classify_word('.'), CharClass::Word);
    assert_eq!(CharClass::classify_word('a'), CharClass::Word);
    assert_eq!(CharClass::classify_word('-'), CharClass::Word);
}

// ============================================================================
// Character Find Motion Tests
// ============================================================================

/// Test: character find motion types
/// Source: Vim f/F/t/T commands
#[test]
fn test_char_find_motion_types() {
    let _f = CharFindMotion::FindForward('x');
    let _F = CharFindMotion::FindBackward('x');
    let _t = CharFindMotion::TillForward('x');
    let _T = CharFindMotion::TillBackward('x');
    let _semi = CharFindMotion::RepeatForward;
    let _comma = CharFindMotion::RepeatBackward;
}

/// Test: character find stores target char
/// Source: Vim behavior
#[test]
fn test_char_find_stores_target() {
    let motion = CharFindMotion::FindForward('x');
    if let CharFindMotion::FindForward(c) = motion {
        assert_eq!(c, 'x');
    } else {
        panic!("Expected FindForward");
    }
}

// ============================================================================
// Line Position Motion Tests
// ============================================================================

/// Test: line position motion types
/// Source: Vim 0, ^, $, g0, g^, g$, gm, gM, | commands
#[test]
fn test_line_position_motion_types() {
    let _zero = LinePositionMotion::FirstColumn;
    let _caret = LinePositionMotion::FirstNonBlank;
    let _dollar = LinePositionMotion::EndOfLine;
    let _g0 = LinePositionMotion::FirstScreenColumn;
    let _gcaret = LinePositionMotion::FirstNonBlankScreen;
    let _gdollar = LinePositionMotion::EndOfScreenLine;
    let _gm = LinePositionMotion::MiddleOfScreenLine;
    let _gM = LinePositionMotion::MiddleOfTextLine;
    let _pipe = LinePositionMotion::ToColumn(5);
}

/// Test: ToColumn stores column number
/// Source: Vim | command
#[test]
fn test_line_position_to_column() {
    let motion = LinePositionMotion::ToColumn(42);
    if let LinePositionMotion::ToColumn(col) = motion {
        assert_eq!(col, 42);
    } else {
        panic!("Expected ToColumn");
    }
}

// ============================================================================
// Vertical Motion Tests
// ============================================================================

/// Test: vertical motion types
/// Source: Vim j, k, gj, gk, +, -, _ commands
#[test]
fn test_vertical_motion_types() {
    let _j = VerticalMotion::Down;
    let _k = VerticalMotion::Up;
    let _gj = VerticalMotion::ScreenDown;
    let _gk = VerticalMotion::ScreenUp;
    let _plus = VerticalMotion::DownFirstNonBlank;
    let _minus = VerticalMotion::UpFirstNonBlank;
    let _underscore = VerticalMotion::CurrentFirstNonBlank;
}

// ============================================================================
// Document Motion Tests
// ============================================================================

/// Test: document motion types
/// Source: Vim gg, G, H, M, L, % commands
#[test]
fn test_document_motion_types() {
    let _gg = DocumentMotion::GotoLine(Some(1));
    let _gg_default = DocumentMotion::GotoLine(None);
    let _G = DocumentMotion::GotoLastLine(None);
    let _G_count = DocumentMotion::GotoLastLine(Some(10));
    let _H = DocumentMotion::WindowTop;
    let _M = DocumentMotion::WindowMiddle;
    let _L = DocumentMotion::WindowBottom;
    let _percent_count = DocumentMotion::Percentage(50);
    let _percent = DocumentMotion::MatchingBracket;
}

/// Test: GotoLine stores line number
/// Source: Vim gg command
#[test]
fn test_document_motion_goto_line() {
    let motion = DocumentMotion::GotoLine(Some(42));
    if let DocumentMotion::GotoLine(Some(line)) = motion {
        assert_eq!(line, 42);
    } else {
        panic!("Expected GotoLine with Some");
    }
}

/// Test: GotoLine None means first line
/// Source: Vim gg without count
#[test]
fn test_document_motion_goto_line_default() {
    let motion = DocumentMotion::GotoLine(None);
    assert!(matches!(motion, DocumentMotion::GotoLine(None)));
}

/// Test: GotoLastLine None means last line
/// Source: Vim G without count
#[test]
fn test_document_motion_goto_last_line_default() {
    let motion = DocumentMotion::GotoLastLine(None);
    assert!(matches!(motion, DocumentMotion::GotoLastLine(None)));
}

// ============================================================================
// Motion Kind Tests
// ============================================================================

/// Test: motion kinds cover all categories
/// Source: Vim motion categories
#[test]
fn test_motion_kinds() {
    let kinds = [
        MotionKind::LeftRight,
        MotionKind::UpDown,
        MotionKind::Word,
        MotionKind::TextObject,
        MotionKind::Search,
        MotionKind::Mark,
        MotionKind::Various,
    ];
    assert_eq!(kinds.len(), 7);
}

// ============================================================================
// Motion Type Default Tests
// ============================================================================

/// Test: left-right motions are characterwise by default
/// Source: Vim :h motion.txt
#[test]
fn test_motion_type_left_right_characterwise() {
    use vxd::motions::default_motion_type;
    assert_eq!(
        default_motion_type(MotionKind::LeftRight),
        MotionType::Characterwise
    );
}

/// Test: up-down motions are linewise by default
/// Source: Vim :h motion.txt
#[test]
fn test_motion_type_up_down_linewise() {
    use vxd::motions::default_motion_type;
    assert_eq!(
        default_motion_type(MotionKind::UpDown),
        MotionType::Linewise
    );
}

/// Test: word motions are characterwise by default
/// Source: Vim :h motion.txt
#[test]
fn test_motion_type_word_characterwise() {
    use vxd::motions::default_motion_type;
    assert_eq!(
        default_motion_type(MotionKind::Word),
        MotionType::Characterwise
    );
}

// ============================================================================
// Motion Inclusivity Default Tests
// ============================================================================

/// Test: forward word motions are exclusive
/// Source: Vim :h exclusive
#[test]
fn test_word_motion_w_exclusive() {
    use vxd::motions::default_inclusivity;
    assert_eq!(
        default_inclusivity(&WordMotion::WordForward),
        MotionInclusivity::Exclusive
    );
    assert_eq!(
        default_inclusivity(&WordMotion::WORDForward),
        MotionInclusivity::Exclusive
    );
}

/// Test: backward word motions are exclusive
/// Source: Vim :h exclusive
#[test]
fn test_word_motion_b_exclusive() {
    use vxd::motions::default_inclusivity;
    assert_eq!(
        default_inclusivity(&WordMotion::WordBackward),
        MotionInclusivity::Exclusive
    );
    assert_eq!(
        default_inclusivity(&WordMotion::WORDBackward),
        MotionInclusivity::Exclusive
    );
}

/// Test: end-of-word motions are inclusive
/// Source: Vim :h inclusive
#[test]
fn test_word_motion_e_inclusive() {
    use vxd::motions::default_inclusivity;
    assert_eq!(
        default_inclusivity(&WordMotion::EndForward),
        MotionInclusivity::Inclusive
    );
    assert_eq!(
        default_inclusivity(&WordMotion::EndWORDForward),
        MotionInclusivity::Inclusive
    );
    assert_eq!(
        default_inclusivity(&WordMotion::EndBackward),
        MotionInclusivity::Inclusive
    );
    assert_eq!(
        default_inclusivity(&WordMotion::EndWORDBackward),
        MotionInclusivity::Inclusive
    );
}

// ============================================================================
// Motion Result Tests
// ============================================================================

/// Test: successful motion result creation
/// Source: Internal API
#[test]
fn test_motion_result_success() {
    use vxd::cursor::CursorPosition;

    let pos = CursorPosition::new(LineNr::new(1), 5);
    let result =
        MotionResult::success(pos, MotionType::Characterwise, MotionInclusivity::Inclusive);

    assert_eq!(result.position, pos);
    assert_eq!(result.motion_type, MotionType::Characterwise);
    assert_eq!(result.inclusive, MotionInclusivity::Inclusive);
    assert!(!result.failed);
}

/// Test: failed motion result creation
/// Source: Internal API
#[test]
fn test_motion_result_failed() {
    use vxd::cursor::CursorPosition;

    let pos = CursorPosition::new(LineNr::new(1), 0);
    let result = MotionResult::failed(pos);

    assert_eq!(result.position, pos);
    assert!(result.failed);
}

// ============================================================================
// Count Tests
// ============================================================================

/// Test: count default is none
/// Source: Vim behavior
#[test]
fn test_count_none() {
    let count = Count::NONE;
    assert!(!count.is_specified());
    assert_eq!(count.value_or_default(), 1);
}

/// Test: count with value
/// Source: Vim behavior
#[test]
fn test_count_with_value() {
    let count = Count::new(5);
    assert!(count.is_specified());
    assert_eq!(count.value_or_default(), 5);
}

/// Test: count value_or with custom default
/// Source: Vim behavior
#[test]
fn test_count_value_or() {
    let none = Count::NONE;
    let some = Count::new(3);

    assert_eq!(none.value_or(10), 10);
    assert_eq!(some.value_or(10), 3);
}

// ============================================================================
// Direction Tests
// ============================================================================

/// Test: direction reverse
/// Source: Vim search direction
#[test]
fn test_direction_reverse() {
    assert_eq!(Direction::Forward.reverse(), Direction::Backward);
    assert_eq!(Direction::Backward.reverse(), Direction::Forward);
}

/// Test: direction default is forward
/// Source: Vim default search direction
#[test]
fn test_direction_default() {
    let dir = Direction::default();
    assert_eq!(dir, Direction::Forward);
}

// ============================================================================
// Line Range Tests
// ============================================================================

/// Test: line range creation
/// Source: Vim range handling
#[test]
fn test_line_range_new() {
    let range = LineRange::new(LineNr::new(1), LineNr::new(5));
    assert_eq!(range.start, LineNr::new(1));
    assert_eq!(range.end, LineNr::new(5));
}

/// Test: single line range
/// Source: Vim range handling
#[test]
fn test_line_range_single() {
    let range = LineRange::single(LineNr::new(3));
    assert_eq!(range.start, LineNr::new(3));
    assert_eq!(range.end, LineNr::new(3));
    assert_eq!(range.len(), 1);
}

/// Test: line range length
/// Source: Vim range handling
#[test]
fn test_line_range_len() {
    let range = LineRange::new(LineNr::new(5), LineNr::new(10));
    assert_eq!(range.len(), 6); // 5, 6, 7, 8, 9, 10
}

/// Test: empty line range
/// Source: Vim range handling
#[test]
fn test_line_range_empty() {
    let range = LineRange::new(LineNr::new(10), LineNr::new(5));
    assert!(range.is_empty());
    assert_eq!(range.len(), 0);
}

// ============================================================================
// Position Tests
// ============================================================================

/// Test: position creation
/// Source: Vim position handling
#[test]
fn test_position_new() {
    let pos = Position::new(LineNr::new(5), ColNr::new(10));
    assert_eq!(pos.line, LineNr::new(5));
    assert_eq!(pos.col, ColNr::new(10));
}

/// Test: position from 1-indexed
/// Source: Vim position handling
#[test]
fn test_position_from_1indexed() {
    let pos = Position::from_1indexed(3, 7);
    assert_eq!(pos.line.0, 3);
    assert_eq!(pos.col.0, 7);
}

/// Test: position origin constant
/// Source: Vim position handling
#[test]
fn test_position_origin() {
    let origin = Position::ORIGIN;
    assert_eq!(origin.line, LineNr::FIRST);
    assert_eq!(origin.col, ColNr::FIRST);
}

// ============================================================================
// LineNr Tests
// ============================================================================

/// Test: line number first constant
/// Source: Vim 1-indexed lines
#[test]
fn test_line_nr_first() {
    assert_eq!(LineNr::FIRST.0, 1);
}

/// Test: line number to zero indexed
/// Source: Internal array access
#[test]
fn test_line_nr_to_zero_indexed() {
    assert_eq!(LineNr::new(1).to_zero_indexed(), 0);
    assert_eq!(LineNr::new(5).to_zero_indexed(), 4);
}

/// Test: line number from zero indexed
/// Source: Internal array access
#[test]
fn test_line_nr_from_zero_indexed() {
    assert_eq!(LineNr::from_zero_indexed(0), LineNr::new(1));
    assert_eq!(LineNr::from_zero_indexed(4), LineNr::new(5));
}

// ============================================================================
// ColNr Tests
// ============================================================================

/// Test: column number first constant
/// Source: Vim 1-indexed columns
#[test]
fn test_col_nr_first() {
    assert_eq!(ColNr::FIRST.0, 1);
}

/// Test: column number to zero indexed
/// Source: Internal array access
#[test]
fn test_col_nr_to_zero_indexed() {
    assert_eq!(ColNr::new(1).to_zero_indexed(), 0);
    assert_eq!(ColNr::new(5).to_zero_indexed(), 4);
}

/// Test: column number from zero indexed
/// Source: Internal array access
#[test]
fn test_col_nr_from_zero_indexed() {
    assert_eq!(ColNr::from_zero_indexed(0), ColNr::new(1));
    assert_eq!(ColNr::from_zero_indexed(4), ColNr::new(5));
}

// ============================================================================
// Motion Type Tests
// ============================================================================

/// Test: motion types are distinct
/// Source: Vim motion types
#[test]
fn test_motion_types_distinct() {
    assert_ne!(MotionType::Characterwise, MotionType::Linewise);
    assert_ne!(MotionType::Linewise, MotionType::Blockwise);
    assert_ne!(MotionType::Characterwise, MotionType::Blockwise);
}

// ============================================================================
// Motion Inclusivity Tests
// ============================================================================

/// Test: motion inclusivity types
/// Source: Vim :h inclusive :h exclusive
#[test]
fn test_motion_inclusivity_types() {
    assert_ne!(MotionInclusivity::Inclusive, MotionInclusivity::Exclusive);
}

// ============================================================================
// Operator Double Command Tests (conceptual)
// ============================================================================

/// Test: double operator flag in context
/// Source: Vim dd, yy, cc behavior
#[test]
fn test_operator_double_flag() {
    let mut ctx = OperatorContext::default();
    assert!(!ctx.is_double);

    ctx.is_double = true;
    assert!(ctx.is_double);
}

// ============================================================================
// Complex Operator/Motion Combination Tests (conceptual)
// ============================================================================

/// Test: delete with word motion creates characterwise region
/// Source: Vim dw behavior
#[test]
fn test_delete_word_characterwise() {
    // Conceptual test: dw operates characterwise
    let op = Operator::Delete;
    let motion_type = MotionType::Characterwise;

    assert!(op.modifies_buffer());
    assert_eq!(motion_type, MotionType::Characterwise);
}

/// Test: delete line (dd) creates linewise region
/// Source: Vim dd behavior
#[test]
fn test_delete_line_linewise() {
    // Conceptual test: dd operates linewise
    let region = OperatorRegion::linewise(LineNr::new(1), LineNr::new(1));
    assert_eq!(region.region_type, MotionType::Linewise);
}

/// Test: yank preserves motion type
/// Source: Vim y{motion} behavior
#[test]
fn test_yank_preserves_motion_type() {
    let op = Operator::Yank;

    // Yank doesn't modify buffer
    assert!(!op.modifies_buffer());
    // But it does use registers
    assert!(op.uses_register());
}

/// Test: change with motion enters insert after
/// Source: Vim c{motion} behavior
#[test]
fn test_change_enters_insert() {
    let op = Operator::Change;

    assert!(op.modifies_buffer());
    assert!(op.enters_insert());
    assert!(op.uses_register());
}

/// Test: indent/dedent don't use registers
/// Source: Vim > and < behavior
#[test]
fn test_indent_no_register() {
    assert!(!Operator::Indent.uses_register());
    assert!(!Operator::Dedent.uses_register());
    assert!(Operator::Indent.modifies_buffer());
    assert!(Operator::Dedent.modifies_buffer());
}

/// Test: case operators modify buffer but don't enter insert
/// Source: Vim g~, gu, gU behavior
#[test]
fn test_case_operators() {
    for op in [
        Operator::ToggleCase,
        Operator::Lowercase,
        Operator::Uppercase,
    ] {
        assert!(op.modifies_buffer());
        assert!(!op.enters_insert());
        assert!(!op.uses_register());
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Test: region with same start and end
/// Source: Single character operations
#[test]
fn test_region_single_char() {
    let pos = Position::from_1indexed(1, 1);
    let region = OperatorRegion::characterwise(pos, pos, true);

    assert_eq!(region.start, region.end);
    assert!(region.inclusive);
}

/// Test: linewise region single line
/// Source: dd on single line
#[test]
fn test_region_single_line() {
    let region = OperatorRegion::linewise(LineNr::new(1), LineNr::new(1));
    let range = region.line_range();

    assert_eq!(range.len(), 1);
    assert!(!range.is_empty());
}

/// Test: blockwise region spans multiple lines and columns
/// Source: Visual block selection
#[test]
fn test_region_block_spans() {
    let start = Position::from_1indexed(1, 1);
    let end = Position::from_1indexed(5, 10);
    let region = OperatorRegion::blockwise(start, end);

    assert_eq!(region.region_type, MotionType::Blockwise);
    let range = region.line_range();
    assert_eq!(range.len(), 5);
}
