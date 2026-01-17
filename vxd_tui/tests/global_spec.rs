//! Global command tests (:g, :v)
//!
//! These tests verify:
//! - :global (match lines)
//! - :vglobal (inverse match)
//! - Pattern matching
//! - Command execution simulation

mod common;

use common::TestHarness;
use vxd::buffer::{Buffer, BufferManager};
use vxd::global::GlobalCommand;
use vxd::search::SimpleSearchEngine;
use vxd::types::LineNr;

#[test]
fn test_global_match_lines() {
    let h = TestHarness::with_lines(&[
        "foo",
        "bar",
        "foo bar",
        "baz",
    ]);

    let cmd = GlobalCommand::parse("/foo/p", false).unwrap();
    
    // We need a search engine. TestHarness uses Editor which uses... ?
    // Editor uses `SimpleSearchEngine`? No, Editor has `cursor` and `buffers`, but `search` isn't fully integrated yet?
    // Let's create a SimpleSearchEngine from buffer content.
    
    let lines = h.get_lines();
    let search_engine = SimpleSearchEngine::new(lines);
    
    let matched = cmd.match_lines(h.editor.buffers.current(), &search_engine).unwrap();
    
    assert_eq!(matched, vec![LineNr(1), LineNr(3)]);
}

#[test]
fn test_vglobal_match_lines() {
    let h = TestHarness::with_lines(&[
        "foo",
        "bar",
        "foo bar",
        "baz",
    ]);

    // :v/foo/p - matches lines NOT containing foo
    let cmd = GlobalCommand::parse("/foo/p", true).unwrap();
    
    let lines = h.get_lines();
    let search_engine = SimpleSearchEngine::new(lines);
    
    let matched = cmd.match_lines(h.editor.buffers.current(), &search_engine).unwrap();
    
    assert_eq!(matched, vec![LineNr(2), LineNr(4)]);
}

#[test]
fn test_global_execution_simulation_delete() {
    let mut h = TestHarness::with_lines(&[
        "keep1",
        "delete me",
        "keep2",
        "delete me too",
        "keep3",
    ]);

    // :g/delete/d
    let cmd = GlobalCommand::parse("/delete/d", false).unwrap();
    
    let lines = h.get_lines();
    let search_engine = SimpleSearchEngine::new(lines);
    
    let matched = cmd.match_lines(h.editor.buffers.current(), &search_engine).unwrap();
    
    // Simulate execution: delete marked lines.
    // Note: iterating and deleting requires handling indices carefully.
    // Usually reverse order is safest for indices, but Vim iterates forward.
    // If we delete line 2, line 3 becomes line 2.
    // The collected LineNrs are [2, 4].
    // If we delete 2:
    // "keep1" (1)
    // "keep2" (2) <- was 3
    // "delete me too" (3) <- was 4
    // "keep3" (4) <- was 5
    // Now we need to delete original 4. It is now at 3.
    // Shift = 1.
    
    let mut shift = 0;
    for line in matched {
        // Calculate current line number
        let current_line = line.0 - shift;
        
        // Check if command is 'd'
        if cmd.command == "d" {
            // Delete line
            h.set_cursor(current_line, 0);
            
            // Actually `h.set_lines_range` is easier or `editor.buffers.current_mut().set_lines`.
            
            h.editor.buffers.current_mut().set_lines(
                (current_line as i64) - 1, 
                current_line as i64, 
                false, 
                vec![]
            ).unwrap();
            
            shift += 1;
        }
    }
    
    assert_eq!(h.get_lines(), vec!["keep1", "keep2", "keep3"]);
}

#[test]
fn test_global_parse_error() {
    let err = GlobalCommand::parse("foo", false).unwrap_err();
    // Should be some error about delimiter
    assert!(format!("{:?}", err).contains("Regular expression missing"));
}
