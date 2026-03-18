mod markdown;
mod state;
mod theme;
mod ui;

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use state::AppState;

enum AppEvent {
    FileChanged,
    Terminal(Event),
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: meld <file.md>");
        std::process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]).canonicalize().map_err(|e| {
        eprintln!("error: {}: {}", args[1], e);
        e
    })?;

    let content = fs::read_to_string(&file_path)?;
    let mut state = AppState::new(Some(file_path.clone()), content);

    // Event channel
    let (tx, rx) = mpsc::channel::<AppEvent>();

    // File watcher
    let tx_watcher = tx.clone();
    let watch_path = file_path.clone();
    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    let _ = tx_watcher.send(AppEvent::FileChanged);
                }
            }
        })
        .expect("failed to create file watcher");

    watcher
        .watch(
            watch_path.parent().unwrap_or(&watch_path),
            RecursiveMode::NonRecursive,
        )
        .expect("failed to watch file");

    // Terminal input thread
    let tx_input = tx.clone();
    std::thread::spawn(move || loop {
        if let Ok(ev) = event::read() {
            let _ = tx_input.send(AppEvent::Terminal(ev));
        }
    });

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    loop {
        terminal.draw(|f| ui::draw(f, &state))?;

        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(AppEvent::FileChanged) => {
                if let Ok(new_content) = fs::read_to_string(&file_path) {
                    state.content = new_content;
                }
            }
            Ok(AppEvent::Terminal(Event::Key(key))) => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char('j') | KeyCode::Down => state.scroll_down(1),
                KeyCode::Char('k') | KeyCode::Up => state.scroll_up(1),
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.scroll_down(20)
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.scroll_up(20)
                }
                KeyCode::Char('g') => state.scroll_top(),
                KeyCode::Char('G') => state.scroll_bottom(),
                KeyCode::Char('t') => state.cycle_theme(),
                _ => {}
            },
            Ok(AppEvent::Terminal(Event::Resize(_, _))) => {}
            _ => {}
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
