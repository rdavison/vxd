mod common;

use vxd::mappings::MappingManager;
use vxd::modes::Mode;
use common::TestHarness;

#[test]
fn test_insert_mode_mapping_jk_to_esc() {
    let mut h = TestHarness::new();
    
    // Define a mapping: jk -> <Esc> in Insert mode
    h.editor.mappings.add(Mode::Insert, "jk", "<Esc>", false).unwrap();

    // Enter insert mode
    h.feed("i");
    
    // Type 'hello'
    h.feed("hello");
    
    // Type 'jk'
    h.feed("jk");
    
    // Should be in Normal mode now
    assert!(matches!(h.mode(), Mode::Normal));
    
    // Buffer should contain "hello" (jk should not be inserted)
    assert_eq!(h.content(), "hello");
}

#[test]
fn test_normal_mode_mapping_H_to_l() {
    let mut h = TestHarness::new();
    // Map 'H' to 'l' (move right)
    h.editor.mappings.add(Mode::Normal, "H", "l", false).unwrap();
    h.set_lines(&["hello"]);
    
    // Cursor at 0
    assert_cursor!(h, 1, 0);
    
    // Press 'H'
    h.feed("H");
    
    // Should have moved right
    assert_cursor!(h, 1, 1);
}
