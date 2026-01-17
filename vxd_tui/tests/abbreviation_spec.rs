mod common;

use vxd::abbreviations::AbbreviationManager;
use vxd::modes::Mode;
use common::TestHarness;

#[test]
fn test_insert_mode_abbreviation_basic() {
    let mut h = TestHarness::new();
    
    // Define an abbreviation
    h.editor.abbreviations.add(Mode::Insert, "ad", "advertisement", false, false).unwrap();

    // Enter insert mode
    h.feed("i");

    // Type 'ad' then space
    h.feed("ad ");
    
    // Expect 'advertisement '
    assert_eq!(h.content(), "advertisement ");
}

#[test]
fn test_insert_mode_abbreviation_punctuation_trigger() {
    let mut h = TestHarness::new();
    h.editor.abbreviations.add(Mode::Insert, "ad", "advertisement", false, false).unwrap();
    h.feed("i");
    h.feed("ad.");
    assert_eq!(h.content(), "advertisement.");
}

#[test]
fn test_insert_mode_abbreviation_no_trigger_on_keyword() {
    let mut h = TestHarness::new();
    h.editor.abbreviations.add(Mode::Insert, "ad", "advertisement", false, false).unwrap();
    h.feed("i");
    h.feed("ada"); // 'a' is a keyword char, should not trigger
    assert_eq!(h.content(), "ada");
}

#[test]
fn test_insert_mode_abbreviation_no_trigger_on_superstring() {
    let mut h = TestHarness::new();
    h.editor.abbreviations.add(Mode::Insert, "ad", "advertisement", false, false).unwrap();
    h.feed("i");
    h.feed("bad "); // 'b' is keyword, so 'ad' is not a whole word
    assert_eq!(h.content(), "bad ");
}
