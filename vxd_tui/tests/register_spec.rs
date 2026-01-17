//! Register tests ported from Neovim tests
//!
//! These tests verify register behavior including:
//! - Named registers (a-z)
//! - Numbered registers (0-9)
//! - Special registers (", *, +, etc.)
//! - Register types (charwise, linewise, blockwise)

mod common;

use common::TestHarness;
use vxd::registers::{Register, RegisterBank, RegisterContent, RegisterType};

/// Test: unnamed register is default
/// Source: put_spec.lua / registers_spec.lua
#[test]
fn test_unnamed_register_default() {
    let mut h = TestHarness::new();

    // Set unnamed register
    let _ = h
        .editor
        .registers
        .set(Register::Unnamed, RegisterContent::characterwise("hello"));

    let content = h.editor.registers.get(Register::Unnamed);
    assert!(content.is_some());
    assert_eq!(content.unwrap().text, vec!["hello"]);
}

/// Test: named registers a-z
/// Source: registers_spec.lua
#[test]
fn test_named_registers() {
    let mut h = TestHarness::new();

    // Test a few representative registers
    for c in ['a', 'm', 'z'] {
        let _ = h.editor.registers.set(
            Register::Named(c),
            RegisterContent::characterwise(format!("content_{}", c)),
        );

        let content = h.editor.registers.get(Register::Named(c));
        assert!(content.is_some());
        assert_eq!(content.unwrap().text, vec![format!("content_{}", c)]);
    }
}

/// Test: uppercase register appends
/// Source: put_spec.lua (uppercase appends to lowercase)
#[test]
fn test_uppercase_appends() {
    let mut h = TestHarness::new();

    // Set register 'a'
    let _ = h.editor.registers.set(
        Register::Named('a'),
        RegisterContent::characterwise("first"),
    );

    // Append with 'A'
    let _ = h
        .editor
        .registers
        .append(Register::Named('A'), RegisterContent::characterwise("second"));

    let content = h.editor.registers.get(Register::Named('a'));
    assert!(content.is_some());
    let c = content.unwrap();
    // Should have both pieces
    assert!(c.text.join("").contains("first"));
    assert!(c.text.join("").contains("second"));
}

/// Test: register 0 holds last yank
/// Source: put_spec.lua (register 0 is yank register)
#[test]
fn test_register_0_yank() {
    let mut h = TestHarness::new();

    // Simulate yank to register 0
    let _ = h.editor.registers.set(
        Register::Numbered(0),
        RegisterContent::characterwise("yanked text"),
    );

    let content = h.editor.registers.get(Register::Numbered(0));
    assert!(content.is_some());
    assert_eq!(content.unwrap().text, vec!["yanked text"]);
}

/// Test: numbered registers 1-9 hold delete history
/// Source: put_spec.lua (numbered registers)
#[test]
fn test_numbered_delete_history() {
    let mut h = TestHarness::new();

    // Push deletions into numbered registers
    for i in 1..=5 {
        h.editor
            .registers
            .push_delete(RegisterContent::linewise(vec![format!("deleted_{}", i)]));
    }

    // Register 1 should have most recent
    let content = h.editor.registers.get(Register::Numbered(1));
    assert!(content.is_some());
    assert_eq!(content.unwrap().text, vec!["deleted_5"]);

    // Earlier deletions in higher numbers
    let content = h.editor.registers.get(Register::Numbered(2));
    assert!(content.is_some());
    assert_eq!(content.unwrap().text, vec!["deleted_4"]);
}

/// Test: small delete register
/// Source: put_spec.lua (small delete register -)
#[test]
fn test_small_delete_register() {
    let mut h = TestHarness::new();

    // Small deletes (less than one line) go to register -
    let _ = h
        .editor
        .registers
        .set(Register::SmallDelete, RegisterContent::characterwise("x"));

    let content = h.editor.registers.get(Register::SmallDelete);
    assert!(content.is_some());
    assert_eq!(content.unwrap().text, vec!["x"]);
}

/// Test: black hole register discards
/// Source: put_spec.lua (black hole register _)
#[test]
fn test_black_hole_register() {
    let mut h = TestHarness::new();

    // Writing to black hole should work but reading returns nothing
    let _ = h.editor.registers.set(
        Register::BlackHole,
        RegisterContent::characterwise("discarded"),
    );

    let content = h.editor.registers.get(Register::BlackHole);
    // Black hole either returns None or empty
    assert!(content.is_none() || content.unwrap().is_empty());
}

/// Test: register types
/// Source: put_spec.lua (register type affects paste behavior)
#[test]
fn test_register_types() {
    let mut h = TestHarness::new();

    // Characterwise
    let _ = h
        .editor
        .registers
        .set(Register::Named('a'), RegisterContent::characterwise("char"));
    let c = h.editor.registers.get(Register::Named('a')).unwrap();
    assert_eq!(c.reg_type, RegisterType::Characterwise);

    // Linewise
    let _ = h.editor.registers.set(
        Register::Named('b'),
        RegisterContent::linewise(vec!["line".to_string()]),
    );
    let c = h.editor.registers.get(Register::Named('b')).unwrap();
    assert_eq!(c.reg_type, RegisterType::Linewise);

    // Blockwise
    let _ = h.editor.registers.set(
        Register::Named('c'),
        RegisterContent::blockwise(vec!["block1".to_string(), "block2".to_string()], 6),
    );
    let c = h.editor.registers.get(Register::Named('c')).unwrap();
    assert!(matches!(c.reg_type, RegisterType::Blockwise { .. }));
}

/// Test: expression register (read-only aspects)
/// Source: put_spec.lua (expression register =)
#[test]
fn test_expression_register_readonly() {
    let h = TestHarness::new();

    // Expression register is typically read-only in the sense that
    // you can't set it like other registers
    let content = h.editor.registers.get(Register::Expression);
    // Should be None or empty for unset expression
    assert!(content.is_none() || content.unwrap().is_empty());
}

/// Test: last inserted text register
/// Source: registers_spec.lua (register .)
#[test]
fn test_last_inserted_register() {
    let h = TestHarness::new();

    // Last inserted register is read-only
    // In a real implementation, this would be set automatically
    let content = h.editor.registers.get(Register::LastInserted);
    // May be None or have some value
    let _ = content;
}

/// Test: current file register
/// Source: registers_spec.lua (register %)
#[test]
fn test_current_file_register() {
    let h = TestHarness::new();

    // Current file register is read-only, returns filename
    // In our test environment, it might be empty or "unnamed"
    let content = h.editor.registers.get(Register::CurrentFile);
    // Just verify it doesn't panic
    let _ = content;
}

/// Test: alternate file register
/// Source: registers_spec.lua (register #)
#[test]
fn test_alternate_file_register() {
    let h = TestHarness::new();

    // Alternate file register
    let content = h.editor.registers.get(Register::AlternateFile);
    // Just verify it doesn't panic
    let _ = content;
}

/// Test: command register
/// Source: registers_spec.lua (register :)
#[test]
fn test_command_register() {
    let h = TestHarness::new();

    // Last command register is read-only
    let content = h.editor.registers.get(Register::LastCommand);
    // Just verify it doesn't panic
    let _ = content;
}

/// Test: search register
/// Source: registers_spec.lua (register /)
#[test]
fn test_search_register() {
    let h = TestHarness::new();

    // Last search pattern register is read-only
    let content = h.editor.registers.get(Register::LastSearch);
    // Just verify it doesn't panic
    let _ = content;
}

/// Test: clipboard registers (* and +)
/// Source: registers_spec.lua (system clipboard)
#[test]
fn test_clipboard_registers() {
    let mut h = TestHarness::new();

    // Selection register *
    let _ = h.editor.registers.set(
        Register::Selection,
        RegisterContent::characterwise("selected"),
    );

    let content = h.editor.registers.get(Register::Selection);
    assert!(content.is_some());

    // Clipboard register +
    let _ = h.editor.registers.set(
        Register::Clipboard,
        RegisterContent::characterwise("clipboard"),
    );

    let content = h.editor.registers.get(Register::Clipboard);
    assert!(content.is_some());
}

/// Test: multiline register content
/// Source: put_spec.lua (multiline content)
#[test]
fn test_multiline_register() {
    let mut h = TestHarness::new();

    let _ = h.editor.registers.set(
        Register::Named('a'),
        RegisterContent::linewise(vec![
            "line1".to_string(),
            "line2".to_string(),
            "line3".to_string(),
        ]),
    );

    let content = h.editor.registers.get(Register::Named('a')).unwrap();
    assert_eq!(content.text.len(), 3);
    assert_eq!(content.text[0], "line1");
    assert_eq!(content.text[1], "line2");
    assert_eq!(content.text[2], "line3");
}

/// Test: empty register
/// Source: edge case
#[test]
fn test_empty_register() {
    let h = TestHarness::new();

    // Unset register should return None
    let content = h.editor.registers.get(Register::Named('x'));
    assert!(content.is_none());
}

/// Test: register content as_string
/// Source: internal API test
#[test]
fn test_register_as_string() {
    // Characterwise
    let c = RegisterContent::characterwise("hello");
    assert_eq!(c.as_string(), "hello");

    // Linewise (should add newline)
    let c = RegisterContent::linewise(vec!["line1".to_string(), "line2".to_string()]);
    assert!(c.as_string().contains("line1"));
    assert!(c.as_string().contains("line2"));
}

/// Test: register content is_empty
/// Source: internal API test
#[test]
fn test_register_is_empty() {
    let c = RegisterContent::characterwise("");
    assert!(c.is_empty());

    let c = RegisterContent::characterwise("text");
    assert!(!c.is_empty());
}
