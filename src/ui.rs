use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use ratatui::Frame;

use crate::markdown::parse_markdown;
use crate::state::AppState;
use crate::theme::ALL_THEMES;

pub fn draw(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let theme = state.theme;

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .padding(Padding::horizontal(1));

    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let chunks = Layout::vertical([
        Constraint::Length(1), // title bar
        Constraint::Length(1), // separator
        Constraint::Min(1),   // content
        Constraint::Length(1), // status bar
    ])
    .split(inner);

    // Title bar
    let filename = state
        .file_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "no file".to_string());

    let title = Line::from(vec![
        Span::styled("meld", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(filename, Style::default().fg(theme.text)),
    ]);
    f.render_widget(Paragraph::new(title), chunks[0]);

    // Separator
    let sep = "─".repeat(chunks[1].width as usize);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            sep,
            Style::default().fg(theme.border),
        ))),
        chunks[1],
    );

    // Content area
    let content_area = chunks[2];
    let styled_lines = parse_markdown(&state.content, theme, content_area.width);
    let total_lines = styled_lines.len();

    let visible_height = content_area.height as usize;
    let scroll = state.scroll.min(total_lines.saturating_sub(visible_height));

    let visible: Vec<Line> = styled_lines
        .into_iter()
        .skip(scroll)
        .take(visible_height)
        .map(|sl| sl.line)
        .collect();

    f.render_widget(Paragraph::new(visible), content_area);

    // Status bar
    let theme_name = ALL_THEMES
        .get(state.theme_index)
        .map(|(name, _)| *name)
        .unwrap_or("unknown");

    let scroll_pct = if total_lines <= visible_height {
        100
    } else {
        ((scroll as f64 / (total_lines - visible_height) as f64) * 100.0) as usize
    };

    let status = Line::from(vec![
        Span::styled(
            format!(" {} ", theme_name),
            Style::default().fg(theme.accent),
        ),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(
            format!("{}%", scroll_pct),
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(
            "q:quit  j/k:scroll  t:theme",
            Style::default().fg(theme.text_muted),
        ),
    ]);
    f.render_widget(Paragraph::new(status), chunks[3]);
}
