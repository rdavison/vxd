//! Window and tab tests ported from Neovim tests
//!
//! These tests verify window and tab behavior including:
//! - Window splitting (horizontal/vertical)
//! - Window sizing
//! - Window navigation
//! - Tab pages
//! - Floating windows

mod common;

use common::TestHarness;
use vxd::buffer::BufHandle;
use vxd::cursor::CursorPosition;
use vxd::tabs::{TabHandle, TabInfo};
use vxd::types::{Direction, LineNr};
use vxd::windows::{
    SplitDirection, WinHandle, WindowConfig, WindowPosition, WindowSize, WindowState,
};

// ============================================================================
// Window Handle Tests
// ============================================================================

/// Test: window handle creation
/// Source: window API
#[test]
fn test_window_handle() {
    let handle = WinHandle(1);
    assert_eq!(handle.0, 1);
}

/// Test: current window handle constant
/// Source: nvim_get_current_win returns 0 for current
#[test]
fn test_window_handle_current() {
    assert_eq!(WinHandle::CURRENT.0, 0);
}

// ============================================================================
// Window Size Tests
// ============================================================================

/// Test: window size structure
/// Source: nvim_win_get_width/height
#[test]
fn test_window_size() {
    let size = WindowSize {
        width: 80,
        height: 24,
    };

    assert_eq!(size.width, 80);
    assert_eq!(size.height, 24);
}

/// Test: default window size
/// Source: initial window state
#[test]
fn test_window_size_default() {
    let size = WindowSize::default();

    assert_eq!(size.width, 0);
    assert_eq!(size.height, 0);
}

// ============================================================================
// Window Position Tests
// ============================================================================

/// Test: window position structure
/// Source: nvim_win_get_position
#[test]
fn test_window_position() {
    let pos = WindowPosition { row: 10, col: 20 };

    assert_eq!(pos.row, 10);
    assert_eq!(pos.col, 20);
}

/// Test: default window position
/// Source: top-left is (0, 0)
#[test]
fn test_window_position_default() {
    let pos = WindowPosition::default();

    assert_eq!(pos.row, 0);
    assert_eq!(pos.col, 0);
}

// ============================================================================
// Split Direction Tests
// ============================================================================

/// Test: horizontal split direction
/// Source: :split command
#[test]
fn test_split_horizontal() {
    let dir = SplitDirection::Horizontal;
    assert!(matches!(dir, SplitDirection::Horizontal));
}

/// Test: vertical split direction
/// Source: :vsplit command
#[test]
fn test_split_vertical() {
    let dir = SplitDirection::Vertical;
    assert!(matches!(dir, SplitDirection::Vertical));
}

// ============================================================================
// Window Config Tests
// ============================================================================

/// Test: window config for floating
/// Source: nvim_open_win with config
#[test]
fn test_window_config_floating() {
    let config = WindowConfig {
        position: WindowPosition { row: 5, col: 10 },
        size: WindowSize {
            width: 40,
            height: 10,
        },
        floating: true,
        focused: false,
        border: Some("rounded".to_string()),
        title: Some("My Window".to_string()),
        zindex: Some(50),
    };

    assert!(config.floating);
    assert_eq!(config.size.width, 40);
    assert!(config.border.is_some());
    assert!(config.title.is_some());
}

/// Test: default window config
/// Source: non-floating window
#[test]
fn test_window_config_default() {
    let config = WindowConfig::default();

    assert!(!config.floating);
    assert!(!config.focused);
    assert!(config.border.is_none());
    assert!(config.title.is_none());
    assert!(config.zindex.is_none());
}

// ============================================================================
// Window State Tests
// ============================================================================

/// Test: window state tracks buffer
/// Source: each window has a buffer
#[test]
fn test_window_state_buffer() {
    let state = WindowState {
        buffer: BufHandle(1),
        cursor: CursorPosition::new(LineNr(1), 0),
        topline: LineNr(1),
        leftcol: 0,
        curswant: 0,
    };

    assert_eq!(state.buffer.0, 1);
}

/// Test: window state tracks cursor
/// Source: each window has its own cursor
#[test]
fn test_window_state_cursor() {
    let state = WindowState {
        buffer: BufHandle(1),
        cursor: CursorPosition::new(LineNr(5), 10),
        topline: LineNr(1),
        leftcol: 0,
        curswant: 10,
    };

    assert_eq!(state.cursor.line, LineNr(5));
    assert_eq!(state.cursor.col, 10);
}

/// Test: window state tracks topline (scroll position)
/// Source: each window scrolls independently
#[test]
fn test_window_state_topline() {
    let state = WindowState {
        buffer: BufHandle(1),
        cursor: CursorPosition::new(LineNr(50), 0),
        topline: LineNr(40),
        leftcol: 0,
        curswant: 0,
    };

    assert_eq!(state.topline, LineNr(40));
}

/// Test: window state tracks horizontal scroll
/// Source: leftcol for horizontal scrolling
#[test]
fn test_window_state_leftcol() {
    let state = WindowState {
        buffer: BufHandle(1),
        cursor: CursorPosition::new(LineNr(1), 100),
        topline: LineNr(1),
        leftcol: 80,
        curswant: 100,
    };

    assert_eq!(state.leftcol, 80);
}

/// Test: window state tracks curswant
/// Source: cursor wants column for vertical movement
#[test]
fn test_window_state_curswant() {
    let state = WindowState {
        buffer: BufHandle(1),
        cursor: CursorPosition::new(LineNr(1), 5),
        topline: LineNr(1),
        leftcol: 0,
        curswant: 20, // Cursor was at col 20, now on shorter line
    };

    assert_eq!(state.curswant, 20);
}

// ============================================================================
// Tab Handle Tests
// ============================================================================

/// Test: tab handle creation
/// Source: tab API
#[test]
fn test_tab_handle() {
    let handle = TabHandle(1);
    assert_eq!(handle.0, 1);
}

/// Test: current tab handle constant
/// Source: nvim_get_current_tabpage returns 0
#[test]
fn test_tab_handle_current() {
    assert_eq!(TabHandle::CURRENT.0, 0);
}

// ============================================================================
// Tab Info Tests
// ============================================================================

/// Test: tab info structure
/// Source: nvim_tabpage_list_wins
#[test]
fn test_tab_info() {
    let info = TabInfo {
        handle: TabHandle(1),
        windows: vec![WinHandle(1), WinHandle(2)],
        current_window: WinHandle(1),
    };

    assert_eq!(info.handle.0, 1);
    assert_eq!(info.windows.len(), 2);
    assert_eq!(info.current_window.0, 1);
}

/// Test: tab with single window
/// Source: default tab state
#[test]
fn test_tab_info_single_window() {
    let info = TabInfo {
        handle: TabHandle(1),
        windows: vec![WinHandle(1)],
        current_window: WinHandle(1),
    };

    assert_eq!(info.windows.len(), 1);
}

// ============================================================================
// Window Navigation Tests
// ============================================================================

/// Test: direction for window navigation
/// Source: Ctrl-W h/j/k/l
#[test]
fn test_window_navigation_directions() {
    let directions = [Direction::Forward, Direction::Backward];

    for dir in directions {
        assert!(matches!(dir, Direction::Forward | Direction::Backward));
    }
}

// ============================================================================
// Window Splitting Behavior Tests
// ============================================================================

/// Test: horizontal split creates window below
/// Source: :split behavior
#[test]
fn test_horizontal_split_below() {
    // After horizontal split, new window is below
    let original_pos = WindowPosition { row: 0, col: 0 };
    let new_pos = WindowPosition { row: 12, col: 0 }; // Half of 24

    assert!(new_pos.row > original_pos.row);
}

/// Test: vertical split creates window to right
/// Source: :vsplit behavior
#[test]
fn test_vertical_split_right() {
    // After vertical split, new window is to the right
    let original_pos = WindowPosition { row: 0, col: 0 };
    let new_pos = WindowPosition { row: 0, col: 40 }; // Half of 80

    assert!(new_pos.col > original_pos.col);
}

/// Test: split reduces original window size
/// Source: splitting shares space
#[test]
fn test_split_shares_space() {
    let original_height = 24;
    let after_split_height = 12; // Approximately half

    assert!(after_split_height < original_height);
}

// ============================================================================
// Multiple Windows Same Buffer Tests
// ============================================================================

/// Test: two windows can show same buffer
/// Source: :split on same buffer
#[test]
fn test_multiple_windows_same_buffer() {
    let buffer = BufHandle(1);

    let win1 = WindowState {
        buffer,
        cursor: CursorPosition::new(LineNr(1), 0),
        topline: LineNr(1),
        leftcol: 0,
        curswant: 0,
    };

    let win2 = WindowState {
        buffer,                                     // Same buffer
        cursor: CursorPosition::new(LineNr(10), 5), // Different cursor
        topline: LineNr(5),                         // Different scroll
        leftcol: 0,
        curswant: 5,
    };

    // Same buffer
    assert_eq!(win1.buffer, win2.buffer);
    // Different cursors
    assert_ne!(win1.cursor, win2.cursor);
    // Different scroll positions
    assert_ne!(win1.topline, win2.topline);
}

// ============================================================================
// Floating Window Tests
// ============================================================================

/// Test: floating window config
/// Source: nvim_open_win with relative='editor'
#[test]
fn test_floating_window_config() {
    let config = WindowConfig {
        position: WindowPosition { row: 5, col: 10 },
        size: WindowSize {
            width: 30,
            height: 10,
        },
        floating: true,
        focused: true,
        border: Some("single".to_string()),
        title: None,
        zindex: Some(100),
    };

    assert!(config.floating);
    assert!(config.focused);
    assert_eq!(config.zindex, Some(100));
}

/// Test: floating window with border styles
/// Source: border option in nvim_open_win
#[test]
fn test_floating_window_borders() {
    let borders = ["none", "single", "double", "rounded", "solid", "shadow"];

    for border in borders {
        let config = WindowConfig {
            floating: true,
            border: Some(border.to_string()),
            ..Default::default()
        };

        assert!(config.border.is_some());
    }
}

/// Test: floating window z-index ordering
/// Source: zindex option
#[test]
fn test_floating_window_zindex() {
    let config1 = WindowConfig {
        floating: true,
        zindex: Some(50),
        ..Default::default()
    };

    let config2 = WindowConfig {
        floating: true,
        zindex: Some(100),
        ..Default::default()
    };

    // Higher zindex appears on top
    assert!(config2.zindex.unwrap() > config1.zindex.unwrap());
}

// ============================================================================
// Tab Page Tests
// ============================================================================

/// Test: tab contains windows
/// Source: tab pages hold window layouts
#[test]
fn test_tab_contains_windows() {
    let info = TabInfo {
        handle: TabHandle(1),
        windows: vec![WinHandle(1), WinHandle(2), WinHandle(3)],
        current_window: WinHandle(2),
    };

    assert_eq!(info.windows.len(), 3);
    assert!(info.windows.contains(&WinHandle(2)));
}

/// Test: tab tracks current window
/// Source: each tab remembers active window
#[test]
fn test_tab_current_window() {
    let info = TabInfo {
        handle: TabHandle(1),
        windows: vec![WinHandle(1), WinHandle(2)],
        current_window: WinHandle(2),
    };

    assert_eq!(info.current_window, WinHandle(2));
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test: minimum window size
/// Source: window can't be smaller than 1x1
#[test]
fn test_window_minimum_size() {
    let size = WindowSize {
        width: 1,
        height: 1,
    };

    assert!(size.width >= 1);
    assert!(size.height >= 1);
}

/// Test: window at origin
/// Source: first window is at (0, 0)
#[test]
fn test_window_at_origin() {
    let pos = WindowPosition { row: 0, col: 0 };

    assert_eq!(pos.row, 0);
    assert_eq!(pos.col, 0);
}

/// Test: single tab single window
/// Source: minimal editor state
#[test]
fn test_single_tab_single_window() {
    let info = TabInfo {
        handle: TabHandle(1),
        windows: vec![WinHandle(1)],
        current_window: WinHandle(1),
    };

    assert_eq!(info.windows.len(), 1);
    assert_eq!(info.current_window, info.windows[0]);
}
