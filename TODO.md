# TODO

## Now

- [ ] Task summary in status bar (e.g., `3/7 tasks`) #feature

## Next

- [ ] Syntax highlighting in fenced code blocks (syntect or tree-sitter) #feature
- [ ] Accept stdin (`cat file.md | mdr`) #feature
- [ ] Follow markdown links — open URLs in browser, jump to local `.md` files #feature
- [ ] Table of contents overlay — list headings, jump to any section #feature
- [ ] Browser preview pane (split layout showing selected file) #improvement
- [ ] Bookmarks — mark positions in a file, jump back to them #improvement
- [ ] Inline image rendering via Sixel (with fallback to `[image: alt text]` placeholder) #improvement
- [ ] Footnote rendering #chore
- [ ] Tidy command — keybind to move completed `[x]` tasks to a "Done" section #chore

## Later

## Done

- [x] Configurable themes via `config.toml` with serde/toml, per-theme category label colors #feature
- [x] Multiple file tabs — open several files, switch with `Tab`/`Shift-Tab`, close with `q` #feature
- [x] Search (`/`) with highlighted matches and `n`/`N` navigation #feature
- [x] Toggle checkboxes in-place (`x` or `Space` on a task line) — flip `[ ]` / `[x]` and write back to file #feature
- [x] Cursor-based navigation with line highlighting #feature
- [x] Filter view — `F` to collapse document to only unchecked task lines (with heading context) #feature
- [x] Fuzzy search/filter in file picker #feature
- [x] Outline-aware folding — `Enter` to collapse/expand sections by heading #feature
- [x] Use `unicode-width` for line length calculations #improvement
- [x] Use actual visible height for `Ctrl-d`/`Ctrl-u` page size in reader mode #improvement
- [x] Clamp `scroll_bottom` properly instead of setting `usize::MAX` #improvement
- [x] Cache parsed markdown — only re-parse when content changes, not on every scroll/redraw #improvement
- [x] Mouse scroll support (scrolls viewport, clamps cursor) #improvement
- [x] Scrollbar widget (configurable via `scrollbar` in config.toml) #improvement
- [x] Auto-refresh file picker when files are added/removed in the watched directory #improvement
- [x] Task-aware navigation (`Ctrl-n` / `Ctrl-p` to jump between unchecked tasks) #improvement
- [x] Persist selected theme to config file #docs
- [x] File watcher notification — show `[updated]` in status bar when file reloads (clears on next keypress) #docs
- [x] Render tables #refactor
- [x] Render strikethrough #refactor
