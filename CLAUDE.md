# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build              # debug build
cargo build --release    # release build
cargo run -- <file.md>   # run directly, opens in reader mode
cargo run                # no args: opens file browser in cwd
just install             # release build + copy to ~/.local/bin/
just run <file.md>       # shorthand for cargo run
```

No tests or linter are configured yet.

## Architecture

Meld is a terminal markdown reader built with Rust, ratatui, and crossterm. It renders markdown in the terminal with syntax highlighting and live-reloads on file changes.

**Two primary modes** — `AppMode::Browser` (file picker) and `AppMode::Reader` (markdown viewer). Without arguments, meld starts in browser mode showing `.md`/`.markdown` files and directories. Selecting a file switches to reader mode; `Esc`/`Backspace` returns to the browser. Three overlay modes stack on top: `AppMode::Search` (text input), `AppMode::ThemePicker` (modal selector), and `AppMode::Help` (keybinding reference). Overlay modes store the previous mode so they can return correctly.

**Single-threaded event loop** (`main.rs`): Uses `crossterm::event::poll` for input and an `AtomicBool` flag (set by the notify file watcher) for file changes. Only redraws when state actually changes. Global keybindings (`q`, `Ctrl-c`, `t`) are handled first, then mode-specific bindings are dispatched.

**Rendering pipeline**: On each draw, `ui::draw` dispatches to `draw_browser` or `draw_reader`. The reader calls `markdown::parse_markdown` which converts the full markdown source into a `Vec<StyledLine>` using pulldown-cmark. The UI then slices this by scroll offset and renders via ratatui. There is no caching — the entire document is re-parsed on every redraw.

**Key modules**:
- `main.rs` — event loop, file watcher setup, terminal init/cleanup.
- `state.rs` — `AppState` holds mode, scroll position, theme, file content, and `BrowserState`. Mode transitions (`open_file`, `back_to_browser`) live here.
- `browser.rs` — `BrowserState` manages directory listing (dirs first, then `.md` files, hidden files excluded), selection, and scroll for the file picker.
- `markdown.rs` — pulldown-cmark event loop producing styled, word-wrapped lines. Handles headings, code blocks, blockquotes, lists (ordered/unordered), task lists, inline formatting, and horizontal rules.
- `ui.rs` — ratatui layout for both modes: title bar, separator, scrollable content, status bar.
- `theme.rs` — six color themes using 256-color indexed palette (`ALL_THEMES` array). Default is "tokyo night moon" (index 5). Themes are cycled at runtime with `t`.

**Key dependencies**: ratatui 0.30, crossterm 0.28, pulldown-cmark 0.12, notify 7, unicode-width 0.2. Version matters — ratatui and crossterm APIs change significantly between major versions.

**Search system** (`state.rs`): Case-insensitive substring search across rendered lines. `open_search()` enters Search mode, `update_search()` scans on each keystroke, `search_next()`/`search_prev()` cycle matches. Highlighting is applied in `ui.rs` post-parse.

**External editor** (`main.rs`): In reader mode, `e` suspends the TUI, launches `$EDITOR` (or `vi`) on the current file, then restores the TUI and reloads content.
