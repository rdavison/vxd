//! File finding tests (usr_22.3).

use vxd::filefinder::FileFinder;
use vxd_tui::filefinder::TuiFileFinder;

#[test]
fn test_find_by_basename() {
    let mut finder = TuiFileFinder::new();
    finder
        .set_paths(vec![
            "/src/main.rs".to_string(),
            "/src/lib.rs".to_string(),
        ])
        .unwrap();

    let matches = finder.find_files("lib.rs");
    assert_eq!(matches, vec!["/src/lib.rs".to_string()]);
}

#[test]
fn test_find_by_prefix() {
    let mut finder = TuiFileFinder::new();
    finder
        .set_paths(vec![
            "/src/lib.rs".to_string(),
            "/src/lib_test.rs".to_string(),
        ])
        .unwrap();

    let matches = finder.find_files("lib");
    assert_eq!(
        matches,
        vec!["/src/lib.rs".to_string(), "/src/lib_test.rs".to_string()]
    );
}

#[test]
fn test_find_with_path_segment() {
    let mut finder = TuiFileFinder::new();
    finder
        .set_paths(vec![
            "/src/lib.rs".to_string(),
            "/docs/src/lib.rs".to_string(),
        ])
        .unwrap();

    let matches = finder.find_files("src/lib.rs");
    assert_eq!(
        matches,
        vec!["/docs/src/lib.rs".to_string(), "/src/lib.rs".to_string()]
    );
}
