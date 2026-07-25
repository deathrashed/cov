use crate::config::Config;

pub mod artwork;
pub mod cache;
pub mod images;
pub mod matcher;
pub mod scanner;
pub mod theme;

mod app;
pub mod screens;
pub mod widgets;

/// Run the TUI event loop. Called by `cov tui`.
pub fn run(cfg: Config, debug: bool) -> anyhow::Result<()> {
    let mut app = app::App::new(cfg)?;
    app.run(debug)
}
