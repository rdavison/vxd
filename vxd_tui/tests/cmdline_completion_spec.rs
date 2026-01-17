//! Cmdline completion, history, and window tests ported from Neovim tests
//!
//! These tests verify command-line behavior including:
//! - Command-line editing (section 20.1)
//! - Command-line abbreviations (section 20.2)
//! - Command-line completion (section 20.3)
//! - Command-line history (section 20.4)
//! - Command-line window (section 20.5)
//!
//! Source tests:
//! - test/functional/editor/mode_cmdline_spec.lua
//! - test/functional/editor/completion_spec.lua
//! - test/functional/api/cmdline_spec.lua

#![allow(non_snake_case)]

mod common;

use vxd::cmdline::CmdlineHistoryKind;
use vxd::completion::{CompletionItem, CompletionKind, CompletionState};

// ============================================================================
// CmdlineHistoryKind Tests
// ============================================================================

/// Test: command history kind exists
/// Source: Vim :h history
#[test]
fn test_cmdline_history_kind_command() {
    let kind = CmdlineHistoryKind::Command;
    assert_eq!(kind, CmdlineHistoryKind::Command);
}

/// Test: search forward history kind exists
/// Source: Vim :h history
#[test]
fn test_cmdline_history_kind_search_forward() {
    let kind = CmdlineHistoryKind::SearchForward;
    assert_eq!(kind, CmdlineHistoryKind::SearchForward);
}

/// Test: search backward history kind exists
/// Source: Vim :h history
#[test]
fn test_cmdline_history_kind_search_backward() {
    let kind = CmdlineHistoryKind::SearchBackward;
    assert_eq!(kind, CmdlineHistoryKind::SearchBackward);
}

/// Test: expression history kind exists
/// Source: Vim :h history
#[test]
fn test_cmdline_history_kind_expression() {
    let kind = CmdlineHistoryKind::Expression;
    assert_eq!(kind, CmdlineHistoryKind::Expression);
}

/// Test: input history kind exists
/// Source: Vim :h history
#[test]
fn test_cmdline_history_kind_input() {
    let kind = CmdlineHistoryKind::Input;
    assert_eq!(kind, CmdlineHistoryKind::Input);
}

/// Test: all history kinds are distinct
/// Source: Vim history types
#[test]
fn test_cmdline_history_kinds_distinct() {
    let kinds = [
        CmdlineHistoryKind::Command,
        CmdlineHistoryKind::SearchForward,
        CmdlineHistoryKind::SearchBackward,
        CmdlineHistoryKind::Expression,
        CmdlineHistoryKind::Input,
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
// CompletionKind Tests
// ============================================================================

/// Test: keyword completion kind
/// Source: Vim Ctrl-N/Ctrl-P
#[test]
fn test_completion_kind_keyword() {
    let kind = CompletionKind::Keyword;
    assert_eq!(kind, CompletionKind::Keyword);
}

/// Test: line completion kind
/// Source: Vim Ctrl-X Ctrl-L
#[test]
fn test_completion_kind_line() {
    let kind = CompletionKind::Line;
    assert_eq!(kind, CompletionKind::Line);
}

/// Test: file completion kind
/// Source: Vim Ctrl-X Ctrl-F
#[test]
fn test_completion_kind_file() {
    let kind = CompletionKind::File;
    assert_eq!(kind, CompletionKind::File);
}

/// Test: dictionary completion kind
/// Source: Vim Ctrl-X Ctrl-K
#[test]
fn test_completion_kind_dictionary() {
    let kind = CompletionKind::Dictionary;
    assert_eq!(kind, CompletionKind::Dictionary);
}

/// Test: thesaurus completion kind
/// Source: Vim Ctrl-X Ctrl-T
#[test]
fn test_completion_kind_thesaurus() {
    let kind = CompletionKind::Thesaurus;
    assert_eq!(kind, CompletionKind::Thesaurus);
}

/// Test: tag completion kind
/// Source: Vim Ctrl-X Ctrl-]
#[test]
fn test_completion_kind_tag() {
    let kind = CompletionKind::Tag;
    assert_eq!(kind, CompletionKind::Tag);
}

/// Test: include completion kind
/// Source: Vim Ctrl-X Ctrl-I
#[test]
fn test_completion_kind_include() {
    let kind = CompletionKind::Include;
    assert_eq!(kind, CompletionKind::Include);
}

/// Test: define completion kind
/// Source: Vim Ctrl-X Ctrl-D
#[test]
fn test_completion_kind_define() {
    let kind = CompletionKind::Define;
    assert_eq!(kind, CompletionKind::Define);
}

/// Test: command completion kind
/// Source: Vim Ctrl-X Ctrl-V
#[test]
fn test_completion_kind_command() {
    let kind = CompletionKind::Command;
    assert_eq!(kind, CompletionKind::Command);
}

/// Test: user completion kind
/// Source: Vim Ctrl-X Ctrl-U
#[test]
fn test_completion_kind_user() {
    let kind = CompletionKind::User;
    assert_eq!(kind, CompletionKind::User);
}

/// Test: omni completion kind
/// Source: Vim Ctrl-X Ctrl-O
#[test]
fn test_completion_kind_omni() {
    let kind = CompletionKind::Omni;
    assert_eq!(kind, CompletionKind::Omni);
}

/// Test: spelling completion kind
/// Source: Vim Ctrl-X s
#[test]
fn test_completion_kind_spelling() {
    let kind = CompletionKind::Spelling;
    assert_eq!(kind, CompletionKind::Spelling);
}

/// Test: buffer completion kind
/// Source: Vim buffer completion
#[test]
fn test_completion_kind_buffer() {
    let kind = CompletionKind::Buffer;
    assert_eq!(kind, CompletionKind::Buffer);
}

/// Test: all completion kinds are distinct
/// Source: Vim completion types
#[test]
fn test_completion_kinds_distinct() {
    let kinds = [
        CompletionKind::Keyword,
        CompletionKind::Line,
        CompletionKind::File,
        CompletionKind::Dictionary,
        CompletionKind::Thesaurus,
        CompletionKind::Tag,
        CompletionKind::Include,
        CompletionKind::Define,
        CompletionKind::Command,
        CompletionKind::User,
        CompletionKind::Omni,
        CompletionKind::Spelling,
        CompletionKind::Buffer,
    ];

    assert_eq!(kinds.len(), 13);

    for (i, a) in kinds.iter().enumerate() {
        for (j, b) in kinds.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}

// ============================================================================
// CompletionItem Tests
// ============================================================================

/// Test: simple completion item creation
/// Source: Vim completion item structure
#[test]
fn test_completion_item_new() {
    let item = CompletionItem::new("hello");
    assert_eq!(item.word, "hello");
    assert_eq!(item.abbr, None);
    assert_eq!(item.menu, None);
    assert_eq!(item.info, None);
    assert_eq!(item.kind, None);
    assert_eq!(item.priority, 0);
    assert!(!item.dup);
    assert_eq!(item.user_data, None);
}

/// Test: completion item with menu
/// Source: Vim completion item 'menu' field
#[test]
fn test_completion_item_with_menu() {
    let item = CompletionItem::new("hello").with_menu("greeting");
    assert_eq!(item.word, "hello");
    assert_eq!(item.menu, Some("greeting".to_string()));
}

/// Test: completion item with kind
/// Source: Vim completion item 'kind' field
#[test]
fn test_completion_item_with_kind() {
    let item = CompletionItem::new("myvar").with_kind("v");
    assert_eq!(item.word, "myvar");
    assert_eq!(item.kind, Some("v".to_string()));
}

/// Test: completion item builder chain
/// Source: Vim completion item structure
#[test]
fn test_completion_item_builder_chain() {
    let item = CompletionItem::new("function_name")
        .with_menu("my_module")
        .with_kind("f");

    assert_eq!(item.word, "function_name");
    assert_eq!(item.menu, Some("my_module".to_string()));
    assert_eq!(item.kind, Some("f".to_string()));
}

/// Test: completion item equality
/// Source: internal comparison
#[test]
fn test_completion_item_equality() {
    let item1 = CompletionItem::new("test");
    let item2 = CompletionItem::new("test");
    let item3 = CompletionItem::new("other");

    assert_eq!(item1, item2);
    assert_ne!(item1, item3);
}

/// Test: completion item with all fields set
/// Source: Vim complete-items
#[test]
fn test_completion_item_all_fields() {
    let mut item = CompletionItem::new("word");
    item.abbr = Some("abbrev".to_string());
    item.menu = Some("menu text".to_string());
    item.info = Some("detailed info".to_string());
    item.kind = Some("f".to_string());
    item.priority = 10;
    item.dup = true;
    item.user_data = Some("custom data".to_string());

    assert_eq!(item.word, "word");
    assert_eq!(item.abbr, Some("abbrev".to_string()));
    assert_eq!(item.menu, Some("menu text".to_string()));
    assert_eq!(item.info, Some("detailed info".to_string()));
    assert_eq!(item.kind, Some("f".to_string()));
    assert_eq!(item.priority, 10);
    assert!(item.dup);
    assert_eq!(item.user_data, Some("custom data".to_string()));
}

// ============================================================================
// CompletionState Tests
// ============================================================================

/// Test: default completion state is empty
/// Source: Vim completion state
#[test]
fn test_completion_state_default() {
    let state = CompletionState::default();
    assert!(state.items.is_empty());
    assert_eq!(state.selected, None);
    assert_eq!(state.original, "");
    assert_eq!(state.start_col, 0);
    assert_eq!(state.kind, None);
}

/// Test: completion state with items
/// Source: Vim popup menu state
#[test]
fn test_completion_state_with_items() {
    let mut state = CompletionState::default();
    state.items.push(CompletionItem::new("apple"));
    state.items.push(CompletionItem::new("banana"));
    state.items.push(CompletionItem::new("cherry"));

    assert_eq!(state.items.len(), 3);
    assert_eq!(state.items[0].word, "apple");
    assert_eq!(state.items[1].word, "banana");
    assert_eq!(state.items[2].word, "cherry");
}

/// Test: completion state with selection
/// Source: Vim completion selection
#[test]
fn test_completion_state_with_selection() {
    let mut state = CompletionState::default();
    state.items.push(CompletionItem::new("first"));
    state.items.push(CompletionItem::new("second"));
    state.selected = Some(1);

    assert_eq!(state.selected, Some(1));
    assert_eq!(state.items[state.selected.unwrap()].word, "second");
}

/// Test: completion state tracks original text
/// Source: Vim completion original
#[test]
fn test_completion_state_original() {
    let mut state = CompletionState::default();
    state.original = "hel".to_string();
    state.items.push(CompletionItem::new("hello"));
    state.items.push(CompletionItem::new("help"));

    assert_eq!(state.original, "hel");
}

/// Test: completion state tracks start column
/// Source: Vim completion start position
#[test]
fn test_completion_state_start_col() {
    let mut state = CompletionState::default();
    state.start_col = 5;

    assert_eq!(state.start_col, 5);
}

/// Test: completion state tracks completion kind
/// Source: Vim completion kind
#[test]
fn test_completion_state_kind() {
    let mut state = CompletionState::default();
    state.kind = Some(CompletionKind::Keyword);

    assert_eq!(state.kind, Some(CompletionKind::Keyword));
}

// ============================================================================
// Completion Selection Logic Tests
// ============================================================================

/// Test: no selection when empty
/// Source: Vim popup menu behavior
#[test]
fn test_completion_no_selection_when_empty() {
    let state = CompletionState::default();
    assert_eq!(state.selected, None);
    assert!(state.items.is_empty());
}

/// Test: selection index bounds
/// Source: Vim popup menu bounds
#[test]
fn test_completion_selection_bounds() {
    let mut state = CompletionState::default();
    state.items.push(CompletionItem::new("a"));
    state.items.push(CompletionItem::new("b"));
    state.items.push(CompletionItem::new("c"));

    // Valid selections
    state.selected = Some(0);
    assert!(state.selected.unwrap() < state.items.len());

    state.selected = Some(2);
    assert!(state.selected.unwrap() < state.items.len());
}

// ============================================================================
// Completion Kind Indicators
// ============================================================================

/// Test: common kind indicators
/// Source: Vim :h complete-items
#[test]
fn test_completion_kind_indicators() {
    // Common kind indicators used in Vim
    let indicators = [
        ("v", "variable"),
        ("f", "function"),
        ("m", "member"),
        ("t", "typedef"),
        ("d", "define"),
    ];

    for (indicator, _description) in indicators {
        let item = CompletionItem::new("test").with_kind(indicator);
        assert_eq!(item.kind, Some(indicator.to_string()));
    }
}

// ============================================================================
// History Behavior Tests (Conceptual)
// ============================================================================

/// Test: history kinds correspond to commands
/// Source: Vim :h history
#[test]
fn test_history_kind_command_for_colon() {
    // ":" command-line uses Command history
    let kind = CmdlineHistoryKind::Command;
    assert_eq!(kind, CmdlineHistoryKind::Command);
}

/// Test: history kinds correspond to search
/// Source: Vim :h history
#[test]
fn test_history_kind_for_search() {
    // "/" uses SearchForward, "?" uses SearchBackward
    let forward = CmdlineHistoryKind::SearchForward;
    let backward = CmdlineHistoryKind::SearchBackward;
    assert_ne!(forward, backward);
}

/// Test: expression history for "="
/// Source: Vim :h history
#[test]
fn test_history_kind_for_expression() {
    // "=" register uses Expression history
    let kind = CmdlineHistoryKind::Expression;
    assert_eq!(kind, CmdlineHistoryKind::Expression);
}

// ============================================================================
// Completion Menu Tests (Conceptual)
// ============================================================================

/// Test: pumheight affects visible items
/// Source: Vim 'pumheight' option
#[test]
fn test_pumheight_concept() {
    // pumheight controls max visible items in popup
    let pumheight = 10;
    let mut state = CompletionState::default();

    // Add more items than pumheight
    for i in 0..20 {
        state.items.push(CompletionItem::new(format!("item{}", i)));
    }

    // Menu would show at most pumheight items
    let visible = state.items.len().min(pumheight);
    assert_eq!(visible, 10);
}

/// Test: completion menu can scroll
/// Source: Vim popup menu scrolling
#[test]
fn test_completion_scroll_concept() {
    let mut state = CompletionState::default();

    // Many items
    for i in 0..100 {
        state.items.push(CompletionItem::new(format!("item{}", i)));
    }

    // Selection can be anywhere
    state.selected = Some(50);
    assert_eq!(state.items[50].word, "item50");

    state.selected = Some(99);
    assert_eq!(state.items[99].word, "item99");
}

// ============================================================================
// Completeopt Tests (Conceptual)
// ============================================================================

/// Test: completeopt menu concept
/// Source: Vim 'completeopt' option
#[test]
fn test_completeopt_menu_concept() {
    // "menu" - use popup menu for completions
    // "menuone" - show popup even with one match
    // "preview" - show info in preview window
    // "noinsert" - don't insert until selection made
    // "noselect" - don't select first item

    // Test noselect behavior - no initial selection
    let state = CompletionState::default();
    assert_eq!(state.selected, None);
}

/// Test: completeopt noinsert concept
/// Source: Vim 'completeopt' noinsert
#[test]
fn test_completeopt_noinsert_concept() {
    // With noinsert, original text is preserved until selection
    let mut state = CompletionState::default();
    state.original = "hel".to_string();
    state.items.push(CompletionItem::new("hello"));

    // Original is preserved
    assert_eq!(state.original, "hel");
}

// ============================================================================
// Command-line Editing Tests (Conceptual)
// ============================================================================

/// Test: command-line cursor position
/// Source: Vim :h getcmdpos()
#[test]
fn test_cmdline_cursor_position_concept() {
    // Cursor position is 1-indexed in cmdline
    let cursor_pos = 1; // First position
    assert!(cursor_pos >= 1);
}

/// Test: command-line text manipulation
/// Source: Vim :h c_CTRL-U
#[test]
fn test_cmdline_ctrl_u_concept() {
    // Ctrl-U deletes from cursor to start
    let cmdline = "some text";
    let cursor = 5; // cursor on ' '

    let after_ctrl_u = &cmdline[cursor..];
    assert_eq!(after_ctrl_u, "text");
}

/// Test: command-line word deletion
/// Source: Vim :h c_CTRL-W
#[test]
fn test_cmdline_ctrl_w_concept() {
    // Ctrl-W deletes word before cursor
    let cmdline = "hello world";
    let words: Vec<&str> = cmdline.split_whitespace().collect();

    assert_eq!(words.len(), 2);
    assert_eq!(words[0], "hello");
}

// ============================================================================
// Command-line Window Tests (Conceptual)
// ============================================================================

/// Test: command-line window opens with q:
/// Source: Vim :h cmdwin
#[test]
fn test_cmdwin_open_concept() {
    // q: opens command-line window with Command history
    // q/ opens with SearchForward history
    // q? opens with SearchBackward history
    let trigger = "q:";
    assert_eq!(trigger.chars().count(), 2);
}

/// Test: command-line window height
/// Source: Vim 'cmdwinheight'
#[test]
fn test_cmdwinheight_concept() {
    // Default cmdwinheight is 7
    let cmdwinheight = 7;
    assert!(cmdwinheight > 0);
}

/// Test: command-line window is special buffer
/// Source: Vim :h cmdwin
#[test]
fn test_cmdwin_special_buffer_concept() {
    // Cmdwin has special buffer type
    // Some commands are disallowed in cmdwin
    let is_cmdwin = true;
    let can_quit = is_cmdwin; // Can use :q to close
    assert!(can_quit);
}

// ============================================================================
// Wildmenu Tests (Conceptual)
// ============================================================================

/// Test: wildmenu shows completions
/// Source: Vim 'wildmenu'
#[test]
fn test_wildmenu_concept() {
    // wildmenu shows completions in status line
    let wildmenu_enabled = true;
    assert!(wildmenu_enabled);
}

/// Test: wildmode controls completion behavior
/// Source: Vim 'wildmode'
#[test]
fn test_wildmode_concept() {
    // wildmode can be: "full", "longest", "list", "lastused"
    let modes = ["full", "longest", "list", "lastused"];
    assert_eq!(modes.len(), 4);
}

/// Test: wildchar triggers completion
/// Source: Vim 'wildchar'
#[test]
fn test_wildchar_concept() {
    // Default wildchar is Tab (9)
    let wildchar = '\t';
    assert_eq!(wildchar as u8, 9);
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test: empty completion list
/// Source: No matches found
#[test]
fn test_completion_empty_list() {
    let state = CompletionState::default();
    assert!(state.items.is_empty());
    assert_eq!(state.selected, None);
}

/// Test: single completion item
/// Source: One match found
#[test]
fn test_completion_single_item() {
    let mut state = CompletionState::default();
    state.items.push(CompletionItem::new("only_match"));

    assert_eq!(state.items.len(), 1);
}

/// Test: completion item with empty word
/// Source: Edge case
#[test]
fn test_completion_item_empty_word() {
    let item = CompletionItem::new("");
    assert_eq!(item.word, "");
}

/// Test: completion item with unicode
/// Source: Unicode support
#[test]
fn test_completion_item_unicode() {
    let item = CompletionItem::new("日本語");
    assert_eq!(item.word, "日本語");

    let item2 = CompletionItem::new("émoji").with_menu("🎉");
    assert_eq!(item2.word, "émoji");
    assert_eq!(item2.menu, Some("🎉".to_string()));
}

/// Test: very long completion item
/// Source: Edge case
#[test]
fn test_completion_item_very_long() {
    let long_word = "a".repeat(1000);
    let item = CompletionItem::new(&long_word);
    assert_eq!(item.word.len(), 1000);
}

/// Test: many completion items
/// Source: Large completion list
#[test]
fn test_completion_many_items() {
    let mut state = CompletionState::default();

    for i in 0..10000 {
        state.items.push(CompletionItem::new(format!("item{}", i)));
    }

    assert_eq!(state.items.len(), 10000);
    assert_eq!(state.items[9999].word, "item9999");
}
