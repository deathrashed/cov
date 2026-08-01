use crate::config::Mode;
use crate::tui::app::{App, SettingsField, SettingsState};
use ratatui::{
    Frame,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(app: &App, f: &mut Frame, settings: &SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_focused_style())
        .title(" Settings ");
    let inner = block.inner(f.area());
    f.render_widget(block, f.area());

    let action = match settings.default_mode {
        Mode::Save => "Save cover beside album",
        Mode::Embed => "Save and embed cover",
    };
    let resolution = if settings.resolution.is_empty() {
        "COV default"
    } else {
        settings.resolution.as_str()
    };
    let providers = if settings.sources.is_empty() {
        "COV defaults"
    } else {
        settings.sources.as_str()
    };
    let rows = [
        (
            SettingsField::Library,
            "Library folder",
            settings.library_root.as_str(),
        ),
        (SettingsField::DefaultAction, "Enter action", action),
        (
            SettingsField::OutputName,
            "Cover filename",
            settings.output_basename.as_str(),
        ),
        (SettingsField::Resolution, "Minimum resolution", resolution),
        (SettingsField::Providers, "Providers", providers),
    ];
    let mut lines = vec![
        Line::from(" Changes are staged until you press Enter."),
        Line::default(),
    ];
    for (field, label, value) in rows {
        let marker = if settings.focus == field { "›" } else { " " };
        let style = if settings.focus == field {
            app.theme.selected_style()
        } else {
            app.theme.list_text_style()
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {marker} {label:<20} "), style),
            Span::styled(value, style),
        ]));
    }
    lines.push(Line::default());
    lines.push(Line::from(
        " Tab/↑↓: select   Space/←→: action   Enter: save   Esc: cancel",
    ));
    f.render_widget(
        Paragraph::new(lines).style(app.theme.list_text_style()),
        inner,
    );
}
