use std::fs;
use std::io;
use std::path::PathBuf;

use crate::browser::BrowserState;
use crate::theme::{default_theme, Theme, ALL_THEMES};

pub enum AppMode {
    Reader,
    Search,
    FilePicker,
    ThemePicker { original_index: usize },
    Help,
}

pub struct AppState {
    pub mode: AppMode,
    pub content: String,
    pub file_path: Option<PathBuf>,
    pub scroll: usize,
    pub total_lines: usize,
    pub visible_height: usize,
    pub theme: Theme,
    pub theme_index: usize,
    pub browser: BrowserState,
    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub search_current: usize,
}

impl AppState {
    pub fn new_picker(dir: PathBuf) -> Self {
        Self {
            mode: AppMode::FilePicker,
            content: String::new(),
            file_path: None,
            scroll: 0,
            total_lines: 0,
            visible_height: 0,
            theme: default_theme(),
            theme_index: 5,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_current: 0,
            browser: BrowserState::new(dir),
        }
    }

    pub fn new_reader(file_path: PathBuf, content: String) -> Self {
        let browser_dir = file_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            mode: AppMode::Reader,
            content,
            file_path: Some(file_path),
            scroll: 0,
            total_lines: 0,
            visible_height: 0,
            theme: default_theme(),
            theme_index: 5,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_current: 0,
            browser: BrowserState::new(browser_dir),
        }
    }

    pub fn open_file(&mut self, path: PathBuf) -> io::Result<()> {
        let content = fs::read_to_string(&path)?;
        self.content = content;
        self.file_path = Some(path);
        self.scroll = 0;
        self.mode = AppMode::Reader;
        Ok(())
    }

    pub fn open_theme_picker(&mut self) {
        match self.mode {
            AppMode::ThemePicker { .. } | AppMode::Help => return,
            _ => {}
        }
        self.mode = AppMode::ThemePicker {
            original_index: self.theme_index,
        };
    }

    pub fn theme_picker_select(&mut self, index: usize) {
        self.theme_index = index;
        self.theme = ALL_THEMES[index].1;
    }

    pub fn theme_picker_confirm(&mut self) {
        if matches!(self.mode, AppMode::ThemePicker { .. }) {
            self.mode = AppMode::Reader;
        }
    }

    pub fn theme_picker_cancel(&mut self) {
        if let AppMode::ThemePicker { original_index } = self.mode {
            self.theme_index = original_index;
            self.theme = ALL_THEMES[original_index].1;
            self.mode = AppMode::Reader;
        }
    }

    pub fn open_help(&mut self) {
        match self.mode {
            AppMode::ThemePicker { .. } | AppMode::Help => return,
            _ => {}
        }
        self.mode = AppMode::Help;
    }

    pub fn close_help(&mut self) {
        if matches!(self.mode, AppMode::Help) {
            self.mode = AppMode::Reader;
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        let max = self.total_lines.saturating_sub(self.visible_height);
        self.scroll = self.scroll.saturating_add(n).min(max);
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn scroll_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_bottom(&mut self) {
        self.scroll = self.total_lines.saturating_sub(self.visible_height);
    }

    pub fn open_search(&mut self) {
        self.search_query.clear();
        self.search_matches.clear();
        self.search_current = 0;
        self.mode = AppMode::Search;
    }

    pub fn close_search(&mut self) {
        self.search_query.clear();
        self.search_matches.clear();
        self.search_current = 0;
        self.mode = AppMode::Reader;
    }

    pub fn update_search(&mut self) {
        self.search_matches.clear();
        self.search_current = 0;
        if self.search_query.is_empty() {
            return;
        }
        let query_lower = self.search_query.to_lowercase();
        for (i, line) in self.content.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                self.search_matches.push(i);
            }
        }
    }

    pub fn search_next(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.search_current = (self.search_current + 1) % self.search_matches.len();
        self.scroll_to_match();
    }

    pub fn search_prev(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        if self.search_current == 0 {
            self.search_current = self.search_matches.len() - 1;
        } else {
            self.search_current -= 1;
        }
        self.scroll_to_match();
    }

    pub fn search_first(&mut self) {
        if !self.search_matches.is_empty() {
            if let Some(idx) = self.search_matches.iter().position(|&l| l >= self.scroll) {
                self.search_current = idx;
            } else {
                self.search_current = 0;
            }
            self.scroll_to_match();
        }
    }

    fn scroll_to_match(&mut self) {
        if let Some(&line) = self.search_matches.get(self.search_current) {
            self.scroll = line.saturating_sub(self.visible_height / 3);
        }
    }
}
