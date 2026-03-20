use std::path::{Path, PathBuf};
use std::process::Command;

/// Fuzzy match with scoring. Returns None if no match, Some(score) if matched.
/// Higher score = better match.
fn fuzzy_score(text: &str, query: &str) -> Option<i32> {
    let text_chars: Vec<char> = text.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();

    if query_chars.is_empty() {
        return Some(0);
    }

    let mut score: i32 = 0;
    let mut ti = 0;
    let mut prev_match_idx: Option<usize> = None;

    for &qc in &query_chars {
        let mut found = false;
        while ti < text_chars.len() {
            if text_chars[ti] == qc {
                // Consecutive match bonus
                if let Some(prev) = prev_match_idx {
                    if ti == prev + 1 {
                        score += 10;
                    }
                }
                // Word boundary bonus (start of text, after '/', '-', '_', '.')
                if ti == 0 || matches!(text_chars[ti - 1], '/' | '-' | '_' | '.') {
                    score += 8;
                }
                // Penalty for gap
                if let Some(prev) = prev_match_idx {
                    let gap = ti - prev - 1;
                    score -= gap as i32;
                } else {
                    // Penalty for late first match
                    score -= ti as i32;
                }
                prev_match_idx = Some(ti);
                ti += 1;
                found = true;
                break;
            }
            ti += 1;
        }
        if !found {
            return None;
        }
    }

    // Bonus for shorter text (prefer tighter matches)
    score -= (text_chars.len() as i32) / 4;

    Some(score)
}

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
    /// Cached recursive .md files from git root (populated on first filter use)
    recursive_entries: Vec<BrowserEntry>,
    recursive_root: Option<PathBuf>,
    recursive_loaded: bool,
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
            recursive_root: None,
            recursive_loaded: false,
        };
        state.load_dir();
        state
    }

    pub fn load_dir(&mut self) {
        self.entries.clear();
        self.filter.clear();
        self.recursive_loaded = false;

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

    pub fn preload_recursive(&mut self) {
        self.ensure_recursive_loaded();
    }

    fn ensure_recursive_loaded(&mut self) {
        if self.recursive_loaded {
            return;
        }
        self.recursive_loaded = true;
        self.recursive_entries.clear();

        let root = find_git_root(&self.current_dir)
            .unwrap_or_else(|| self.current_dir.clone());
        let files = collect_md_files(&root);

        self.recursive_entries = files
            .into_iter()
            .map(|(rel, path)| BrowserEntry {
                name: rel,
                path,
                is_dir: false,
            })
            .collect();
        self.recursive_root = Some(root);
    }

    pub fn rebuild_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_indices = (0..self.entries.len()).collect();
        } else {
            self.ensure_recursive_loaded();
            let query = self.filter.to_lowercase();
            let mut scored: Vec<(usize, i32)> = self
                .recursive_entries
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    fuzzy_score(&e.name.to_lowercase(), &query).map(|s| (i, s))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_indices = scored.into_iter().map(|(i, _)| i).collect();
        }
    }

    pub fn filtered_entries(&self) -> Vec<(usize, &BrowserEntry)> {
        let source = if self.filter.is_empty() {
            &self.entries
        } else {
            &self.recursive_entries
        };
        self.filtered_indices
            .iter()
            .map(|&i| (i, &source[i]))
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
        let source = if self.filter.is_empty() {
            &self.entries
        } else {
            &self.recursive_entries
        };
        let entry = &source[real_index];
        if entry.is_dir {
            self.current_dir = entry.path.clone();
            self.load_dir();
            None
        } else {
            Some(entry.path.clone())
        }
    }

    /// Reload directory contents, preserving filter and selection.
    pub fn refresh(&mut self) {
        let old_filter = self.filter.clone();
        let source = if self.filter.is_empty() {
            &self.entries
        } else {
            &self.recursive_entries
        };
        let selected_name = self
            .filtered_indices
            .get(self.selected)
            .and_then(|&i| source.get(i))
            .map(|e| e.name.clone());

        self.load_dir();
        self.filter = old_filter;
        self.rebuild_filter();

        let source = if self.filter.is_empty() {
            &self.entries
        } else {
            &self.recursive_entries
        };
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
