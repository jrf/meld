use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Find the git repository root, if any.
fn find_git_root(start: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start)
        .output()
        .ok()?;
    if output.status.success() {
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(PathBuf::from(root))
    } else {
        None
    }
}

/// Collect all tracked .md files using git, falling back to manual walk.
fn collect_md_files(root: &Path) -> Vec<(String, PathBuf)> {
    // Try git ls-files first (respects .gitignore)
    if let Ok(output) = Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard", "*.md", "*.markdown"])
        .current_dir(root)
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut files: Vec<(String, PathBuf)> = stdout
                .lines()
                .filter(|l| !l.is_empty())
                .map(|rel| (rel.to_string(), root.join(rel)))
                .collect();
            files.sort_by(|a, b| a.0.cmp(&b.0));
            return files;
        }
    }

    // Fallback: manual recursive walk, skipping hidden dirs
    let mut files = Vec::new();
    collect_md_files_recursive(root, &mut files, root);
    files
}

fn collect_md_files_recursive(dir: &Path, files: &mut Vec<(String, PathBuf)>, base: &Path) {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };

    let mut entries: Vec<_> = read_dir.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_md_files_recursive(&entry.path(), files, base);
        } else if name.ends_with(".md") || name.ends_with(".markdown") {
            let rel = entry
                .path()
                .strip_prefix(base)
                .unwrap_or(&entry.path())
                .to_string_lossy()
                .to_string();
            files.push((rel, entry.path()));
        }
    }
}

pub struct BrowserEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

pub struct BrowserState {
    pub current_dir: PathBuf,
    pub entries: Vec<BrowserEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub filtered_indices: Vec<usize>,
    /// Cached recursive .md files from git root
    recursive_entries: Vec<BrowserEntry>,
    recursive_loaded: bool,
    /// Receiver for background recursive file collection
    recursive_rx: Option<mpsc::Receiver<Vec<BrowserEntry>>>,
}

impl BrowserState {
    pub fn new(dir: PathBuf) -> Self {
        let mut state = Self {
            current_dir: dir,
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            filter: String::new(),
            filtered_indices: Vec::new(),
            recursive_entries: Vec::new(),

            recursive_loaded: false,
            recursive_rx: None,
        };
        state.load_dir();
        state
    }

    pub fn load_dir(&mut self) {
        self.entries.clear();
        self.filter.clear();
        self.recursive_loaded = false;
        self.recursive_rx = None;

        // Add parent directory entry
        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(BrowserEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
            });
        }

        let Ok(read_dir) = std::fs::read_dir(&self.current_dir) else {
            self.rebuild_filter();
            return;
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in read_dir.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files
            if name.starts_with('.') {
                continue;
            }

            if file_type.is_dir() {
                dirs.push(BrowserEntry {
                    name: format!("{}/", name),
                    path: entry.path(),
                    is_dir: true,
                });
            } else if name.ends_with(".md") || name.ends_with(".markdown") {
                files.push(BrowserEntry {
                    name,
                    path: entry.path(),
                    is_dir: false,
                });
            }
        }

        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        files.sort_by(|a, b| a.name.cmp(&b.name));

        self.entries.extend(dirs);
        self.entries.extend(files);
        self.rebuild_filter();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Whether recursive file collection is still in progress.
    pub fn recursive_loading(&self) -> bool {
        self.recursive_rx.is_some()
    }

    /// Kick off background collection of recursive .md files.
    pub fn preload_recursive(&mut self) {
        if self.recursive_loaded || self.recursive_rx.is_some() {
            return;
        }
        let dir = self.current_dir.clone();
        let (tx, rx) = mpsc::channel();
        self.recursive_rx = Some(rx);
        thread::spawn(move || {
            let root = find_git_root(&dir).unwrap_or(dir);
            let entries: Vec<BrowserEntry> = collect_md_files(&root)
                .into_iter()
                .map(|(rel, path)| BrowserEntry {
                    name: rel,
                    path,
                    is_dir: false,
                })
                .collect();
            let _ = tx.send(entries);
        });
    }

    /// Check if background results are ready; returns true if new data arrived.
    pub fn poll_recursive(&mut self) -> bool {
        if let Some(ref rx) = self.recursive_rx {
            if let Ok(entries) = rx.try_recv() {
                self.recursive_entries = entries;
                self.recursive_loaded = true;
                self.recursive_rx = None;
                // Re-filter with the new data if there's an active filter
                if !self.filter.is_empty() {
                    self.rebuild_filter();
                }
                return true;
            }
        }
        false
    }

    pub fn rebuild_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_indices = (0..self.entries.len()).collect();
            return;
        }

        let pattern = Pattern::parse(
            &self.filter,
            CaseMatching::Ignore,
            Normalization::Smart,
        );
        let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
        let mut buf = Vec::new();

        // Use recursive entries if loaded, otherwise fall back to local entries
        let source = if self.recursive_loaded {
            &self.recursive_entries
        } else {
            &self.entries
        };
        let mut scored: Vec<(usize, u32)> = source
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                let haystack = Utf32Str::new(&e.name, &mut buf);
                pattern.score(haystack, &mut matcher).map(|s| (i, s))
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        self.filtered_indices = scored.into_iter().map(|(i, _)| i).collect();
    }

    pub fn filtered_entries(&self) -> Vec<(usize, &BrowserEntry)> {
        let source = self.active_source();
        self.filtered_indices
            .iter()
            .filter_map(|&i| source.get(i).map(|e| (i, e)))
            .collect()
    }

    pub fn select_down(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = (self.selected + 1).min(self.filtered_indices.len() - 1);
        }
    }

    pub fn select_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Returns Some(path) if a markdown file was selected, None if navigated into a directory.
    pub fn enter_selected(&mut self) -> Option<PathBuf> {
        let &real_index = self.filtered_indices.get(self.selected)?;
        let entry = self.active_source().get(real_index)?;
        if entry.is_dir {
            self.current_dir = entry.path.clone();
            self.load_dir();
            None
        } else {
            Some(entry.path.clone())
        }
    }

    fn active_source(&self) -> &Vec<BrowserEntry> {
        if !self.filter.is_empty() && self.recursive_loaded {
            &self.recursive_entries
        } else {
            &self.entries
        }
    }

    /// Reload directory contents, preserving filter and selection.
    pub fn refresh(&mut self) {
        let old_filter = self.filter.clone();
        let selected_name = self
            .filtered_indices
            .get(self.selected)
            .and_then(|&i| self.active_source().get(i))
            .map(|e| e.name.clone());

        self.load_dir();
        self.filter = old_filter;
        self.rebuild_filter();

        let source = self.active_source();
        if let Some(name) = selected_name {
            if let Some(pos) = self
                .filtered_indices
                .iter()
                .position(|&i| source[i].name == name)
            {
                self.selected = pos;
            } else {
                self.selected = self
                    .selected
                    .min(self.filtered_indices.len().saturating_sub(1));
            }
        }
    }

    pub fn adjust_scroll(&mut self, visible_height: usize) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected - visible_height + 1;
        }
    }
}
