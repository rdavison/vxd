# VXD Project - Vim eXact Definition

## Project Overview

VXD is a project to create a trait-based specification of Vim's expected behaviors, derived from Neovim's test suite. The goal is to synthesize a model of the minimum interface that satisfies Vim's expected behaviors, including all quirks and edge cases.

## Architecture

### Crate Structure

```
vxd/
├── vxd/              # Core specification crate (traits, types, stubs)
│   └── src/
│       ├── lib.rs
│       ├── types.rs      # Core types (Position, LineNr, ColNr, etc.)
│       ├── buffer.rs     # Buffer trait and types
│       ├── cursor.rs     # Cursor trait and types
│       ├── modes.rs      # Mode system
│       ├── motions.rs    # Motion commands (pending)
│       ├── operators.rs  # Operator commands (pending)
│       ├── registers.rs  # Register system (pending)
│       ├── marks.rs      # Mark system (pending)
│       ├── search.rs     # Search/pattern matching (pending)
│       ├── commands.rs   # Ex command system (pending)
│       ├── options.rs    # Option/setting system (pending)
│       ├── autocmd.rs    # Autocommand events (pending)
│       ├── windows.rs    # Window management (pending)
│       ├── tabs.rs       # Tab page management (pending)
│       ├── folds.rs      # Folding system (pending)
│       ├── completion.rs # Completion system (pending)
│       ├── undo.rs       # Undo/redo tree (pending)
│       ├── visual.rs     # Visual mode selections (pending)
│       └── textobjects.rs # Text objects (pending)
│
└── vxd_tui/          # TUI implementation crate (to be created)
    └── src/
        └── ...       # Implementations of vxd traits
```

### Feature Flags

Each behavioral domain is gated behind a cargo feature flag:

- `buffer` - Buffer operations and text manipulation
- `cursor` - Cursor movement and positioning
- `modes` - Mode switching (Normal/Insert/Visual/Cmdline)
- `motions` - Movement commands (w, b, e, gg, G, etc.) [depends: cursor]
- `operators` - Operator commands (d, c, y, etc.) [depends: buffer, cursor, registers]
- `registers` - Register system (yank/paste/named)
- `marks` - Mark system (local and global marks) [depends: buffer, cursor]
- `search` - Search and pattern matching [depends: buffer, cursor]
- `commands` - Ex command system [depends: buffer]
- `options` - Option/setting system
- `autocmd` - Autocommand event system
- `windows` - Window management [depends: buffer]
- `tabs` - Tab page management [depends: windows]
- `folds` - Folding system [depends: buffer]
- `completion` - Completion system [depends: buffer, cursor]
- `undo` - Undo/redo tree [depends: buffer]
- `visual` - Visual mode selections [depends: buffer, cursor, modes]
- `textobjects` - Text objects (iw, aw, ip, etc.) [depends: buffer, cursor]
- `all` - Enable all features

## Design Philosophy

1. **Trait-Based Design**: Each Vim concept is captured as a Rust trait that defines the behavioral contract. Implementations can adopt Vim compatibility incrementally.

2. **Exact Vim Compatibility**: The goal is to match Vim's behavior exactly, including quirks. Tests are derived from Neovim's test suite.

3. **Zero Shared Code**: The implementation (`vxd_tui`) should not share any code with the original Neovim C codebase. The original C code may be consulted for understanding, but must be rewritten in modern Rust.

4. **Test-Driven**: Each feature's implementation is validated by ensuring all associated tests pass.

## Source Material

The behavioral specifications are derived from Neovim's test suite located at:

- `test/functional/` - Functional/integration tests (448 test files)
  - `test/functional/api/` - API tests (buffer, window, etc.)
  - `test/functional/editor/` - Editor behavior tests
  - `test/functional/ui/` - UI/display tests
  - `test/functional/vimscript/` - VimScript function tests
  - `test/functional/legacy/` - Ported Vim tests (135 tests)
- `test/unit/` - Unit tests (34 test files)

## Implemented Features

### 1. Core Types (`types.rs`)

- `LineNr` - 1-indexed line number
- `ColNr` - 1-indexed column number (byte offset)
- `Position` - (line, col) pair
- `VirtColNr` - Virtual column (display column)
- `LineRange` - Inclusive line range
- `CharRange` - Character range between positions
- `BufferId`, `WindowId`, `TabId` - Unique identifiers
- `VimError` - Error types with Vim error codes
- `Direction` - Forward/Backward
- `MotionType` - Characterwise/Linewise/Blockwise
- `MotionInclusivity` - Inclusive/Exclusive
- `Count` - Command count prefix

### 2. Buffer (`buffer.rs`)

**Key Behavioral Contracts:**
- Buffers always have at least 1 line (even if empty: `[""]`)
- Line numbers are 1-indexed in UI, but API uses 0-indexed with negative index support
- Negative indices are relative to end (-1 = last line)
- Unloaded buffers return empty content but don't error
- Cursor positions are preserved across edits when possible

**Types:**
- `BufHandle` - Buffer identifier
- `BufferType` - Normal, Nofile, Scratch, Quickfix, Help, Terminal, Prompt, Popup, Acwrite
- `BufHidden` - UseGlobal, Hide, Unload, Delete, Wipe
- `BufDeleteMode` - Unlist, Unload, Wipe
- `BufferLoadState` - Loaded, Unloaded, Wiped
- `BufferInfo` - Comprehensive buffer state
- `BufferChange` - Change event info

**Traits:**
- `Buffer` - Core buffer operations (get_lines, set_lines, set_text, etc.)
- `BufferManager` - Manages multiple buffers

### 3. Cursor (`cursor.rs`)

**Key Behavioral Contracts:**
- Line is always clamped to [1, line_count]
- Column constraints vary by mode:
  - Normal: [0, line_len-1] (can't be past EOL)
  - Insert: [0, line_len] (one past EOL allowed)
  - Virtual Edit: anywhere using coladd
- Cursor remembers desired column (`curswant`) for vertical movement
- Cannot position in middle of multibyte characters
- Virtual column differs from byte column (tabs, wide chars)

**Types:**
- `CursorPosition` - (line, col, coladd)
- `CursorWant` - Column memory (Column(n) or EndOfLine)
- `CursorShape` - Block, Horizontal(%), Vertical(%)
- `CursorBlink` - Blink timing
- `CursorStyle` - Complete cursor appearance
- `CursorContext` - Mode-dependent positioning rules
- `VirtualEdit` - None, Block, Insert, OneMore, All

**Traits:**
- `Cursor` - Core cursor operations
- `CursorManager` - Extended cursor management

### 4. Modes (`modes.rs`)

**Key Behavioral Contracts:**
- Exactly one primary mode active at any time
- Mode determines valid commands and key behaviors
- Mode transitions are deterministic
- Visual selection is cleared on exit to normal mode
- Some states are "blocking" (prevent RPC interruption)

**Types:**
- `Mode` - Normal, Insert, Replace, Visual(type), Select(type), CommandLine(type), OperatorPending, Terminal(type)
- `VisualMode` - Char, Line, Block
- `CommandLineMode` - Normal, Insert, Replace
- `TerminalMode` - Insert, Normal
- `ModeState` - Extended mode state with blocking, pending operator, count, ctrl-o
- `CtrlOMode` - FromInsert, FromVisual
- `ModeTransition` - Transition event
- `InvalidTransition` - Transition error

**Traits:**
- `ModeManager` - Mode state and transitions

**Mode Codes (nvim_get_mode):**
- `n` - Normal
- `i` - Insert
- `R` - Replace
- `v` - Visual (char)
- `V` - Visual (line)
- `\x16` - Visual (block)
- `s/S/\x13` - Select modes
- `c` - Command-line
- `no` - Operator-pending
- `t` - Terminal (insert)
- `nt` - Terminal (normal)
- `niI` - Ctrl-O from insert
- `vs` - Ctrl-O from visual

## Pending Features (To Be Implemented)

### Motions
- Word motions: w, b, e, W, B, E, ge, gE
- Character motions: h, l, f, F, t, T, ;, ,
- Line motions: j, k, +, -, gj, gk, 0, ^, $, g0, g^, g$, |
- Document motions: gg, G, H, M, L, %
- Search motions: /, ?, n, N, *, #
- Motion counts and multipliers
- Inclusive vs exclusive motions
- Linewise vs characterwise motion types

### Operators
- d (delete), c (change), y (yank)
- > (indent), < (dedent), = (format)
- g~ (toggle case), gu (lowercase), gU (uppercase)
- ! (filter), gq (format text)
- Operator + motion combinations
- Operator + text object combinations
- Dot repeat (.)

### Registers
- Unnamed register ("")
- Named registers (a-z, A-Z for append)
- Numbered registers (0-9)
- Small delete register (-)
- Read-only registers (%, #, :, .)
- Expression register (=)
- Selection registers (*, +)
- Black hole register (_)
- Last search register (/)
- Linewise vs characterwise content

### Marks
- Local marks (a-z)
- Global marks (A-Z, 0-9)
- Special marks: '.', '^', '<', '>', '[', ']', '"'
- Jump list
- Change list
- Mark adjustment on buffer changes

### Search
- Forward (/) and backward (?) search
- Pattern syntax (Vim regex)
- Search options (ignorecase, smartcase, magic)
- Incremental search
- Search highlighting
- Search history

### Ex Commands
- Command parsing
- Range handling
- Command arguments
- Command completion
- Common commands (:w, :q, :e, :s, etc.)

### Options
- Global, window-local, buffer-local options
- Option types (boolean, number, string)
- Option validation
- Common options (tabstop, shiftwidth, expandtab, etc.)

### Autocommands
- Event types
- Pattern matching
- Autocommand groups
- Nested autocommands
- Event ordering

### Windows
- Window splitting (horizontal, vertical)
- Window navigation
- Window sizing
- Window-local options
- Window-buffer association

### Tabs
- Tab creation and deletion
- Tab navigation
- Tab-local windows

### Folds
- Fold methods (manual, indent, expr, marker, syntax)
- Fold levels
- Fold open/close
- Fold navigation

### Completion
- Insert mode completion
- Omni completion
- Path completion
- Buffer completion
- Command-line completion

### Undo
- Undo tree (not just linear)
- Undo branches
- Undo persistence
- Change tracking

### Visual Mode
- Selection types (char, line, block)
- Selection adjustment
- Selection operations

### Text Objects
- Word objects: iw, aw, iW, aW
- Sentence objects: is, as
- Paragraph objects: ip, ap
- Block objects: i(, a(, i{, a{, i[, a[, i<, a<
- Quote objects: i", a", i', a', i`, a`
- Tag objects: it, at

## VXD_TUI Crate (To Be Created)

The `vxd_tui` crate will provide a TUI (Terminal User Interface) implementation of the traits defined in `vxd`. This implementation will:

1. Import types/traits from `vxd`
2. Implement each trait with actual functionality
3. Validate against the behavioral tests from `vxd`
4. Provide a working Vim-compatible editor

### Implementation Strategy

For each feature:
1. Implement the trait from `vxd`
2. Run the associated tests
3. Fix any failing tests
4. If stuck, add to retry queue (FIFO)
5. Continue until all tests pass

### Retry Queue

A FIFO queue for tests that are blocked or need revisiting:
- Tests that depend on unimplemented features
- Tests with complex edge cases
- Tests that require deeper investigation

## Test Derivation

Tests are derived from Neovim's Lua test suite. The test patterns follow:

```lua
describe('feature name', function()
  before_each(n.clear)  -- Fresh editor state
  
  it('specific behavior', function()
    n.command(':set option')
    n.feed('keys')
    t.eq(expected, actual)
  end)
end)
```

These are converted to Rust tests using the trait-based test patterns defined in each module.

## References

- Neovim source: This repository (forked from neovim/neovim)
- Neovim test suite: `test/functional/`, `test/unit/`
- Vim documentation: `:help` in Vim
- Neovim API: `:help api`
