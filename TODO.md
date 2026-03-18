# TODO

## Now

- [x] Use `unicode-width` for line length calculations (dependency exists but `current_line_len` uses `.len()`, breaks on wide/CJK chars)
- [x] Use actual visible height for `Ctrl-d`/`Ctrl-u` page size in reader mode (currently hardcoded to 20)
- [x] Clamp `scroll_bottom` properly instead of setting `usize::MAX`
- [x] Render tables (pulldown-cmark parses them but they're silently dropped)
- [x] Render strikethrough

## Next

- [ ] Cache parsed markdown — only re-parse when content changes, not on every scroll/redraw
- [x] Search (`/`) with highlighted matches and `n`/`N` navigation
- [ ] Syntax highlighting in fenced code blocks (syntect or tree-sitter)
- [ ] Mouse scroll support (crossterm already emits mouse events)
- [ ] Accept stdin (`cat file.md | meld`)
- [ ] Persist selected theme to config file (`~/.config/meld/` on Linux, `~/Library/Application Support/meld/` on macOS)

## Later

- [ ] Follow markdown links — open URLs in browser, jump to local `.md` files
- [ ] Browser preview pane (split layout showing selected file)
- [ ] Fuzzy search/filter in browser mode
- [ ] Scrollbar widget
- [ ] Footnote rendering
