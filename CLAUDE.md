# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build              # debug build
cargo build --release    # release build
cargo run -- <file.md>   # run directly
just install             # release build + copy to ~/.local/bin/
just run <file.md>       # shorthand for cargo run
```

No tests or linter are configured yet.

## Architecture

Meld is a terminal markdown reader built with Rust, ratatui, and crossterm. It renders a markdown file in the terminal with syntax highlighting and live-reloads on file changes.

**Single-threaded event loop** (`main.rs`): Uses `crossterm::event::poll` for input and an `AtomicBool` flag (set by the notify file watcher) for file changes. Only redraws when state actually changes.

**Rendering pipeline**: On each draw, `ui::draw` calls `markdown::parse_markdown` which converts the full markdown source into a `Vec<StyledLine>` using pulldown-cmark. The UI then slices this by scroll offset and renders via ratatui. There is no caching — the entire document is re-parsed on every redraw.

**Key modules**:
- `markdown.rs` — pulldown-cmark event loop producing styled, word-wrapped lines. Handles headings, code blocks, blockquotes, lists (ordered/unordered), task lists, inline formatting, and horizontal rules.
- `ui.rs` — ratatui layout: title bar, separator, scrollable content, status bar.
- `theme.rs` — color themes using 256-color indexed palette. Themes are cycled at runtime with `t`.
- `state.rs` — scroll position, current theme, file path, and content.
