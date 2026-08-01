use crate::tui::app::App;
use crate::tui::images;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn draw(app: &App, f: &mut Frame, area: Rect) {
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style())
        .title(" Preview ");
    let inner = preview_block.inner(area);
    f.render_widget(preview_block, area);

    if let Some(album) = app.filtered.get(app.selected) {
        let status_text = app
            .statuses
            .get(&album.dir)
            .map(images::preview_text)
            .unwrap_or_else(|| "checking…".to_string());

        let track_count = album.tracks.len();
        let track_str = if track_count == 1 {
            "1 audio track".to_string()
        } else {
            format!("{} audio tracks", track_count)
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("Album:  ", app.theme.footer_key_style()),
                Span::styled(&album.display, app.theme.list_text_style()),
            ]),
            Line::from(vec![
                Span::styled("Tracks: ", app.theme.footer_key_style()),
                Span::styled(track_str, app.theme.list_text_style()),
            ]),
            Line::from(vec![
                Span::styled("Status: ", app.theme.footer_key_style()),
                Span::styled(status_text, app.theme.list_text_style()),
            ]),
            Line::from(vec![
                Span::styled("Path:   ", app.theme.footer_key_style()),
                Span::styled(
                    album.dir.to_string_lossy().to_string(),
                    app.theme.list_text_style(),
                ),
            ]),
        ];

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    } else {
        let placeholder = " Select an album from the list\n to preview artwork metadata.";
        f.render_widget(
            Paragraph::new(placeholder).style(app.theme.footer_text_style()),
            inner,
        );
    }
}
