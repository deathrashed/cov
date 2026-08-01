use crate::tui::app::App;
use crate::tui::widgets;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(app: &App, f: &mut Frame) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(area);

    // Input bar
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_focused_style())
        .title(" Search ")
        .title_alignment(ratatui::layout::Alignment::Left);

    let input_inner = input_block.inner(chunks[0]);
    f.render_widget(input_block, chunks[0]);

    if app.input.is_empty() {
        let placeholder = if app.is_scanning {
            "Type to search albums… (scanning library…)"
        } else {
            "Type to search albums…"
        };
        let line = Line::from(vec![
            Span::styled("> ", app.theme.footer_key_style()),
            Span::styled(placeholder, app.theme.footer_text_style()),
        ]);
        f.render_widget(Paragraph::new(line), input_inner);
    } else {
        let line = Line::from(vec![
            Span::styled("> ", app.theme.footer_key_style()),
            Span::styled(&app.input, app.theme.input_text_style()),
        ]);
        f.render_widget(Paragraph::new(line), input_inner);
    }

    // Position terminal cursor right after input text
    f.set_cursor_position((input_inner.x + 2 + app.cursor as u16, input_inner.y));

    // Main content: list + preview
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // Render album list widget
    widgets::album_list::draw(app, f, main_chunks[0]);

    // Render preview pane widget
    widgets::preview::draw(app, f, main_chunks[1]);

    // Render footer widget with status line and shortcut buttons
    widgets::footer::draw(app, f, chunks[2]);
}
