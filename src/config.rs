use std::fs;
use std::path::PathBuf;

pub struct Config {
    pub theme: Option<String>,
    pub scrollbar: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: None,
            scrollbar: true,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|d| d.join(".config").join("mdr").join("config.toml"))
}

fn parse_toml_string(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn parse_toml_bool(value: &str) -> Option<bool> {
    match value.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn load_config() -> Config {
    let mut config = Config::default();
    let path = match config_path() {
        Some(p) => p,
        None => return config,
    };
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return config,
    };

    for line in contents.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "theme" => {
                    let v = parse_toml_string(value);
                    if !v.is_empty() {
                        config.theme = Some(v);
                    }
                }
                "scrollbar" => {
                    if let Some(b) = parse_toml_bool(value) {
                        config.scrollbar = b;
                    }
                }
                _ => {}
            }
        }
    }

    config
}

pub fn save_config(config: &Config) {
    if let Some(path) = config_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut contents = String::new();
        if let Some(ref theme) = config.theme {
            contents.push_str(&format!("theme = \"{theme}\"\n"));
        }
        contents.push_str(&format!("scrollbar = {}\n", config.scrollbar));
        let _ = fs::write(path, contents);
    }
}
