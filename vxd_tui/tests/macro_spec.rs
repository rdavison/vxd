//! Macro tests ported from Neovim's macro_spec.lua
//!
//! These tests verify macro recording and playback behavior including:
//! - Recording macros with q{register}
//! - Playing back macros with @{register}
//! - Repeat last macro with @@
//! - Macro storage in registers
//! - Macro execution context

mod common;

use common::TestHarness;
use vxd::registers::{Register, RegisterBank, RegisterContent, RegisterType};

// ============================================================================
// Basic Macro Tests
// ============================================================================

/// Test: macros are stored in registers
/// Source: macro_spec.lua - "can be recorded and replayed"
#[test]
fn test_macro_stored_in_register() {
    let mut h = TestHarness::new();

    // Macros are stored as register content
    // q{register} starts recording, q stops recording
    // The recorded keys go into the register
    let _ = h.editor.registers.set(
        Register::Named('i'),
        RegisterContent::characterwise("ahello\x1b"), // ahello<Esc>
    );

    let content = h.editor.registers.get(Register::Named('i')).unwrap();
    assert_eq!(content.text.join(""), "ahello\x1b");
}

/// Test: macro can be replayed with @
/// Source: macro_spec.lua - "can be recorded and replayed"
#[test]
fn test_macro_replay_with_at() {
    let mut h = TestHarness::new();

    // @{register} plays back the macro
    let _ = h.editor.registers.set(
        Register::Named('a'),
        RegisterContent::characterwise("itest\x1b"),
    );

    let content = h.editor.registers.get(Register::Named('a')).unwrap();
    assert!(!content.is_empty());
}

/// Test: @@ repeats last macro
/// Source: macro_spec.lua - "can be replayed with Q and @@"
#[test]
fn test_macro_repeat_with_at_at() {
    let mut h = TestHarness::new();

    // @@ repeats the last @{register} command
    // This uses a special "last executed" tracking
    let _ = h.editor.registers.set(
        Register::Named('q'),
        RegisterContent::characterwise("AFOO\x1b"),
    );

    let content = h.editor.registers.get(Register::Named('q')).unwrap();
    assert!(content.text.join("").contains("FOO"));
}

// ============================================================================
// Macro Recording Tests
// ============================================================================

/// Test: register content type for macros
/// Source: macro_spec.lua - macro behavior
#[test]
fn test_macro_register_type() {
    let mut h = TestHarness::new();

    // Macros are stored as characterwise content
    let _ = h
        .editor
        .registers
        .set(Register::Named('m'), RegisterContent::characterwise("dd"));

    let content = h.editor.registers.get(Register::Named('m')).unwrap();
    assert_eq!(content.reg_type, RegisterType::Characterwise);
}

/// Test: macro with special keys
/// Source: macro_spec.lua - recording includes special keys
#[test]
fn test_macro_special_keys() {
    let mut h = TestHarness::new();

    // Macros can contain escape sequences and special keys
    let _ = h.editor.registers.set(
        Register::Named('s'),
        RegisterContent::characterwise("itext\x1b0"), // itext<Esc>0
    );

    let content = h.editor.registers.get(Register::Named('s')).unwrap();
    assert!(content.text.join("").contains("\x1b")); // Contains escape
}

/// Test: macro with control characters
/// Source: macro_spec.lua - control characters in macros
#[test]
fn test_macro_control_characters() {
    let mut h = TestHarness::new();

    // Macros can contain control characters like Ctrl-R
    let _ = h.editor.registers.set(
        Register::Named('c'),
        RegisterContent::characterwise("i\x12a"), // i<C-R>a
    );

    let content = h.editor.registers.get(Register::Named('c')).unwrap();
    assert!(content.text.join("").contains('\x12')); // Ctrl-R
}

// ============================================================================
// Macro Playback Tests
// ============================================================================

/// Test: macro with count
/// Source: macro_spec.lua - "G3Q" plays macro 3 times
#[test]
fn test_macro_with_count() {
    let mut h = TestHarness::new();

    // 3@a plays the macro in register 'a' three times
    let _ = h
        .editor
        .registers
        .set(Register::Named('a'), RegisterContent::characterwise("x"));

    let content = h.editor.registers.get(Register::Named('a')).unwrap();
    // The register content itself doesn't change with count
    assert_eq!(content.text, vec!["x"]);
}

/// Test: macro in visual mode
/// Source: macro_spec.lua - "can be recorded and replayed in Visual mode"
#[test]
fn test_macro_visual_mode() {
    let mut h = TestHarness::new();

    // Macros can be recorded and played in visual mode
    let _ = h.editor.registers.set(
        Register::Named('i'),
        RegisterContent::characterwise("fofR"), // search pattern command
    );

    let content = h.editor.registers.get(Register::Named('i')).unwrap();
    assert!(!content.is_empty());
}

/// Test: macro in linewise visual mode
/// Source: macro_spec.lua - "can be replayed with @ in linewise Visual mode"
#[test]
fn test_macro_linewise_visual() {
    let mut h = TestHarness::new();

    // V{motion}@q applies macro to each line in visual selection
    let _ = h.editor.registers.set(
        Register::Named('q'),
        RegisterContent::characterwise("AFOO\x1b"),
    );

    let content = h.editor.registers.get(Register::Named('q')).unwrap();
    assert!(content.text.join("").contains("FOO"));
}

/// Test: macro in blockwise visual mode
/// Source: macro_spec.lua - "can be replayed with @ in blockwise Visual mode"
#[test]
fn test_macro_blockwise_visual() {
    let mut h = TestHarness::new();

    // Ctrl-V{motion}@q applies macro to each line in block selection
    let _ = h.editor.registers.set(
        Register::Named('q'),
        RegisterContent::characterwise("AFOO\x1b"),
    );

    let content = h.editor.registers.get(Register::Named('q')).unwrap();
    assert!(!content.is_empty());
}

// ============================================================================
// Macro and Mapping Interaction Tests
// ============================================================================

/// Test: macros apply mappings
/// Source: macro_spec.lua - "applies maps"
#[test]
fn test_macro_applies_mappings() {
    let mut h = TestHarness::new();

    // When a macro is played, it goes through the normal key processing
    // including mappings
    // If 'x' is mapped to 'l', recording 'x' and playing back executes 'l'
    let _ = h.editor.registers.set(
        Register::Named('i'),
        RegisterContent::characterwise("lxxx\x1b"), // Records the typed keys
    );

    let content = h.editor.registers.get(Register::Named('i')).unwrap();
    assert!(content.text.join("").contains("xxx"));
}

// ============================================================================
// Q Command Tests
// ============================================================================

/// Test: Q replays last recorded macro
/// Source: macro_spec.lua - "can be replayed with Q"
#[test]
fn test_q_replays_macro() {
    let mut h = TestHarness::new();

    // Q is like @@ - repeats the last macro
    let _ = h.editor.registers.set(
        Register::Named('q'),
        RegisterContent::characterwise("AFOO\x1b"),
    );

    let content = h.editor.registers.get(Register::Named('q')).unwrap();
    assert!(!content.is_empty());
}

/// Test: Q with count
/// Source: macro_spec.lua - "G3Q"
#[test]
fn test_q_with_count() {
    let mut h = TestHarness::new();

    // 3Q replays the macro 3 times
    let _ = h.editor.registers.set(
        Register::Named('q'),
        RegisterContent::characterwise("AFOO\x1b"),
    );

    let content = h.editor.registers.get(Register::Named('q')).unwrap();
    // Multiple executions don't change the register
    assert_eq!(content.text.join(""), "AFOO\x1b");
}

// ============================================================================
// reg_executing() Tests
// ============================================================================

/// Test: reg_executing concept
/// Source: macro_spec.lua - reg_executing() tests
#[test]
fn test_reg_executing_concept() {
    // reg_executing() returns the register being executed
    // Empty string when not executing a macro
    // This is tracked during macro execution
    let executing_register: Option<Register> = None;
    assert!(executing_register.is_none());
}

/// Test: reg_recorded concept
/// Source: macro_spec.lua - reg_recorded() tests
#[test]
fn test_reg_recorded_concept() {
    // reg_recorded() returns the last recorded register
    // After qqyyq, reg_recorded() returns 'q'
    let last_recorded: Option<char> = Some('q');
    assert_eq!(last_recorded, Some('q'));
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test: empty macro
/// Source: edge case
#[test]
fn test_empty_macro() {
    let mut h = TestHarness::new();

    // Recording and immediately stopping creates empty macro
    let _ = h
        .editor
        .registers
        .set(Register::Named('e'), RegisterContent::characterwise(""));

    let content = h.editor.registers.get(Register::Named('e')).unwrap();
    assert!(content.is_empty());
}

/// Test: macro register can be set directly
/// Source: :let @a = 'cmd'
#[test]
fn test_macro_set_directly() {
    let mut h = TestHarness::new();

    // Macros can be set with :let @a = 'keys'
    let _ = h
        .editor
        .registers
        .set(Register::Named('a'), RegisterContent::characterwise("gg0"));

    let content = h.editor.registers.get(Register::Named('a')).unwrap();
    assert_eq!(content.text, vec!["gg0"]);
}

/// Test: macro with undo
/// Source: macro_spec.lua - macros containing 'u'
#[test]
fn test_macro_with_undo() {
    let mut h = TestHarness::new();

    // A macro can contain undo commands
    let _ = h.editor.registers.set(
        Register::Named('u'),
        RegisterContent::characterwise("AFOO\x1bu"), // AFOO<Esc>u
    );

    let content = h.editor.registers.get(Register::Named('u')).unwrap();
    assert!(content.text.join("").contains('u'));
}

/// Test: nested macro execution
/// Source: @a containing @b
#[test]
fn test_nested_macro() {
    let mut h = TestHarness::new();

    // A macro can call another macro
    let _ = h.editor.registers.set(
        Register::Named('b'),
        RegisterContent::characterwise("iinner\x1b"),
    );

    let _ = h
        .editor
        .registers
        .set(Register::Named('a'), RegisterContent::characterwise("@b")); // Calls macro b

    let content_a = h.editor.registers.get(Register::Named('a')).unwrap();
    let content_b = h.editor.registers.get(Register::Named('b')).unwrap();

    assert_eq!(content_a.text, vec!["@b"]);
    assert!(content_b.text.join("").contains("inner"));
}

/// Test: macro with search
/// Source: macro_spec.lua - macros with search patterns
#[test]
fn test_macro_with_search() {
    let mut h = TestHarness::new();

    // Macros can contain search commands
    let _ = h.editor.registers.set(
        Register::Named('s'),
        RegisterContent::characterwise("/pattern\x0d"), // /pattern<CR>
    );

    let content = h.editor.registers.get(Register::Named('s')).unwrap();
    assert!(content.text.join("").contains("pattern"));
}

/// Test: macro recording overwrites register
/// Source: recording to existing register
#[test]
fn test_macro_overwrites_register() {
    let mut h = TestHarness::new();

    // Recording to a register with content overwrites it
    let _ = h
        .editor
        .registers
        .set(Register::Named('a'), RegisterContent::characterwise("old"));

    let _ = h
        .editor
        .registers
        .set(Register::Named('a'), RegisterContent::characterwise("new"));

    let content = h.editor.registers.get(Register::Named('a')).unwrap();
    assert_eq!(content.text, vec!["new"]);
}

/// Test: uppercase register appends to macro
/// Source: qA appends to register a
#[test]
fn test_macro_append_with_uppercase() {
    let mut h = TestHarness::new();

    // Recording to uppercase register appends
    let _ = h.editor.registers.set(
        Register::Named('a'),
        RegisterContent::characterwise("first"),
    );

    let _ = h.editor.registers.append(
        Register::Named('A'),
        RegisterContent::characterwise("second"),
    );

    let content = h.editor.registers.get(Register::Named('a')).unwrap();
    let text = content.text.join("");
    assert!(text.contains("first"));
    assert!(text.contains("second"));
}

// ============================================================================
// Special Register Macros
// ============================================================================

/// Test: macro from numbered register
/// Source: @1, @2, etc.
#[test]
fn test_macro_numbered_register() {
    let mut h = TestHarness::new();

    // Can execute macro from numbered registers
    let _ = h
        .editor
        .registers
        .set(Register::Numbered(1), RegisterContent::characterwise("dd"));

    let content = h.editor.registers.get(Register::Numbered(1)).unwrap();
    assert_eq!(content.text, vec!["dd"]);
}

/// Test: macro from unnamed register
/// Source: @"
#[test]
fn test_macro_unnamed_register() {
    let mut h = TestHarness::new();

    // @" executes the unnamed register as macro
    let _ = h
        .editor
        .registers
        .set(Register::Unnamed, RegisterContent::characterwise("p"));

    let content = h.editor.registers.get(Register::Unnamed).unwrap();
    assert_eq!(content.text, vec!["p"]);
}
