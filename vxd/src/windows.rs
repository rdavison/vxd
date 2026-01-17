//! Window management.
//!
//! Windows are viewports into buffers. Multiple windows can display the
//! same buffer, and windows can be split horizontally or vertically.

use crate::buffer::BufHandle;
use crate::cursor::CursorPosition;
use crate::types::*;

// ============================================================================
// Window Types
// ============================================================================

/// Window handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WinHandle(pub usize);

impl WinHandle {
    /// Current window handle (0 in API)
    pub const CURRENT: WinHandle = WinHandle(0);
}

/// Window dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowSize {
    /// Width in columns
    pub width: usize,
    /// Height in rows
    pub height: usize,
}

/// Window position (relative to editor)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowPosition {
    /// Row (0-indexed from top)
    pub row: usize,
    /// Column (0-indexed from left)
    pub col: usize,
}

/// Split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitDirection {
    /// Split horizontally (new window below)
    Horizontal,
    /// Split vertically (new window to the right)
    Vertical,
}

/// Window configuration
#[derive(Debug, Clone, Default)]
pub struct WindowConfig {
    /// Position relative to editor
    pub position: WindowPosition,
    /// Size
    pub size: WindowSize,
    /// Whether this is a floating window
    pub floating: bool,
    /// Whether window has focus
    pub focused: bool,
    /// Border style (for floating windows)
    pub border: Option<String>,
    /// Window title (for floating windows)
    pub title: Option<String>,
    /// Z-index (for floating windows)
    pub zindex: Option<u32>,
}

/// Window state
#[derive(Debug, Clone)]
pub struct WindowState {
    /// Buffer displayed in window
    pub buffer: BufHandle,
    /// Cursor position
    pub cursor: CursorPosition,
    /// Top line visible (for scrolling)
    pub topline: LineNr,
    /// Left column visible (for horizontal scrolling)
    pub leftcol: usize,
    /// Desired/wanted column
    pub curswant: usize,
}

// ============================================================================
// Window Trait
// ============================================================================

/// Trait for window operations
pub trait Window {
    /// Get window handle
    fn handle(&self) -> WinHandle;

    /// Get the buffer displayed in this window
    fn buffer(&self) -> BufHandle;

    /// Set the buffer for this window
    fn set_buffer(&mut self, buf: BufHandle) -> VimResult<()>;

    /// Get cursor position
    fn cursor(&self) -> CursorPosition;

    /// Set cursor position
    fn set_cursor(&mut self, pos: CursorPosition) -> VimResult<()>;

    /// Get window size
    fn size(&self) -> WindowSize;

    /// Set window size
    fn set_size(&mut self, size: WindowSize) -> VimResult<()>;

    /// Get window width
    fn width(&self) -> usize {
        self.size().width
    }

    /// Set window width
    fn set_width(&mut self, width: usize) -> VimResult<()>;

    /// Get window height
    fn height(&self) -> usize {
        self.size().height
    }

    /// Set window height
    fn set_height(&mut self, height: usize) -> VimResult<()>;

    /// Get window position
    fn position(&self) -> WindowPosition;

    /// Get the top visible line
    fn topline(&self) -> LineNr;

    /// Set the top visible line
    fn set_topline(&mut self, line: LineNr) -> VimResult<()>;

    /// Check if window is valid
    fn is_valid(&self) -> bool;

    /// Check if this is a floating window
    fn is_floating(&self) -> bool;

    /// Close the window
    fn close(&mut self, force: bool) -> VimResult<()>;
}

// ============================================================================
// Window Manager Trait
// ============================================================================

/// Manages windows
pub trait WindowManager {
    /// The window type
    type Win: Window;

    /// Get the current window
    fn current(&self) -> &Self::Win;

    /// Get the current window mutably
    fn current_mut(&mut self) -> &mut Self::Win;

    /// Get a window by handle
    fn get(&self, handle: WinHandle) -> Option<&Self::Win>;

    /// Get a window by handle mutably
    fn get_mut(&mut self, handle: WinHandle) -> Option<&mut Self::Win>;

    /// List all window handles
    fn list(&self) -> Vec<WinHandle>;

    /// Create a new split window
    fn split(&mut self, direction: SplitDirection) -> VimResult<WinHandle>;

    /// Create a floating window
    fn create_floating(&mut self, config: WindowConfig) -> VimResult<WinHandle>;

    /// Close a window
    fn close(&mut self, handle: WinHandle, force: bool) -> VimResult<()>;

    /// Set the current window
    fn set_current(&mut self, handle: WinHandle) -> VimResult<()>;

    /// Move to the window in a direction
    fn go_to(&mut self, direction: Direction) -> VimResult<()>;

    /// Get number of windows
    fn count(&self) -> usize {
        self.list().len()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #[allow(dead_code)]
    mod behavioral_tests {
        //! # Window Behavioral Tests
        //!
        //! Tests derived from Neovim's test suite:
        //! - test/functional/api/window_spec.lua
        //! - test/functional/ui/float_spec.lua
        //!
        //! ## Key Quirks
        //!
        //! 1. **Window local options**: Each window has its own option values.
        //!
        //! 2. **Cursor per window**: Each window maintains its own cursor position
        //!    even when showing the same buffer.
        //!
        //! 3. **Topline**: Each window scrolls independently.
        //!
        //! 4. **Close behavior**: Closing last window quits Vim (unless hidden).
        //!
        //! 5. **Floating windows**: Have special positioning and z-order.
    }
}
