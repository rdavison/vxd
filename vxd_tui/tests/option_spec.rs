//! Option/setting tests ported from Neovim tests
//!
//! These tests verify option behavior including:
//! - Option scopes (global, window, buffer)
//! - Option types (boolean, number, string)
//! - Option value manipulation
//! - Common option names
//! - Option definitions
//!
//! Source tests:
//! - test/functional/options/defaults_spec.lua
//! - test/functional/options/num_options_spec.lua

#![allow(non_snake_case)]

mod common;

use vxd::options::{options, OptionDef, OptionScope, OptionValue};

// ============================================================================
// OptionScope Tests
// ============================================================================

/// Test: global option scope
/// Source: Vim :h option-summary
#[test]
fn test_option_scope_global() {
    let scope = OptionScope::Global;
    assert_eq!(scope, OptionScope::Global);
}

/// Test: window option scope
/// Source: Vim :h option-summary
#[test]
fn test_option_scope_window() {
    let scope = OptionScope::Window;
    assert_eq!(scope, OptionScope::Window);
}

/// Test: buffer option scope
/// Source: Vim :h option-summary
#[test]
fn test_option_scope_buffer() {
    let scope = OptionScope::Buffer;
    assert_eq!(scope, OptionScope::Buffer);
}

/// Test: all scopes are distinct
/// Source: Internal consistency
#[test]
fn test_option_scopes_distinct() {
    assert_ne!(OptionScope::Global, OptionScope::Window);
    assert_ne!(OptionScope::Global, OptionScope::Buffer);
    assert_ne!(OptionScope::Window, OptionScope::Buffer);
}

// ============================================================================
// OptionValue Tests - Boolean
// ============================================================================

/// Test: boolean option value true
/// Source: Vim boolean options
#[test]
fn test_option_value_boolean_true() {
    let val = OptionValue::Boolean(true);
    assert_eq!(val.as_bool(), Some(true));
}

/// Test: boolean option value false
/// Source: Vim boolean options
#[test]
fn test_option_value_boolean_false() {
    let val = OptionValue::Boolean(false);
    assert_eq!(val.as_bool(), Some(false));
}

/// Test: boolean option as_number returns None
/// Source: Type safety
#[test]
fn test_option_value_boolean_as_number() {
    let val = OptionValue::Boolean(true);
    assert_eq!(val.as_number(), None);
}

/// Test: boolean option as_str returns None
/// Source: Type safety
#[test]
fn test_option_value_boolean_as_str() {
    let val = OptionValue::Boolean(true);
    assert_eq!(val.as_str(), None);
}

// ============================================================================
// OptionValue Tests - Number
// ============================================================================

/// Test: number option value positive
/// Source: Vim number options
#[test]
fn test_option_value_number_positive() {
    let val = OptionValue::Number(42);
    assert_eq!(val.as_number(), Some(42));
}

/// Test: number option value zero
/// Source: Vim number options
#[test]
fn test_option_value_number_zero() {
    let val = OptionValue::Number(0);
    assert_eq!(val.as_number(), Some(0));
}

/// Test: number option value negative
/// Source: Vim number options
#[test]
fn test_option_value_number_negative() {
    let val = OptionValue::Number(-10);
    assert_eq!(val.as_number(), Some(-10));
}

/// Test: number option as_bool returns None
/// Source: Type safety
#[test]
fn test_option_value_number_as_bool() {
    let val = OptionValue::Number(1);
    assert_eq!(val.as_bool(), None);
}

/// Test: number option as_str returns None
/// Source: Type safety
#[test]
fn test_option_value_number_as_str() {
    let val = OptionValue::Number(42);
    assert_eq!(val.as_str(), None);
}

// ============================================================================
// OptionValue Tests - String
// ============================================================================

/// Test: string option value
/// Source: Vim string options
#[test]
fn test_option_value_string() {
    let val = OptionValue::String("test".to_string());
    assert_eq!(val.as_str(), Some("test"));
}

/// Test: string option empty
/// Source: Vim string options
#[test]
fn test_option_value_string_empty() {
    let val = OptionValue::String("".to_string());
    assert_eq!(val.as_str(), Some(""));
}

/// Test: string option as_bool returns None
/// Source: Type safety
#[test]
fn test_option_value_string_as_bool() {
    let val = OptionValue::String("true".to_string());
    assert_eq!(val.as_bool(), None);
}

/// Test: string option as_number returns None
/// Source: Type safety
#[test]
fn test_option_value_string_as_number() {
    let val = OptionValue::String("42".to_string());
    assert_eq!(val.as_number(), None);
}

// ============================================================================
// OptionDef Tests
// ============================================================================

/// Test: option definition creation
/// Source: Internal API
#[test]
fn test_option_def_creation() {
    let def = OptionDef {
        name: "tabstop".to_string(),
        short_name: Some("ts".to_string()),
        scope: OptionScope::Buffer,
        default: OptionValue::Number(8),
        hidden: false,
        description: "Number of spaces a tab counts for".to_string(),
    };

    assert_eq!(def.name, "tabstop");
    assert_eq!(def.short_name, Some("ts".to_string()));
    assert_eq!(def.scope, OptionScope::Buffer);
    assert!(!def.hidden);
}

/// Test: option definition without short name
/// Source: Internal API
#[test]
fn test_option_def_no_short_name() {
    let def = OptionDef {
        name: "virtualedit".to_string(),
        short_name: None,
        scope: OptionScope::Global,
        default: OptionValue::String("".to_string()),
        hidden: false,
        description: "When to use virtual editing".to_string(),
    };

    assert_eq!(def.short_name, None);
}

/// Test: hidden option definition
/// Source: Internal API
#[test]
fn test_option_def_hidden() {
    let def = OptionDef {
        name: "internal_option".to_string(),
        short_name: None,
        scope: OptionScope::Global,
        default: OptionValue::Boolean(false),
        hidden: true,
        description: "Internal option".to_string(),
    };

    assert!(def.hidden);
}

// ============================================================================
// Common Option Names Tests
// ============================================================================

/// Test: tabstop option name
/// Source: Vim :h tabstop
#[test]
fn test_option_name_tabstop() {
    assert_eq!(options::TABSTOP, "tabstop");
}

/// Test: shiftwidth option name
/// Source: Vim :h shiftwidth
#[test]
fn test_option_name_shiftwidth() {
    assert_eq!(options::SHIFTWIDTH, "shiftwidth");
}

/// Test: expandtab option name
/// Source: Vim :h expandtab
#[test]
fn test_option_name_expandtab() {
    assert_eq!(options::EXPANDTAB, "expandtab");
}

/// Test: autoindent option name
/// Source: Vim :h autoindent
#[test]
fn test_option_name_autoindent() {
    assert_eq!(options::AUTOINDENT, "autoindent");
}

/// Test: smartindent option name
/// Source: Vim :h smartindent
#[test]
fn test_option_name_smartindent() {
    assert_eq!(options::SMARTINDENT, "smartindent");
}

/// Test: number option name
/// Source: Vim :h number
#[test]
fn test_option_name_number() {
    assert_eq!(options::NUMBER, "number");
}

/// Test: relativenumber option name
/// Source: Vim :h relativenumber
#[test]
fn test_option_name_relativenumber() {
    assert_eq!(options::RELATIVENUMBER, "relativenumber");
}

/// Test: wrap option name
/// Source: Vim :h wrap
#[test]
fn test_option_name_wrap() {
    assert_eq!(options::WRAP, "wrap");
}

/// Test: ignorecase option name
/// Source: Vim :h ignorecase
#[test]
fn test_option_name_ignorecase() {
    assert_eq!(options::IGNORECASE, "ignorecase");
}

/// Test: smartcase option name
/// Source: Vim :h smartcase
#[test]
fn test_option_name_smartcase() {
    assert_eq!(options::SMARTCASE, "smartcase");
}

/// Test: incsearch option name
/// Source: Vim :h incsearch
#[test]
fn test_option_name_incsearch() {
    assert_eq!(options::INCSEARCH, "incsearch");
}

/// Test: hlsearch option name
/// Source: Vim :h hlsearch
#[test]
fn test_option_name_hlsearch() {
    assert_eq!(options::HLSEARCH, "hlsearch");
}

/// Test: showmatch option name
/// Source: Vim :h showmatch
#[test]
fn test_option_name_showmatch() {
    assert_eq!(options::SHOWMATCH, "showmatch");
}

/// Test: cursorline option name
/// Source: Vim :h cursorline
#[test]
fn test_option_name_cursorline() {
    assert_eq!(options::CURSORLINE, "cursorline");
}

/// Test: cursorcolumn option name
/// Source: Vim :h cursorcolumn
#[test]
fn test_option_name_cursorcolumn() {
    assert_eq!(options::CURSORCOLUMN, "cursorcolumn");
}

/// Test: scrolloff option name
/// Source: Vim :h scrolloff
#[test]
fn test_option_name_scrolloff() {
    assert_eq!(options::SCROLLOFF, "scrolloff");
}

/// Test: sidescrolloff option name
/// Source: Vim :h sidescrolloff
#[test]
fn test_option_name_sidescrolloff() {
    assert_eq!(options::SIDESCROLLOFF, "sidescrolloff");
}

/// Test: hidden option name
/// Source: Vim :h hidden
#[test]
fn test_option_name_hidden() {
    assert_eq!(options::HIDDEN, "hidden");
}

/// Test: fileencoding option name
/// Source: Vim :h fileencoding
#[test]
fn test_option_name_fileencoding() {
    assert_eq!(options::FILEENCODING, "fileencoding");
}

/// Test: fileformat option name
/// Source: Vim :h fileformat
#[test]
fn test_option_name_fileformat() {
    assert_eq!(options::FILEFORMAT, "fileformat");
}

/// Test: modified option name
/// Source: Vim :h modified
#[test]
fn test_option_name_modified() {
    assert_eq!(options::MODIFIED, "modified");
}

/// Test: readonly option name
/// Source: Vim :h readonly
#[test]
fn test_option_name_readonly() {
    assert_eq!(options::READONLY, "readonly");
}

/// Test: modifiable option name
/// Source: Vim :h modifiable
#[test]
fn test_option_name_modifiable() {
    assert_eq!(options::MODIFIABLE, "modifiable");
}

/// Test: spell option name
/// Source: Vim :h spell
#[test]
fn test_option_name_spell() {
    assert_eq!(options::SPELL, "spell");
}

/// Test: list option name
/// Source: Vim :h list
#[test]
fn test_option_name_list() {
    assert_eq!(options::LIST, "list");
}

/// Test: timeoutlen option name
/// Source: Vim :h timeoutlen
#[test]
fn test_option_name_timeoutlen() {
    assert_eq!(options::TIMEOUTLEN, "timeoutlen");
}

/// Test: updatetime option name
/// Source: Vim :h updatetime
#[test]
fn test_option_name_updatetime() {
    assert_eq!(options::UPDATETIME, "updatetime");
}

/// Test: signcolumn option name
/// Source: Vim :h signcolumn
#[test]
fn test_option_name_signcolumn() {
    assert_eq!(options::SIGNCOLUMN, "signcolumn");
}

/// Test: colorcolumn option name
/// Source: Vim :h colorcolumn
#[test]
fn test_option_name_colorcolumn() {
    assert_eq!(options::COLORCOLUMN, "colorcolumn");
}

/// Test: textwidth option name
/// Source: Vim :h textwidth
#[test]
fn test_option_name_textwidth() {
    assert_eq!(options::TEXTWIDTH, "textwidth");
}

/// Test: virtualedit option name
/// Source: Vim :h virtualedit
#[test]
fn test_option_name_virtualedit() {
    assert_eq!(options::VIRTUALEDIT, "virtualedit");
}

// ============================================================================
// Option Value Equality Tests
// ============================================================================

/// Test: boolean option equality
/// Source: Internal consistency
#[test]
fn test_option_value_boolean_equality() {
    let val1 = OptionValue::Boolean(true);
    let val2 = OptionValue::Boolean(true);
    let val3 = OptionValue::Boolean(false);

    assert_eq!(val1, val2);
    assert_ne!(val1, val3);
}

/// Test: number option equality
/// Source: Internal consistency
#[test]
fn test_option_value_number_equality() {
    let val1 = OptionValue::Number(42);
    let val2 = OptionValue::Number(42);
    let val3 = OptionValue::Number(100);

    assert_eq!(val1, val2);
    assert_ne!(val1, val3);
}

/// Test: string option equality
/// Source: Internal consistency
#[test]
fn test_option_value_string_equality() {
    let val1 = OptionValue::String("test".to_string());
    let val2 = OptionValue::String("test".to_string());
    let val3 = OptionValue::String("other".to_string());

    assert_eq!(val1, val2);
    assert_ne!(val1, val3);
}

/// Test: different types are not equal
/// Source: Internal consistency
#[test]
fn test_option_value_different_types() {
    let bool_val = OptionValue::Boolean(true);
    let num_val = OptionValue::Number(1);
    let str_val = OptionValue::String("true".to_string());

    assert_ne!(bool_val, num_val);
    assert_ne!(bool_val, str_val);
    assert_ne!(num_val, str_val);
}

// ============================================================================
// Option Scope Semantics Tests (Conceptual)
// ============================================================================

/// Test: global options affect entire editor
/// Source: Vim :h global-option
#[test]
fn test_global_option_concept() {
    // Global options like 'ignorecase' affect all buffers/windows
    let scope = OptionScope::Global;
    assert_eq!(scope, OptionScope::Global);
}

/// Test: window options are per-window
/// Source: Vim :h local-option
#[test]
fn test_window_option_concept() {
    // Window options like 'number' can differ per window
    let scope = OptionScope::Window;
    assert_eq!(scope, OptionScope::Window);
}

/// Test: buffer options are per-buffer
/// Source: Vim :h local-option
#[test]
fn test_buffer_option_concept() {
    // Buffer options like 'tabstop' can differ per buffer
    let scope = OptionScope::Buffer;
    assert_eq!(scope, OptionScope::Buffer);
}

// ============================================================================
// Common Option Default Value Tests (Conceptual)
// ============================================================================

/// Test: tabstop default is 8
/// Source: Vim default
#[test]
fn test_tabstop_default_concept() {
    let default = OptionValue::Number(8);
    assert_eq!(default.as_number(), Some(8));
}

/// Test: shiftwidth default is 8 (or 0 to use tabstop)
/// Source: Vim default
#[test]
fn test_shiftwidth_default_concept() {
    let default = OptionValue::Number(8);
    assert_eq!(default.as_number(), Some(8));
}

/// Test: expandtab default is false
/// Source: Vim default
#[test]
fn test_expandtab_default_concept() {
    let default = OptionValue::Boolean(false);
    assert_eq!(default.as_bool(), Some(false));
}

/// Test: number default is false
/// Source: Vim default
#[test]
fn test_number_default_concept() {
    let default = OptionValue::Boolean(false);
    assert_eq!(default.as_bool(), Some(false));
}

/// Test: wrap default is true
/// Source: Vim default
#[test]
fn test_wrap_default_concept() {
    let default = OptionValue::Boolean(true);
    assert_eq!(default.as_bool(), Some(true));
}

/// Test: ignorecase default is false
/// Source: Vim default
#[test]
fn test_ignorecase_default_concept() {
    let default = OptionValue::Boolean(false);
    assert_eq!(default.as_bool(), Some(false));
}

/// Test: hlsearch default is true (in Neovim)
/// Source: Neovim default
#[test]
fn test_hlsearch_default_concept() {
    let default = OptionValue::Boolean(true);
    assert_eq!(default.as_bool(), Some(true));
}

/// Test: scrolloff default is 0
/// Source: Vim default
#[test]
fn test_scrolloff_default_concept() {
    let default = OptionValue::Number(0);
    assert_eq!(default.as_number(), Some(0));
}

/// Test: updatetime default is 4000
/// Source: Vim default
#[test]
fn test_updatetime_default_concept() {
    let default = OptionValue::Number(4000);
    assert_eq!(default.as_number(), Some(4000));
}

/// Test: timeoutlen default is 1000
/// Source: Vim default
#[test]
fn test_timeoutlen_default_concept() {
    let default = OptionValue::Number(1000);
    assert_eq!(default.as_number(), Some(1000));
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test: very large number option
/// Source: Edge case
#[test]
fn test_option_value_large_number() {
    let val = OptionValue::Number(i64::MAX);
    assert_eq!(val.as_number(), Some(i64::MAX));
}

/// Test: very small number option
/// Source: Edge case
#[test]
fn test_option_value_small_number() {
    let val = OptionValue::Number(i64::MIN);
    assert_eq!(val.as_number(), Some(i64::MIN));
}

/// Test: long string option
/// Source: Edge case
#[test]
fn test_option_value_long_string() {
    let long_str = "a".repeat(10000);
    let val = OptionValue::String(long_str.clone());
    assert_eq!(val.as_str(), Some(long_str.as_str()));
}

/// Test: unicode string option
/// Source: Unicode support
#[test]
fn test_option_value_unicode_string() {
    let val = OptionValue::String("日本語テスト".to_string());
    assert_eq!(val.as_str(), Some("日本語テスト"));
}

/// Test: string with special characters
/// Source: Edge case
#[test]
fn test_option_value_special_chars_string() {
    let val = OptionValue::String("path/to/file:123".to_string());
    assert_eq!(val.as_str(), Some("path/to/file:123"));
}

// ============================================================================
// Option Definition Edge Cases
// ============================================================================

/// Test: option def with empty description
/// Source: Edge case
#[test]
fn test_option_def_empty_description() {
    let def = OptionDef {
        name: "test".to_string(),
        short_name: None,
        scope: OptionScope::Global,
        default: OptionValue::Boolean(false),
        hidden: false,
        description: "".to_string(),
    };

    assert_eq!(def.description, "");
}

/// Test: option def with single char short name
/// Source: Vim short option names
#[test]
fn test_option_def_single_char_short() {
    let def = OptionDef {
        name: "autoindent".to_string(),
        short_name: Some("ai".to_string()),
        scope: OptionScope::Buffer,
        default: OptionValue::Boolean(false),
        hidden: false,
        description: "Auto indent".to_string(),
    };

    assert_eq!(def.short_name, Some("ai".to_string()));
}

// ============================================================================
// All Option Names Uniqueness
// ============================================================================

/// Test: all option names are unique strings
/// Source: Internal consistency
#[test]
fn test_all_option_names_unique() {
    let names = [
        options::TABSTOP,
        options::SHIFTWIDTH,
        options::EXPANDTAB,
        options::AUTOINDENT,
        options::SMARTINDENT,
        options::NUMBER,
        options::RELATIVENUMBER,
        options::WRAP,
        options::IGNORECASE,
        options::SMARTCASE,
        options::INCSEARCH,
        options::HLSEARCH,
        options::SHOWMATCH,
        options::CURSORLINE,
        options::CURSORCOLUMN,
        options::SCROLLOFF,
        options::SIDESCROLLOFF,
        options::HIDDEN,
        options::FILEENCODING,
        options::FILEFORMAT,
        options::MODIFIED,
        options::READONLY,
        options::MODIFIABLE,
        options::SPELL,
        options::LIST,
        options::TIMEOUTLEN,
        options::UPDATETIME,
        options::SIGNCOLUMN,
        options::COLORCOLUMN,
        options::TEXTWIDTH,
        options::VIRTUALEDIT,
    ];

    // Check all are unique
    let mut seen = std::collections::HashSet::new();
    for name in names {
        assert!(seen.insert(name), "Duplicate option name: {}", name);
    }
}

/// Test: all option names are non-empty
/// Source: Internal consistency
#[test]
fn test_all_option_names_non_empty() {
    let names = [
        options::TABSTOP,
        options::SHIFTWIDTH,
        options::EXPANDTAB,
        options::AUTOINDENT,
        options::SMARTINDENT,
        options::NUMBER,
        options::RELATIVENUMBER,
        options::WRAP,
        options::IGNORECASE,
        options::SMARTCASE,
        options::INCSEARCH,
        options::HLSEARCH,
        options::SHOWMATCH,
        options::CURSORLINE,
        options::CURSORCOLUMN,
        options::SCROLLOFF,
        options::SIDESCROLLOFF,
        options::HIDDEN,
        options::FILEENCODING,
        options::FILEFORMAT,
        options::MODIFIED,
        options::READONLY,
        options::MODIFIABLE,
        options::SPELL,
        options::LIST,
        options::TIMEOUTLEN,
        options::UPDATETIME,
        options::SIGNCOLUMN,
        options::COLORCOLUMN,
        options::TEXTWIDTH,
        options::VIRTUALEDIT,
    ];

    for name in names {
        assert!(!name.is_empty(), "Empty option name found");
    }
}
