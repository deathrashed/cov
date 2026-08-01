use crate::tui::app::App;
use crate::tui::artwork::{Badge, Filter};
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(app: &App, f: &mut Frame, area: Rect) {
    let filter_label = match app.filter {
        Filter::All => "All".to_string(),
        Filter::Missing => "Needs Cover".to_string(),
        Filter::NeedsEmbed => "Needs Embed".to_string(),
    };

    let title = if app.filtered.len() == app.albums.len() {
        format!(" Albums ({}) ", app.filtered.len())
    } else {
        format!(
            " Albums ({}/{} · {}) ",
            app.filtered.len(),
            app.albums.len(),
            filter_label
        )
    };

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style())
        .title(title);
    let list_inner = list_block.inner(area);
    f.render_widget(list_block, area);

    if app.filtered.is_empty() {
        let msg = if app.is_scanning && app.albums.is_empty() {
            " Scanning music library for albums…"
        } else if app.albums.is_empty() {
            " No albums found in configured library directory.\n Press ^R to rescan."
        } else {
            " No albums match current search / filter criteria.\n Press Esc to clear search or ^F to cycle filter."
        };

        f.render_widget(
            Paragraph::new(msg).style(app.theme.footer_text_style()),
            Rect::new(
                list_inner.x + 1,
                list_inner.y + 1,
                list_inner.width.saturating_sub(2),
                2,
            ),
        );
        return;
    }

    let list_start = app.selected.saturating_sub(10);
    let visible: Vec<_> = app.filtered.iter().skip(list_start).take(20).collect();
    for (y, (i, album)) in (list_inner.y..).zip(visible.iter().enumerate()) {
        let abs_idx = list_start + i;
        let style = if abs_idx == app.selected {
            app.theme.selected_style()
        } else {
            app.theme.list_text_style()
        };
        let badge = app
            .statuses
            .get(&album.dir)
            .map(|s| s.badge())
            .unwrap_or(Badge::Checking);
        let badge_style = app.theme.badge_style(badge).patch(style);
        let spans = vec![
            Span::styled(badge.glyph().to_string(), badge_style),
            Span::raw(" "),
            Span::styled(&album.display, style),
        ];
        f.render_widget(
            Paragraph::new(Line::from(spans)).style(style),
            Rect::new(list_inner.x, y, list_inner.width, 1),
        );
    }
}
