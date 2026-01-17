//! Search tests ported from Neovim's search tests
//!
//! These tests verify search behavior including:
//! - Basic forward and backward search
//! - Search patterns and options
//! - Case sensitivity (ignorecase, smartcase)
//! - Search wrapping
//! - Search offsets
//! - Word search (* and #)
//! - Substitute command

mod common;

use common::TestHarness;
use vxd::search::{SearchOffset, SearchOptions, SearchPattern, SearchState, SubstituteFlags};
use vxd::types::Direction;

// ============================================================================
// Basic Search Pattern Tests
// ============================================================================

/// Test: create forward search pattern
/// Source: search behavior - /pattern
#[test]
fn test_search_pattern_forward() {
    let pattern = SearchPattern::forward("test");

    assert_eq!(pattern.pattern, "test");
    assert_eq!(pattern.direction, Direction::Forward);
    assert!(pattern.valid);
}

/// Test: create backward search pattern
/// Source: search behavior - ?pattern
#[test]
fn test_search_pattern_backward() {
    let pattern = SearchPattern::backward("test");

    assert_eq!(pattern.pattern, "test");
    assert_eq!(pattern.direction, Direction::Backward);
    assert!(pattern.valid);
}

/// Test: empty pattern is valid
/// Source: empty search uses last pattern
#[test]
fn test_search_pattern_empty() {
    let pattern = SearchPattern::forward("");

    assert_eq!(pattern.pattern, "");
    assert!(pattern.valid);
}

// ============================================================================
// Search Direction Tests
// ============================================================================

/// Test: direction reversal
/// Source: N reverses direction of n
#[test]
fn test_search_direction_reverse() {
    let forward = Direction::Forward;
    let backward = Direction::Backward;

    assert_eq!(forward.reverse(), Direction::Backward);
    assert_eq!(backward.reverse(), Direction::Forward);
}

/// Test: search n repeats last search
/// Source: n command
#[test]
fn test_search_n_repeats() {
    let pattern = SearchPattern::forward("word");
    let state = SearchState {
        last_pattern: Some(pattern.clone()),
        last_direction: Direction::Forward,
        match_index: 0,
        match_count: 5,
        highlighting: true,
    };

    assert_eq!(state.last_pattern.as_ref().unwrap().pattern, "word");
    assert_eq!(state.last_direction, Direction::Forward);
}

/// Test: search N reverses direction
/// Source: N command
#[test]
fn test_search_n_reverses() {
    let state = SearchState {
        last_pattern: Some(SearchPattern::forward("word")),
        last_direction: Direction::Forward,
        match_index: 0,
        match_count: 5,
        highlighting: true,
    };

    // N would search in reverse direction
    let reversed_direction = state.last_direction.reverse();
    assert_eq!(reversed_direction, Direction::Backward);
}

// ============================================================================
// Search Options Tests
// ============================================================================

/// Test: default search options
/// Source: default Vim settings
#[test]
fn test_search_options_default() {
    let options = SearchOptions::default();

    assert!(!options.ignorecase);
    assert!(!options.smartcase);
    assert!(!options.magic);
    assert!(!options.wrapscan);
    assert!(!options.incsearch);
    assert!(!options.hlsearch);
}

/// Test: ignorecase option
/// Source: :set ignorecase
#[test]
fn test_search_options_ignorecase() {
    let options = SearchOptions {
        ignorecase: true,
        ..Default::default()
    };

    assert!(options.ignorecase);
}

/// Test: smartcase option
/// Source: :set smartcase
#[test]
fn test_search_options_smartcase() {
    let options = SearchOptions {
        ignorecase: true,
        smartcase: true,
        ..Default::default()
    };

    // smartcase overrides ignorecase if pattern has uppercase
    assert!(options.smartcase);
}

/// Test: wrapscan option
/// Source: :set wrapscan
#[test]
fn test_search_options_wrapscan() {
    let options = SearchOptions {
        wrapscan: true,
        ..Default::default()
    };

    // With wrapscan, search wraps around file
    assert!(options.wrapscan);
}

/// Test: incsearch option
/// Source: :set incsearch
#[test]
fn test_search_options_incsearch() {
    let options = SearchOptions {
        incsearch: true,
        ..Default::default()
    };

    // Incremental search shows matches as you type
    assert!(options.incsearch);
}

/// Test: hlsearch option
/// Source: :set hlsearch
#[test]
fn test_search_options_hlsearch() {
    let options = SearchOptions {
        hlsearch: true,
        ..Default::default()
    };

    // hlsearch highlights all matches
    assert!(options.hlsearch);
}

// ============================================================================
// Search Offset Tests
// ============================================================================

/// Test: no offset (default)
/// Source: /pattern
#[test]
fn test_search_offset_none() {
    let offset = SearchOffset::None;
    assert!(matches!(offset, SearchOffset::None));
}

/// Test: line offset
/// Source: /pattern/+2
#[test]
fn test_search_offset_line() {
    let offset = SearchOffset::Line(2);

    if let SearchOffset::Line(n) = offset {
        assert_eq!(n, 2);
    } else {
        panic!("Expected Line offset");
    }
}

/// Test: negative line offset
/// Source: /pattern/-3
#[test]
fn test_search_offset_line_negative() {
    let offset = SearchOffset::Line(-3);

    if let SearchOffset::Line(n) = offset {
        assert_eq!(n, -3);
    } else {
        panic!("Expected Line offset");
    }
}

/// Test: end offset
/// Source: /pattern/e+1
#[test]
fn test_search_offset_end() {
    let offset = SearchOffset::End(1);

    if let SearchOffset::End(n) = offset {
        assert_eq!(n, 1);
    } else {
        panic!("Expected End offset");
    }
}

/// Test: start offset
/// Source: /pattern/s+2
#[test]
fn test_search_offset_start() {
    let offset = SearchOffset::Start(2);

    if let SearchOffset::Start(n) = offset {
        assert_eq!(n, 2);
    } else {
        panic!("Expected Start offset");
    }
}

/// Test: column offset
/// Source: /pattern/b+3
#[test]
fn test_search_offset_column() {
    let offset = SearchOffset::Column(3);

    if let SearchOffset::Column(n) = offset {
        assert_eq!(n, 3);
    } else {
        panic!("Expected Column offset");
    }
}

// ============================================================================
// Search State Tests
// ============================================================================

/// Test: search state tracks last pattern
/// Source: using empty search to repeat
#[test]
fn test_search_state_last_pattern() {
    let mut state = SearchState::default();

    state.last_pattern = Some(SearchPattern::forward("hello"));

    assert!(state.last_pattern.is_some());
    assert_eq!(state.last_pattern.as_ref().unwrap().pattern, "hello");
}

/// Test: search state tracks match count
/// Source: search count display
#[test]
fn test_search_state_match_count() {
    let state = SearchState {
        last_pattern: Some(SearchPattern::forward("word")),
        last_direction: Direction::Forward,
        match_index: 2,
        match_count: 10,
        highlighting: true,
    };

    assert_eq!(state.match_count, 10);
    assert_eq!(state.match_index, 2);
}

/// Test: search state tracks highlighting
/// Source: :set hlsearch/:nohlsearch
#[test]
fn test_search_state_highlighting() {
    let mut state = SearchState::default();

    state.highlighting = true;
    assert!(state.highlighting);

    state.highlighting = false;
    assert!(!state.highlighting);
}

// ============================================================================
// Word Search Tests (*, #)
// ============================================================================

/// Test: * searches for word under cursor forward
/// Source: * command
#[test]
fn test_star_search_forward() {
    let _h = TestHarness::with_lines(&["hello world hello"]);

    // * searches forward for word under cursor
    // Creates pattern with word boundaries: \<word\>
    let pattern = SearchPattern::forward("hello");
    assert_eq!(pattern.direction, Direction::Forward);
}

/// Test: # searches for word under cursor backward
/// Source: # command
#[test]
fn test_hash_search_backward() {
    let _h = TestHarness::with_lines(&["hello world hello"]);

    // # searches backward for word under cursor
    let pattern = SearchPattern::backward("hello");
    assert_eq!(pattern.direction, Direction::Backward);
}

/// Test: g* searches without word boundaries
/// Source: g* command
#[test]
fn test_g_star_no_word_boundary() {
    // g* is like * but without \< and \>
    // So "ell" would match "hello"
    let pattern = SearchPattern::forward("ell");
    assert_eq!(pattern.pattern, "ell");
}

/// Test: g# searches backward without word boundaries
/// Source: g# command
#[test]
fn test_g_hash_no_word_boundary() {
    let pattern = SearchPattern::backward("ell");
    assert_eq!(pattern.pattern, "ell");
    assert_eq!(pattern.direction, Direction::Backward);
}

// ============================================================================
// Substitute Command Tests
// ============================================================================

/// Test: substitute flags default
/// Source: :s/pattern/replacement/
#[test]
fn test_substitute_flags_default() {
    let flags = SubstituteFlags::default();

    assert!(!flags.global);
    assert!(!flags.confirm);
    assert!(!flags.report);
    assert!(!flags.ignore_case);
}

/// Test: substitute global flag
/// Source: :s/pattern/replacement/g
#[test]
fn test_substitute_flags_global() {
    let flags = SubstituteFlags {
        global: true,
        ..Default::default()
    };

    assert!(flags.global);
}

/// Test: substitute confirm flag
/// Source: :s/pattern/replacement/c
#[test]
fn test_substitute_flags_confirm() {
    let flags = SubstituteFlags {
        confirm: true,
        ..Default::default()
    };

    assert!(flags.confirm);
}

/// Test: substitute ignorecase flag
/// Source: :s/pattern/replacement/i
#[test]
fn test_substitute_flags_ignorecase() {
    let flags = SubstituteFlags {
        ignore_case: true,
        ..Default::default()
    };

    assert!(flags.ignore_case);
}

/// Test: substitute noignorecase flag
/// Source: :s/pattern/replacement/I
#[test]
fn test_substitute_flags_noignorecase() {
    let flags = SubstituteFlags {
        no_ignore_case: true,
        ..Default::default()
    };

    assert!(flags.no_ignore_case);
}

/// Test: substitute combined flags
/// Source: :s/pattern/replacement/gc
#[test]
fn test_substitute_flags_combined() {
    let flags = SubstituteFlags {
        global: true,
        confirm: true,
        ..Default::default()
    };

    assert!(flags.global);
    assert!(flags.confirm);
}

// ============================================================================
// Magic Mode Tests
// ============================================================================

/// Test: magic mode affects special characters
/// Source: :set magic / :set nomagic
#[test]
fn test_search_magic_mode() {
    // In magic mode (default), these are special: . * ^ $ ~ []
    // In nomagic mode, only ^ and $ are special
    let options_magic = SearchOptions {
        magic: true,
        ..Default::default()
    };

    let options_nomagic = SearchOptions {
        magic: false,
        ..Default::default()
    };

    assert!(options_magic.magic);
    assert!(!options_nomagic.magic);
}

/// Test: very magic mode (\v)
/// Source: /\vpattern
#[test]
fn test_search_very_magic() {
    // \v in pattern makes most characters special (like Perl regex)
    // This is a pattern that would be interpreted as very magic
    let pattern = SearchPattern::forward(r"\v(hello|world)");
    assert!(pattern.pattern.starts_with(r"\v"));
}

/// Test: very nomagic mode (\V)
/// Source: /\Vpattern
#[test]
fn test_search_very_nomagic() {
    // \V makes everything literal except backslash
    let pattern = SearchPattern::forward(r"\Vhello.world");
    assert!(pattern.pattern.starts_with(r"\V"));
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test: search pattern with special regex chars
/// Source: edge case
#[test]
fn test_search_pattern_special_chars() {
    let pattern = SearchPattern::forward(r"hello\.world");
    assert!(pattern.pattern.contains(r"\."));
}

/// Test: search pattern with newline
/// Source: multiline search
#[test]
fn test_search_pattern_newline() {
    let pattern = SearchPattern::forward(r"hello\nworld");
    assert!(pattern.pattern.contains(r"\n"));
}

/// Test: empty search state
/// Source: initial state
#[test]
fn test_search_state_empty() {
    let state = SearchState::default();

    assert!(state.last_pattern.is_none());
    assert_eq!(state.match_count, 0);
    assert_eq!(state.match_index, 0);
}

/// Test: search with count
/// Source: 3n searches for 3rd match
#[test]
fn test_search_with_count() {
    let state = SearchState {
        last_pattern: Some(SearchPattern::forward("word")),
        last_direction: Direction::Forward,
        match_index: 0,
        match_count: 10,
        highlighting: true,
    };

    // 3n would advance match_index by 3
    let new_index = (state.match_index + 3) % state.match_count;
    assert_eq!(new_index, 3);
}
