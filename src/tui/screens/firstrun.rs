use crate::tui::app::App;
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(app: &App, f: &mut Frame) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_focused_style())
        .title(" Welcome to COV Toolkit! ");
    let inner = block.inner(f.area());
    f.render_widget(block, f.area());

    let text = format!(
        "No music library configured yet.\n\n\
         Enter the path to your music library below and press Enter.\n\n\
         > {}",
        app.input
    );
    f.render_widget(
        Paragraph::new(text).style(app.theme.input_text_style()),
        inner,
    );
}
