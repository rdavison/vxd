# DOCS2TESTS

## Scope
- Source manual files: `runtime/doc/usr_*.txt` (Vim user manual only).
- Rust test locations: core in `vxd/`, CLI/TUI in `vxd_tui/`.

## Coverage Legend
- Covered: Rust tests exist for core behavior described in the section.
- Covered (partial): Some behaviors in section are tested; gaps remain.
- Missing (module exists): Rust module exists but no behavioral tests.
- Missing: No Rust tests found for the feature.
- Not ported: Feature not implemented in Rust yet (as far as repo indicates).
- Doc-only: Documentation-only or meta section; no tests needed.

## Feature Coverage Matrix
### usr_01.txt
- 01.1 Two manuals — Doc-only: Manual structure; no runtime feature.
- 01.2 Vim installed — Not ported: Config/bootstrap in core app not implemented in Rust.
- 01.3 Using the Vim tutor — Not ported: :Tutor not implemented in Rust.
- 01.4 Copyright — Doc-only: Copyright notice.

### usr_02.txt
- 02.1 Running Vim for the First Time — Not ported: Startup flow/CLI behavior not implemented in Rust.
- 02.2 Inserting text — Covered (partial): Insert-mode basics in `vxd_tui/tests/mode_insert_spec.rs`.
- 02.3 Moving around — Covered (partial): Basic movement in `vxd_tui/tests/cursor_spec.rs` and `vxd_tui/tests/mode_normal_spec.rs`.
- 02.4 Deleting characters — Covered (partial): Delete with `x` in `vxd_tui/tests/mode_normal_spec.rs`.
- 02.5 Undo and Redo — Covered (partial): Undo tree tests in `vxd_tui/tests/undo_spec.rs`.
- 02.6 Other editing commands — Covered (partial): Open/append commands in `vxd_tui/tests/mode_insert_spec.rs`.
- 02.7 Getting out — Covered (partial): Quit/ZZ/:wq/:x flows in `vxd_tui/src/exit.rs` and `vxd_tui/tests/exit_spec.rs`.
- 02.8 Finding help — Not ported: Help system not implemented in Rust.

### usr_03.txt
- 03.1 Word movement — Covered (partial): Word motion types in `vxd_tui/tests/operator_spec.rs`.
- 03.2 Moving to the start or end of a line — Covered (partial): Line start/end in `vxd_tui/tests/cursor_spec.rs`.
- 03.3 Moving to a character — Covered (partial): Character find motions in `vxd_tui/tests/char_find_spec.rs`.
- 03.4 Matching a parenthesis — Covered (partial): `%` matching in `vxd_tui/tests/match_bracket_spec.rs`.
- 03.5 Moving to a specific line — Covered (partial): Line jumps (`G`, `gg`) in `vxd_tui/tests/cursor_spec.rs`.
- 03.6 Telling where you are — Covered (partial): Position info helpers in `vxd/src/position.rs`.
- 03.7 Scrolling around — Covered (partial): Scroll topline helper in `vxd/src/scroll.rs`.
- 03.8 Simple searches — Covered (partial): Search tests in `vxd_tui/tests/search_spec.rs`.
- 03.9 Simple search patterns — Covered (partial): Pattern tests in `vxd_tui/tests/search_spec.rs`.
- 03.10 Using marks — Covered (partial): Marks and jumps in `vxd_tui/tests/mark_spec.rs`.

### usr_04.txt
- 04.1 Operators and motions — Covered (partial): Operator types and regions in `vxd_tui/tests/operator_spec.rs`.
- 04.2 Changing text — Covered (partial): Basic change/delete in `vxd_tui/tests/mode_normal_spec.rs`.
- 04.3 Repeating a change — Covered (partial): Count and motion types in `vxd_tui/tests/operator_spec.rs`.
- 04.4 Visual mode — Covered (partial): Visual selection tests in `vxd_tui/tests/visual_spec.rs`.
- 04.5 Moving text — Covered (partial): Line move helper in `vxd/src/move_text.rs`.
- 04.6 Copying text — Covered (partial): Yank/put register tests in `vxd_tui/tests/put_spec.rs`.
- 04.7 Using the clipboard — Covered (partial): Register plumbing in `vxd_tui/tests/register_spec.rs` and `vxd_tui/tests/put_spec.rs`.
- 04.8 Text objects — Covered (partial): Text object types and matching in `vxd_tui/tests/textobject_spec.rs`.
- 04.9 Replace mode — Covered (partial): Replace mode tests in `vxd_tui/tests/replace_spec.rs`.
- 04.10 Conclusion — Doc-only: Chapter summary.

### usr_05.txt
- 05.1 The vimrc file — Not ported: vimrc loading not implemented in Rust.
- 05.2 Example vimrc contents — Doc-only: Example config content.
- 05.3 Simple mappings — Covered: Mappings implemented in `vxd_tui/src/input.rs` and tested in `vxd_tui/tests/mapping_spec.rs`.
- 05.4 Adding a package — Not ported: Package loading not implemented in Rust.
- 05.5 Adding a plugin — Not ported: Plugin system not implemented in Rust.
- 05.6 Adding a help file — Not ported: Help tags not implemented in Rust.
- 05.7 The option window — Not ported: Option window UI not implemented in Rust.
- 05.8 Often used options — Covered (partial): Option manager tests in `vxd/src/options.rs`.

### usr_06.txt
- 06.1 Switching it on — Not ported: Syntax highlighting/colors not implemented in Rust.
- 06.2 No or wrong colors? — Not ported: Color troubleshooting not implemented in Rust.
- 06.3 Different colors — Not ported: Color customization not implemented in Rust.
- 06.4 With colors or without colors — Not ported: Colorless mode not implemented in Rust.
- 06.5 Further reading — Doc-only: Further reading.

### usr_07.txt
- 07.1 Edit another file — Covered (partial): File edit model in `vxd/src/fileedit.rs` and `vxd_tui/tests/fileedit_spec.rs`.
- 07.2 A list of files — Covered (partial): Argument list in `vxd_tui/tests/fileedit_spec.rs`.
- 07.3 Jumping from file to file — Covered (partial): Next/prev navigation in `vxd_tui/tests/fileedit_spec.rs`.
- 07.4 Backup files — Covered (partial): Backup path helper in `vxd/src/backup.rs`.
- 07.5 Copy text between files — Missing: Copy across files not tested.
- 07.6 Viewing a file — Covered (partial): View-only modifiable test in `vxd_tui/tests/view_spec.rs`.
- 07.7 Changing the file name — Covered (partial): Buffer renaming in `vxd_tui/tests/buffer_spec.rs`.

### usr_08.txt
- 08.1 Split a window — Covered (partial): Window splitting basics in `vxd_tui/tests/window_spec.rs`.
- 08.2 Split a window on another file — Covered (partial): Split with file in `vxd_tui/tests/window_spec.rs`.
- 08.3 Window size — Covered (partial): Window sizing in `vxd_tui/tests/window_spec.rs`.
- 08.4 Vertical splits — Covered (partial): Vertical splits in `vxd_tui/tests/window_spec.rs`.
- 08.5 Moving windows — Covered (partial): Window movement concepts in `vxd_tui/tests/window_spec.rs`.
- 08.6 Commands for all windows — Covered (partial): Multi-window operations in `vxd_tui/tests/window_spec.rs`.
- 08.7 Viewing differences with diff mode — Not ported: Diff mode not implemented in Rust.
- 08.8 Various — Covered (partial): Misc window behaviors in `vxd_tui/tests/window_spec.rs`.
- 08.9 Tab pages — Covered (partial): Tab pages in `vxd_tui/tests/window_spec.rs`.

### usr_09.txt
- 09.1 Parts of the GUI — Not ported: GUI parts not implemented in Rust.
- 09.2 Using the mouse — Not ported: Mouse support not implemented in Rust.
- 09.3 The clipboard — Covered (partial): Clipboard registers tested in `vxd_tui/tests/register_spec.rs`.
- 09.4 Select mode — Not ported: Select mode not implemented in Rust.

### usr_10.txt
- 10.1 Record and playback commands — Covered (partial): Macro tests in `vxd_tui/tests/macro_spec.rs`.
- 10.2 Substitution — Covered (partial): Substitute application in `vxd/src/search.rs`.
- 10.3 Command ranges — Covered (partial): Range parsing tests in `vxd/src/commands.rs`.
- 10.4 The global command — Covered (partial): Global command logic and parsing tested in `vxd_tui/tests/global_spec.rs` (no full CLI integration yet).
- 10.5 Visual block mode — Covered: Visual block mode tests in `vxd_tui/tests/visual_block_spec.rs`.
- 10.6 Reading and writing part of a file — Missing: Read/write part of file not tested.
- 10.7 Formatting text — Missing: Formatting text not tested.
- 10.8 Changing case — Covered (partial): Case change helper tests in `vxd/src/operators.rs`.
- 10.9 Using an external program — Missing: External commands not tested.

### usr_11.txt
- 11.1 Basic recovery — Covered (partial): Recovery helpers in `vxd/src/recovery.rs`.
- 11.2 Where is the swap file? — Covered (partial): Swap path helpers in `vxd/src/recovery.rs`.
- 11.3 Crashed or not? — Covered (partial): Recovery metadata modeled in `vxd/src/recovery.rs`.
- 11.4 Further reading — Doc-only: Further reading.

### usr_12.txt
- 12.1 Replace a word — Covered (partial): Recipe helpers in `vxd/src/recipes.rs`.
- 12.2 Change "Last, First" to "First Last" — Covered (partial): Recipe helpers in `vxd/src/recipes.rs`.
- 12.3 Sort a list — Covered (partial): Recipe helpers in `vxd/src/recipes.rs`.
- 12.4 Reverse line order — Covered (partial): Recipe helpers in `vxd/src/recipes.rs`.
- 12.5 Count words — Covered (partial): Recipe helpers in `vxd/src/recipes.rs`.
- 12.6 Find a man page — Covered (partial): Recipe helpers in `vxd/src/recipes.rs`.
- 12.7 Trim blanks — Covered (partial): Recipe helpers in `vxd/src/recipes.rs`.
- 12.8 Find where a word is used — Covered (partial): Recipe helpers in `vxd/src/recipes.rs`.

### usr_20.txt
- 20.1 Command line editing — Covered (partial): Cmdline editing in `vxd_tui/tests/cmdline_spec.rs` and `vxd_tui/tests/cmdline_completion_spec.rs`.
- 20.2 Command line abbreviations — Covered (partial): Conceptual tests in `vxd_tui/tests/cmdline_completion_spec.rs`.
- 20.3 Command line completion — Covered (partial): Completion types and items in `vxd_tui/tests/cmdline_completion_spec.rs`.
- 20.4 Command line history — Covered (partial): History kinds and limit eviction in `vxd_tui/tests/cmdline_spec.rs` and `vxd_tui/tests/cmdline_completion_spec.rs`.
- 20.5 Command line window — Covered (partial): Cmdwin concepts in `vxd_tui/tests/cmdline_completion_spec.rs`.

### usr_21.txt
- 21.1 Suspend and resume — Covered (partial): Suspend model in `vxd/src/suspend.rs` and `vxd_tui/tests/suspend_spec.rs`.
- 21.2 Executing shell commands — Not ported: Shell command execution not implemented in Rust.
- 21.3 Remembering information; ShaDa — Not ported: ShaDa not implemented in Rust.
- 21.4 Sessions — Not ported: Sessions not implemented in Rust.
- 21.5 Views — Not ported: Views not implemented in Rust.
- 21.6 Modelines — Covered (partial): Modeline parser in `vxd/src/modeline.rs`.

### usr_22.txt
- 22.1 The file browser — Covered (partial): File browser model in `vxd/src/filebrowser.rs` and `vxd_tui/tests/filebrowser_spec.rs`.
- 22.2 The current directory — Covered (partial): Working directory model in `vxd/src/cwd.rs` and `vxd_tui/tests/cwd_spec.rs`.
- 22.3 Finding a file — Covered (partial): File finding model in `vxd/src/filefinder.rs` and `vxd_tui/tests/filefinder_spec.rs`.
- 22.4 The buffer list — Covered (partial): Buffer list behaviors in `vxd_tui/tests/buffer_list_spec.rs`.

### usr_23.txt
- 23.1 DOS, Mac and Unix files — Covered (partial): Fileformat detection/conversion in `vxd/src/fileformat.rs`.
- 23.2 Files on the internet — Not ported: Net/internet editing not implemented in Rust.
- 23.3 Binary files — Covered (partial): Binary mode helpers in `vxd/src/binary.rs`.
- 23.4 Compressed files — Not ported: Compressed file support not implemented in Rust.

### usr_24.txt
- 24.1 Making corrections — Covered (partial): Insert corrections/backspace in `vxd_tui/tests/mode_insert_spec.rs`.
- 24.2 Showing matches — Covered (partial): Bracket matching helper in `vxd/src/showmatch.rs`.
- 24.3 Completion — Covered (partial): Buffer/line completion engine tests in `vxd/src/completion.rs`.
- 24.4 Repeating an insert — Covered (partial): Repeat insert via Ctrl-A in `vxd_tui/tests/mode_insert_spec.rs`.
- 24.5 Copying from another line — Covered (partial): Ctrl-Y/Ctrl-E copy in `vxd_tui/tests/mode_insert_spec.rs`.
- 24.6 Inserting a register — Covered (partial): Register content tests in `vxd_tui/tests/register_spec.rs`.
- 24.7 Abbreviations — Covered (partial): Insert-mode abbreviations tests in `vxd_tui/tests/abbreviation_spec.rs`.
- 24.8 Entering special characters — Covered (partial): Special chars/unicode in `vxd_tui/tests/mode_insert_spec.rs`.
- 24.9 Digraphs — Covered (partial): Digraph table tests in `vxd/src/digraphs.rs`.
- 24.10 Normal mode commands — Covered (partial): Normal-mode commands exercised in `vxd_tui/tests/mode_normal_spec.rs`.

### usr_25.txt
- 25.1 Breaking lines — Missing: Line breaking not tested.
- 25.2 Aligning text — Missing: Text alignment not tested.
- 25.3 Indents and tabs — Missing: Indent/tabs not tested.
- 25.4 Dealing with long lines — Missing: Long line editing not tested.
- 25.5 Editing tables — Missing: Table editing not tested.

### usr_26.txt
- 26.1 Repeating with Visual mode — Missing: Repeat with visual mode not tested.
- 26.2 Add and subtract — Missing: Add/subtract not tested.
- 26.3 Making a change in many files — Missing: Changes across files not tested.
- 26.4 Using Vim from a shell script — Not ported: Shell script integration not implemented in Rust.

### usr_27.txt
- 27.1 Ignoring case — Covered (partial): Ignorecase/smartcase tests in `vxd/src/search.rs`.
- 27.2 Wrapping around the file end — Covered (partial): Wrapscan behavior in `vxd/src/search.rs`.
- 27.3 Offsets — Covered (partial): Search offset application tests in `vxd/src/search.rs`.
- 27.4 Matching multiple times — Covered (partial): Multi-match search tests in `vxd/src/search.rs`.
- 27.5 Alternatives — Missing: Alternatives not tested.
- 27.6 Character ranges — Missing: Character ranges not tested.
- 27.7 Character classes — Missing: Character classes not tested.
- 27.8 Matching a line break — Missing: Line break matching not tested.
- 27.9 Examples — Missing: Search examples not tested.

### usr_28.txt
- 28.1 What is folding? — Covered (partial): Fold basics tested in `vxd_tui/tests/fold_spec.rs`.
- 28.2 Manual folding — Covered (partial): Manual fold creation/deletion in `vxd_tui/tests/fold_spec.rs`.
- 28.3 Working with folds — Covered (partial): Fold open/close/toggle in `vxd_tui/tests/fold_spec.rs`.
- 28.4 Saving and restoring folds — Missing (module exists): Fold persistence not tested.
- 28.5 Folding by indent — Covered (partial): Indent fold method in `vxd_tui/tests/fold_spec.rs`.
- 28.6 Folding with markers — Covered (partial): Marker fold method in `vxd_tui/tests/fold_spec.rs`.
- 28.7 Folding by syntax — Not ported: Syntax folding not implemented in Rust.
- 28.8 Folding by expression — Covered (partial): Expr fold method in `vxd_tui/tests/fold_spec.rs`.
- 28.9 Folding unchanged lines — Missing (module exists): Fold unchanged lines not tested.
- 28.10 Which fold method to use? — Doc-only: Fold method guidance.

### usr_29.txt
- 29.1 Using tags — Not ported: Tags not implemented in Rust.
- 29.2 The preview window — Not ported: Preview window not implemented in Rust.
- 29.3 Moving through a program — Not ported: Program navigation not implemented in Rust.
- 29.4 Finding global identifiers — Not ported: Global identifier search not implemented in Rust.
- 29.5 Finding local identifiers — Not ported: Local identifier search not implemented in Rust.

### usr_30.txt
- 30.1 Compiling — Not ported: Compiler integration not implemented in Rust.
- 30.2 Indenting C files — Not ported: C indenting not implemented in Rust.
- 30.3 Automatic indenting — Not ported: Auto indenting not implemented in Rust.
- 30.4 Other indenting — Not ported: Other indenting not implemented in Rust.
- 30.5 Tabs and spaces — Not ported: Tabs/spaces conversion not implemented in Rust.
- 30.6 Formatting comments — Not ported: Comment formatting not implemented in Rust.

### usr_31.txt
- 31.1 The file browser — Not ported: GUI file browser not implemented in Rust.
- 31.2 Confirmation — Not ported: GUI confirmation not implemented in Rust.
- 31.3 Menu shortcuts — Not ported: Menu shortcuts not implemented in Rust.
- 31.4 Vim window position and size — Not ported: GUI window size not implemented in Rust.
- 31.5 Various — Not ported: GUI misc not implemented in Rust.

### usr_32.txt
- 32.1 Undo up to a file write — Missing (module exists): Undo to file write not tested.
- 32.2 Numbering changes — Missing (module exists): Undo numbering not tested.
- 32.3 Jumping around the tree — Missing (module exists): Undo tree navigation not tested.
- 32.4 Time travelling — Missing (module exists): Time travel not tested.

### usr_40.txt
- 40.1 Key mapping — Covered: Key mappings implemented in `vxd_tui/src/input.rs` and tested in `vxd_tui/tests/mapping_spec.rs`.
- 40.2 Defining command-line commands — Missing: User commands not tested (parsing only in `vxd/src/commands.rs`).
- 40.3 Autocommands — Covered (partial): Autocmd parsing in `vxd/src/autocmd.rs`.

### usr_41.txt
- 41.1 Introduction — Not ported: Vimscript not implemented in Rust.
- 41.2 Variables — Not ported: Vimscript variables not implemented in Rust.
- 41.3 Expressions — Not ported: Vimscript expressions not implemented in Rust.
- 41.4 Conditionals — Not ported: Vimscript conditionals not implemented in Rust.
- 41.5 Executing an expression — Not ported: Vimscript eval not implemented in Rust.
- 41.6 Using functions — Not ported: Vimscript functions not implemented in Rust.
- 41.7 Defining a function — Not ported: Vimscript function defs not implemented in Rust.
- 41.8 Lists and Dictionaries — Not ported: Vimscript list/dict not implemented in Rust.
- 41.9 Exceptions — Not ported: Vimscript exceptions not implemented in Rust.
- 41.10 Various remarks — Not ported: Vimscript misc not implemented in Rust.
- 41.11 Writing a plugin — Not ported: Plugin authoring not implemented in Rust.
- 41.12 Writing a filetype plugin — Not ported: Filetype plugins not implemented in Rust.
- 41.13 Writing a compiler plugin — Not ported: Compiler plugins not implemented in Rust.
- 41.14 Writing a plugin that loads quickly — Not ported: Fast-loading plugins not implemented in Rust.
- 41.15 Writing library scripts — Not ported: Library scripts not implemented in Rust.
- 41.16 Distributing Vim scripts — Not ported: Distribution not implemented in Rust.

### usr_42.txt
- 42.1 Introduction — Not ported: Menus not implemented in Rust.
- 42.2 Menu commands — Not ported: Menu commands not implemented in Rust.
- 42.3 Various — Not ported: Menu misc not implemented in Rust.
- 42.4 Toolbar and popup menus — Not ported: Toolbar/popup menus not implemented in Rust.

### usr_43.txt
- 43.1 Plugins for a filetype — Not ported: Filetype plugins not implemented in Rust.
- 43.2 Adding a filetype — Not ported: Add filetype not implemented in Rust.

### usr_44.txt
- 44.1 Basic syntax commands — Not ported: Syntax commands not implemented in Rust.
- 44.2 Keywords — Not ported: Syntax keywords not implemented in Rust.
- 44.3 Matches — Not ported: Syntax matches not implemented in Rust.
- 44.4 Regions — Not ported: Syntax regions not implemented in Rust.
- 44.5 Nested items — Not ported: Syntax nesting not implemented in Rust.
- 44.6 Following groups — Not ported: Syntax groups not implemented in Rust.
- 44.7 Other arguments — Not ported: Syntax args not implemented in Rust.
- 44.8 Clusters — Not ported: Syntax clusters not implemented in Rust.
- 44.9 Including another syntax file — Not ported: Syntax include not implemented in Rust.
- 44.10 Synchronizing — Not ported: Syntax sync not implemented in Rust.
- 44.11 Installing a syntax file — Not ported: Syntax install not implemented in Rust.
- 44.12 Portable syntax file layout — Not ported: Syntax layout not implemented in Rust.

### usr_45.txt
- 45.1 Language for Messages — Not ported: Message language not implemented in Rust.
- 45.2 Language for Menus — Not ported: Menu language not implemented in Rust.
- 45.3 Using another encoding — Not ported: Encoding switching not implemented in Rust.
- 45.4 Editing files with a different encoding — Not ported: Editing other encodings not implemented in Rust.
- 45.5 Entering language text — Not ported: Entering language text not implemented in Rust.

## Likely Unported Areas (from manual vs. Rust modules)
- Vimscript and plugin/runtime system (`usr_41`, `usr_43`, `usr_44`).
- GUI/menu/mouse features (`usr_09`, `usr_31`, `usr_42`).
- Tags, compiler integration, file browsers, sessions/ShaDa (`usr_21`, `usr_22`, `usr_29`, `usr_30`).
- Syntax highlighting and colors (`usr_06`, `usr_44`).
- Net/compressed/binary file handling (`usr_23`).

## TODO (Prioritized)
- ~~Add core undo tree behavior tests in `vxd` (sections 02.5, 32.1-32.4).~~ Done: `vxd_tui/tests/undo_spec.rs` (27 tests)
- ~~Add search behavior tests in `vxd` (sections 03.8-03.9, 27.1-27.9).~~ Done: `vxd_tui/tests/search_spec.rs` (43 tests)
- ~~Add window and tab behavior tests in `vxd` (sections 08.1-08.6, 08.9).~~ Done: `vxd_tui/tests/window_spec.rs` (37 tests)
- ~~Add operator/motion behavior tests in `vxd` (sections 04.1-04.3, 03.1).~~ Done: `vxd_tui/tests/operator_spec.rs` (81 tests)
- ~~Add visual mode behavior tests in `vxd`/`vxd_tui` (sections 04.4, 26.1, 10.5).~~ Done: `vxd_tui/tests/visual_spec.rs` (32 tests)
- ~~Add register/yank/put behavior tests in `vxd_tui` (sections 04.6-04.7, 24.6).~~ Done: `vxd_tui/tests/put_spec.rs` (36 tests)
- ~~Add cmdline completion/history/window tests in `vxd_tui` (sections 20.1-20.5).~~ Done: `vxd_tui/tests/cmdline_completion_spec.rs` (62 tests)
- ~~Add folding behavior tests in `vxd` (sections 28.1-28.9).~~ Done: `vxd_tui/tests/fold_spec.rs` (35 tests)
- ~~Add visual block mode behavior tests in `vxd_tui` (section 10.5).~~ Done: `vxd_tui/tests/visual_block_spec.rs` (15 tests)
- ~~Add insert-mode abbreviation tests in `vxd_tui` (section 24.7).~~ Done: `vxd_tui/tests/abbreviation_spec.rs` (4 tests)
- ~~Add key mapping tests in `vxd_tui` (sections 05.3, 40.1).~~ Done: `vxd_tui/tests/mapping_spec.rs` (2 tests)
