use crate::tui::app::App;
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(app: &App, f: &mut Frame) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style())
        .title(" Diagnostics (Esc to return) ");
    let inner = block.inner(f.area());
    f.render_widget(block, f.area());

    let checks = crate::doctor::run(&app.cfg);
    let mut text = String::new();
    for check in &checks {
        let status = if check.ok { "PASS" } else { "FAIL" };
        text.push_str(&format!("{status:<5} {}: {}\n", check.label, check.detail));
    }
    f.render_widget(
        Paragraph::new(text).style(app.theme.list_text_style()),
        inner,
    );
}
