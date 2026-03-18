use std::path::PathBuf;

use crate::theme::{default_theme, Theme, ALL_THEMES};

pub struct AppState {
    pub content: String,
    pub file_path: Option<PathBuf>,
    pub scroll: usize,
    pub theme: Theme,
    pub theme_index: usize,
}

impl AppState {
    pub fn new(file_path: Option<PathBuf>, content: String) -> Self {
        Self {
            content,
            file_path,
            scroll: 0,
            theme: default_theme(),
            theme_index: 5,
        }
    }

    pub fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % ALL_THEMES.len();
        self.theme = ALL_THEMES[self.theme_index].1;
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_add(n);
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn scroll_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_bottom(&mut self) {
        self.scroll = usize::MAX;
    }
}
