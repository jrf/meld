use std::fs;
use std::path::PathBuf;

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("mdr").join("config"))
}

pub fn load_theme_name() -> Option<String> {
    let path = config_path()?;
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn save_theme_name(name: &str) {
    if let Some(path) = config_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, name);
    }
}
