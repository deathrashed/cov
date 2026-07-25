use crate::tui::app::App;
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(app: &App, f: &mut Frame) {
    let help_text = "\
Target Picker Shortcuts:
  ^S            Get current playing/selected track in Swinsian and launch COV
  ^W            Get currently selected item in macOS Finder and launch COV
  ^K            Get path currently in macOS Clipboard and launch COV
  ^P            Open native macOS Folder Chooser window to pick any directory

Finder View Navigation:
  ↑ / ↓         Navigate album list by item
  PgUp / PgDn   Navigate album list by 10 items
  Home / End    Jump to start or end of list
  ^P / ^N       Navigate list (Ctrl+P / Ctrl+N or Ctrl+K / Ctrl+J)
  Enter         Save cover beside album tracks and open COV
  ^E            Save & embed cover into audio tags, then open COV
  ^F            Cycle filter: All -> Missing -> Needs Embed
  ^R            Rescan music library
  ^O            Open search overrides form
  ^L            View COVIT log output
  ^D            Run diagnostic checks
  ?             Show this help screen
  Esc           Clear search input (or Quit if search is empty)
  q / ^C        Quit COV TUI

Overrides Form (^O):
  Tab / Down    Move focus to next input field
  Shift+Tab / Up Move focus to previous input field
  Space         Toggle Save / Embed mode (when Mode is focused)
  Enter         Launch COV with specified search overrides
  Esc           Cancel form and return to Finder";

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
