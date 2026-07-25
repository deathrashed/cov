use crate::tui::app::App;
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(app: &App, f: &mut Frame) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style())
        .title(" COVIT Log (Esc to return) ");
    let inner = block.inner(f.area());
    f.render_widget(block, f.area());

    let log_content =
        std::fs::read_to_string(&app.cfg.log_path).unwrap_or_else(|_| "No log yet.".to_string());
    let lines: Vec<&str> = log_content.lines().rev().take(200).collect();
    let text = lines.join("\n");
    f.render_widget(
        Paragraph::new(text).style(app.theme.list_text_style()),
        inner,
    );
}
