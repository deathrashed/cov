use crate::config::Mode;
use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn draw(app: &App, f: &mut Frame, area: Rect) {
    let theme = &app.theme;

    let default_action = match app.cfg.default_mode {
        Mode::Save => "save cover",
        Mode::Embed => "save + embed",
    };
    let items = [
        ("Enter", default_action),
        ("^S", "save cover"),
        ("^E", "open + embed"),
        ("^F", "filter"),
        ("^R", "refresh index"),
        ("^O", "settings"),
        ("?", "help"),
        ("q", "quit"),
    ];

    let mut shortcut_spans = Vec::new();
    shortcut_spans.push(Span::raw(" "));
    for (key, label) in items {
        shortcut_spans.push(Span::styled(key, theme.footer_key_style()));
        shortcut_spans.push(Span::styled(
            format!(":{}  ", label),
            theme.footer_text_style(),
        ));
    }

    let mut lines = Vec::new();

    if !app.status_line.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(" ", theme.footer_key_style()),
            Span::styled(&app.status_line, theme.footer_key_style()),
        ]));
    }

    lines.push(Line::from(shortcut_spans));

    f.render_widget(Paragraph::new(lines), area);
}
