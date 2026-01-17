//! Test copying text between files (buffers).
//!
//! Covers usr_07.5 "Copy text between files".

mod common;
use common::TestHarness;
use vxd::registers::{Register, RegisterContent, RegisterType, RegisterBank};
use vxd::buffer::BufferManager;

#[test]
fn test_copy_paste_between_buffers() {
    let mut h = TestHarness::new();

    // 1. Setup first buffer with content
    h.set_lines(&["line 1", "line 2", "line 3"]);

    // 2. Yank "line 2" into register 'a'
    // Move to line 2
    h.set_cursor(2, 0);
    // Simulate "yy" to yank line 2.
    // Since TestHarness.feed doesn't fully simulate complex ops yet (it might, but let's be safe),
    // we can manually set the register to simulate the yank, OR try to use feed if implemented.
    // Looking at common/mod.rs, 'y' in visual mode is implemented. 'yy' in normal mode might not be.
    // Let's check common/mod.rs again.
    // It has: Key::Char('y') => { self.editor.visual_yank().unwrap(); } in process_visual_key.
    // It does NOT have 'y' or 'yy' in process_normal_key.
    
    // So we will simulate the yank by manually setting the register, 
    // effectively assuming the yank logic works (tested elsewhere) and testing the cross-buffer persistence.
    // OR we can use visual mode yank which IS implemented in harness.
    
    // Let's use visual mode yank to be more "integration-test" like.
    h.feed("V"); // Visual Line mode
    h.feed("y"); // Yank
    
    // Verify register '0' (default yank register) or Unnamed has the content
    let content = h.editor.registers.get(Register::Unnamed).unwrap();
    assert_eq!(content.text, vec!["line 2"]);
    
    // 3. Create a new buffer and switch to it
    let buf2 = h.editor.buffers.create_named("file2.txt").unwrap();
    h.editor.buffers.set_current(buf2).unwrap();
    h.editor.sync_cursor_with_buffer();
    
    // Verify we are in the new buffer (should be empty/one empty line)
    assert_eq!(h.get_lines(), vec![""]);
    
    // 4. Paste content
    // We can use 'p'
    h.feed("p");
    
    // 5. Verify content in new buffer
    // Original empty line + pasted line (since p pastes after cursor)
    // If buffer was empty ([""]), cursor is at (1,0). 
    // Linewise paste after should result in ["", "line 2"].
    // Wait, if buffer has one empty line, and we paste linewise...
    // Let's see behavior.
    assert_eq!(h.get_lines(), vec!["", "line 2"]);
    
    // Clean up or further checks?
}

#[test]
fn test_yank_in_one_paste_in_another_named_register() {
    let mut h = TestHarness::new();

    // Buffer 1
    h.set_lines(&["content in buffer 1"]);
    
    // Yank to register 'a'
    // Manual register set to ensure we are testing the cross-buffer aspect specifically
    let _ = h.editor.registers.set(
        Register::Named('a'), 
        RegisterContent::linewise(vec!["content in buffer 1".to_string()])
    );

    // Switch to Buffer 2
    let buf2 = h.editor.buffers.create_named("file2.txt").unwrap();
    h.editor.buffers.set_current(buf2).unwrap();
    h.editor.sync_cursor_with_buffer();
    
    // Paste from 'a'
    // Harness doesn't support "ap (quote a p). 
    // But we can manually invoke put_register if needed, or implement quote parsing in harness.
    // Harness 'feed' is simple.
    // Let's use editor.put_register directly for this specific test to be precise.
    
    h.editor.put_register(Register::Named('a'), true).unwrap();
    
    let lines = h.get_lines();
    assert!(lines.contains(&"content in buffer 1".to_string()));
}
