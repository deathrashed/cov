use crate::tui::app::App;
use ratatui::{
    Frame,
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(app: &App, f: &mut Frame) {
    let help_text = "\
Album picker:
  ↑ / ↓         Navigate album list by item
  PgUp / PgDn   Navigate album list by 10 items
  Home / End    Jump to start or end of list
  ^P / ^N       Navigate list
  Enter         Run the configured default action
  ^S            Save cover beside the selected album
  ^E            Open COV and embed its chosen cover
  ^F            Cycle: All -> Needs cover -> Needs embed
  ^R            Rebuild the library index
  ^O            Open persistent settings
  ?             Show this help screen
  Esc           Clear search input (or Quit if search is empty)
  q / ^C        Quit COV TUI

Other integrations remain available as commands:
  cov finder, cov swinsian, cov clipboard, cov choose";

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_focused_style())
        .title(" Help (Esc / Enter to return) ");
    let inner = block.inner(f.area());
    f.render_widget(block, f.area());
    f.render_widget(
        Paragraph::new(help_text).style(app.theme.list_text_style()),
        inner,
    );
}
