use crate::tui::app::{App, FormState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(app: &App, f: &mut Frame, form: &FormState) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_focused_style())
        .title(" Overrides Form ");
    let inner = block.inner(chunks[0]);
    f.render_widget(block, chunks[0]);

    let fields = [
        ("Mode", if form.embed_mode { "embed" } else { "save" }),
        ("Artist", &form.artist),
        ("Album", &form.album),
        ("Identifier", &form.identifier),
        ("Country", &form.country),
        ("Resolution", &form.resolution),
        ("Sources", &form.sources),
    ];

    for (y, (i, (label, value))) in (inner.y..).zip(fields.iter().enumerate()) {
        let style = if i == form.focus {
            app.theme.selected_style()
        } else {
            app.theme.list_text_style()
        };
        let line = format!("{:<12}: {}", label, value);
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(line, style)])),
            Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1),
        );
    }

    // Form Footer
    let footer_text =
        " Tab/↑/↓:navigate  Space:toggle mode  Enter:launch with overrides  Esc:cancel ";
    f.render_widget(
        Paragraph::new(Line::from(
            footer_text
                .split(':')
                .enumerate()
                .flat_map(|(i, part)| {
                    if i % 2 == 0 {
                        vec![Span::styled(
                            part.to_string(),
                            app.theme.footer_key_style(),
                        )]
                    } else {
                        vec![Span::styled(
                            format!("{} ", part),
                            app.theme.footer_text_style(),
                        )]
                    }
                })
                .collect::<Vec<_>>(),
        )),
        chunks[1],
    );
}
