use crate::config::Config;
use crate::launcher::{self, LaunchOptions};
use crate::tui::artwork::{ArtworkStatus, Badge, Filter};
use crate::tui::images;
use crate::tui::matcher::AlbumMatcher;
use crate::tui::scanner::{self, Album, ScanMsg};
use crate::tui::theme::Theme;
use crossbeam::channel::{self, Receiver, Sender};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Screen states for the TUI state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    FirstRun,
    Finder,
    Form(FormState),
    Log,
    Doctor,
    Help,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FormState {
    pub embed_mode: bool,
    pub artist: String,
    pub album: String,
    pub identifier: String,
    pub country: String,
    pub resolution: String,
    pub sources: String,
    pub focus: usize, // 0..6
}

/// Actions produced by the reducer for the event loop to execute.
#[derive(Debug)]
pub enum Action {
    None,
    Quit,
    Rescan,
    Launch { album_dir: PathBuf, embed: bool },
}

/// The main TUI application state machine.
pub struct App {
    pub cfg: Config,
    pub theme: Theme,
    pub albums: Vec<Arc<Album>>,
    pub filtered: Vec<Arc<Album>>,
    pub matcher: AlbumMatcher,
    pub input: String,
    pub cursor: usize,
    pub selected: usize,
    pub statuses: HashMap<PathBuf, ArtworkStatus>,
    pub filter: Filter,
    pub screen: Screen,
    pub scan_epoch: Arc<AtomicU64>,
    pub status_line: String,
    pub is_scanning: bool,
    // Channels
    pub scan_tx: Option<Sender<ScanMsg>>,
    pub scan_rx: Receiver<ScanMsg>,
}

impl App {
    /// Create a new App from config. Starts the scanner if library_root is set.
    pub fn new(cfg: Config) -> anyhow::Result<Self> {
        let theme = Theme::load(&cfg.theme);
        let (scan_tx, scan_rx) = channel::unbounded();

        let has_root = cfg
            .library_root
            .as_ref()
            .map(|p| p.exists())
            .unwrap_or(false);

        let mut app = App {
            theme,
            cfg,
            albums: Vec::new(),
            filtered: Vec::new(),
            matcher: AlbumMatcher::new(),
            input: String::new(),
            cursor: 0,
            selected: 0,
            statuses: HashMap::new(),
            filter: Filter::All,
            screen: if has_root {
                Screen::Finder
            } else {
                Screen::FirstRun
            },
            scan_epoch: Arc::new(AtomicU64::new(0)),
            status_line: String::new(),
            is_scanning: false,
            scan_tx: Some(scan_tx),
            scan_rx,
        };

        // Start initial scan if we have a library root
        if app.screen != Screen::FirstRun {
            app.start_scan();
        }

        Ok(app)
    }

    /// Start (or restart) the recursive library scan.
    pub fn start_scan(&mut self) {
        let epoch = self.scan_epoch.fetch_add(1, Ordering::Relaxed) + 1;
        let cancel = self.scan_epoch.clone();
        let tx = self.scan_tx.clone().unwrap();

        if let Some(ref root) = self.cfg.library_root {
            self.is_scanning = true;
            self.status_line = "Scanning library…".to_string();
            scanner::spawn_scan(root.clone(), epoch, cancel, tx);
        }
    }

    /// Handle a scan message.
    pub fn handle_scan(&mut self, msg: ScanMsg) {
        match msg {
            ScanMsg::Batch { ref albums, epoch } => {
                if epoch != self.scan_epoch.load(Ordering::Relaxed) {
                    return; // stale
                }
                self.albums.extend(albums.iter().cloned().map(Arc::new));
                // Re-run query with current input
                self.matcher.replace_items(self.albums.clone());
                self.matcher.query(&self.input);
                self.filtered = self.matcher.results();
            }
            ScanMsg::Done { epoch, total } => {
                if epoch != self.scan_epoch.load(Ordering::Relaxed) {
                    return;
                }
                self.is_scanning = false;
                self.status_line = format!("{} albums", total);
            }
        }
    }

    /// Reduce a message, possibly producing an action.
    pub fn reduce(&mut self, msg: AppMsg) -> Action {
        match msg {
            AppMsg::Key(key) => self.handle_key(key),
            AppMsg::Scan(sm) => {
                self.handle_scan(sm);
                Action::None
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match &self.screen {
            Screen::Finder => self.handle_finder_key(key),
            Screen::Form(_) => {
                let mut form = match std::mem::replace(&mut self.screen, Screen::Finder) {
                    Screen::Form(f) => f,
                    _ => unreachable!(),
                };
                let leaves_form = matches!(key.code, KeyCode::Enter | KeyCode::Esc);
                let action = Self::handle_form_key(&mut form, key);
                if !leaves_form {
                    self.screen = Screen::Form(form);
                }
                action
            }
            Screen::Help | Screen::Log | Screen::Doctor => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.screen = Screen::Finder;
                    Action::None
                }
                _ => Action::None,
            },
            Screen::FirstRun => match key.code {
                KeyCode::Enter => {
                    // Accept library root, move to finder
                    let path = PathBuf::from(self.input.trim());
                    if path.exists() && path.is_dir() {
                        self.cfg.library_root = Some(path);
                        self.screen = Screen::Finder;
                        self.start_scan();
                    }
                    Action::None
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                    Action::None
                }
                KeyCode::Backspace => {
                    self.input.pop();
                    Action::None
                }
                KeyCode::Esc => Action::Quit,
                _ => Action::None,
            },
        }
    }

    fn handle_finder_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.input.is_empty() {
                    return Action::Quit;
                }
                // Clear input on first Esc
                self.input.clear();
                self.cursor = 0;
                self.matcher.query("");
                self.filtered = self.matcher.results();
                Action::None
            }
            KeyCode::Up | KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                if self.selected + 1 < self.filtered.len() {
                    self.selected += 1;
                }
                Action::None
            }
            KeyCode::Enter => {
                // Save mode: open COV
                if let Some(album) = self.filtered.get(self.selected) {
                    self.status_line = format!("COV opened for {}", album.display);
                    return Action::Launch {
                        album_dir: album.dir.clone(),
                        embed: false,
                    };
                }
                Action::None
            }
            KeyCode::Char('e') if key.modifiers == KeyModifiers::CONTROL => {
                // Embed mode: open COV with embed callback
                if let Some(album) = self.filtered.get(self.selected) {
                    self.status_line = format!("COV opened (embed) for {}", album.display);
                    return Action::Launch {
                        album_dir: album.dir.clone(),
                        embed: true,
                    };
                }
                Action::None
            }
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                self.filter = self.filter.next();
                self.apply_filter();
                Action::None
            }
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                self.albums.clear();
                self.filtered.clear();
                self.statuses.clear();
                self.selected = 0;
                Action::Rescan
            }
            KeyCode::Char('o') if key.modifiers == KeyModifiers::CONTROL => {
                self.screen = Screen::Form(FormState::default());
                Action::None
            }
            KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                self.screen = Screen::Log;
                Action::None
            }
            KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                self.screen = Screen::Doctor;
                Action::None
            }
            KeyCode::Char('?') => {
                self.screen = Screen::Help;
                Action::None
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                self.cursor = self.input.len();
                self.matcher.query(&self.input);
                self.filtered = self.matcher.results();
                self.selected = 0;
                Action::None
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.cursor = self.input.len();
                self.matcher.query(&self.input);
                self.filtered = self.matcher.results();
                self.selected = 0;
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_form_key(form: &mut FormState, key: KeyEvent) -> Action {
        let fields = 7; // mode + 6 text fields
        match key.code {
            KeyCode::Tab => {
                form.focus = (form.focus + 1) % fields;
                Action::None
            }
            KeyCode::BackTab => {
                form.focus = if form.focus == 0 {
                    fields - 1
                } else {
                    form.focus - 1
                };
                Action::None
            }
            KeyCode::Enter => Action::None,
            KeyCode::Esc => Action::None,
            KeyCode::Char(' ') if form.focus == 0 => {
                form.embed_mode = !form.embed_mode;
                Action::None
            }
            KeyCode::Char(c) => {
                let field = match form.focus {
                    1 => &mut form.artist,
                    2 => &mut form.album,
                    3 => &mut form.identifier,
                    4 => &mut form.country,
                    5 => &mut form.resolution,
                    6 => &mut form.sources,
                    _ => return Action::None,
                };
                field.push(c);
                Action::None
            }
            KeyCode::Backspace => {
                let field = match form.focus {
                    1 => &mut form.artist,
                    2 => &mut form.album,
                    3 => &mut form.identifier,
                    4 => &mut form.country,
                    5 => &mut form.resolution,
                    6 => &mut form.sources,
                    _ => return Action::None,
                };
                field.pop();
                Action::None
            }
            _ => Action::None,
        }
    }

    fn apply_filter(&mut self) {
        self.filtered = self
            .matcher
            .results()
            .into_iter()
            .filter(|album| {
                let badge = self
                    .statuses
                    .get(&album.dir)
                    .map(|s| s.badge())
                    .unwrap_or(Badge::Checking);
                self.filter.allows(badge)
            })
            .collect();
        self.selected = 0;
    }

    /// Run the main TUI event loop.
    pub fn run(&mut self, debug: bool) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stderr = std::io::stderr();
        execute!(stderr, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stderr);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        // Optional debug log
        if debug {
            // Could set up tracing here
        }

        // Draw loop
        let res = self.event_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        res
    }

    fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stderr>>,
    ) -> anyhow::Result<()> {
        loop {
            // Poll for events with 50ms timeout (allows scan messages to be processed)
            if event::poll(std::time::Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        let action = self.reduce(AppMsg::Key(key));
                        match action {
                            Action::Quit => break,
                            Action::Rescan => {
                                self.start_scan();
                            }
                            Action::Launch { album_dir, embed } => {
                                let path = album_dir.to_string_lossy().to_string();
                                let opts = LaunchOptions {
                                    path,
                                    embed,
                                    output: "cover".to_string(),
                                    covit: self.cfg.covit_path.clone(),
                                    log: self.cfg.log_path.clone(),
                                    artist: None,
                                    album: None,
                                    identifier: None,
                                    country: None,
                                    resolution: None,
                                    sources: None,
                                    foreground: false,
                                };
                                // Launch in background; ignore errors (logged in launcher)
                                launcher::launch(&opts).ok();
                            }
                            Action::None => {}
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }

            // Check scan channel
            if let Ok(msg) = self.scan_rx.try_recv() {
                self.reduce(AppMsg::Scan(msg));
            }

            // Draw
            terminal.draw(|f| {
                self.draw(f);
            })?;
        }

        Ok(())
    }

    /// Draw the current screen.
    fn draw(&self, f: &mut ratatui::Frame) {
        match self.screen {
            Screen::Finder => self.draw_finder(f),
            Screen::Form(ref form) => self.draw_form(f, form),
            Screen::Log => self.draw_log(f),
            Screen::Doctor => self.draw_doctor(f),
            Screen::Help => self.draw_help(f),
            Screen::FirstRun => self.draw_firstrun(f),
        }
    }

    fn draw_finder(&self, f: &mut ratatui::Frame) {
        let area = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);

        // Input bar
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_focused_style())
            .title(format!(" {} ", self.filter.glyph()))
            .title_alignment(ratatui::layout::Alignment::Right);
        let input_inner = input_block.inner(chunks[0]);
        f.render_widget(input_block, chunks[0]);

        let input_text = if self.input.is_empty() && !self.is_scanning {
            format!("> {} | {} albums", self.input, self.filtered.len())
        } else if self.is_scanning {
            format!("> {} | scanning…", self.input)
        } else {
            format!(
                "> {} | {} / {} albums",
                self.input,
                self.filtered.len(),
                self.albums.len()
            )
        };
        f.render_widget(
            Paragraph::new(input_text).style(self.theme.input_text_style()),
            input_inner,
        );

        // Main content: list + preview
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        // Album list
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_style())
            .title(format!(" {} ", self.filter.glyph()));
        let list_inner = list_block.inner(main_chunks[0]);
        f.render_widget(list_block, main_chunks[0]);

        let list_start = self.selected.saturating_sub(10);
        let visible: Vec<_> = self.filtered.iter().skip(list_start).take(20).collect();
        for (y, (i, album)) in (list_inner.y..).zip(visible.iter().enumerate()) {
            let abs_idx = list_start + i;
            let style = if abs_idx == self.selected {
                self.theme.selected_style()
            } else {
                self.theme.list_text_style()
            };
            let badge = self
                .statuses
                .get(&album.dir)
                .map(|s| s.badge())
                .unwrap_or(Badge::Checking);
            let badge_style = self.theme.badge_style(badge);
            let spans = vec![
                Span::styled(badge.glyph().to_string(), badge_style),
                Span::raw(" "),
                Span::styled(&album.display, style),
            ];
            f.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect::new(list_inner.x, y, list_inner.width, 1),
            );
        }

        // Preview pane
        let preview_block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_style())
            .title(" Preview ");
        f.render_widget(preview_block, main_chunks[1]);

        // Preview content
        if let Some(album) = self.filtered.get(self.selected)
            && let Some(status) = self.statuses.get(&album.dir)
        {
            let preview = images::preview_text(status);
            f.render_widget(
                Paragraph::new(preview).style(self.theme.list_text_style()),
                Rect::new(
                    main_chunks[1].x + 1,
                    main_chunks[1].y + 1,
                    main_chunks[1].width.saturating_sub(2),
                    3,
                ),
            );
        }

        // Footer
        let footer_text = " enter:save  ^E:embed  ^O:options  ^F:filter  ^R:rescan  ^L:log  ^D:doctor  ?:help  q:quit ";
        f.render_widget(
            Paragraph::new(Line::from(
                footer_text
                    .split(':')
                    .enumerate()
                    .flat_map(|(i, part)| {
                        if i % 2 == 0 {
                            vec![Span::styled(
                                part.to_string(),
                                self.theme.footer_key_style(),
                            )]
                        } else {
                            vec![Span::styled(
                                format!("{} ", part),
                                self.theme.footer_text_style(),
                            )]
                        }
                    })
                    .collect::<Vec<_>>(),
            )),
            chunks[2],
        );
    }

    fn draw_form(&self, f: &mut ratatui::Frame, form: &FormState) {
        let area = f.area();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_focused_style())
            .title(" Overrides (Tab/Shift+Tab to navigate, Enter to confirm, Esc to cancel) ");
        let inner = block.inner(area);
        f.render_widget(block, area);

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
                self.theme.selected_style()
            } else {
                self.theme.list_text_style()
            };
            let line = format!("{}: {}", label, value);
            f.render_widget(
                Paragraph::new(Line::from(vec![Span::styled(line, style)])),
                Rect::new(inner.x, y, inner.width, 1),
            );
        }
    }

    fn draw_log(&self, f: &mut ratatui::Frame) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_style())
            .title(" COVIT Log (Esc to return) ");
        let inner = block.inner(f.area());
        f.render_widget(block, f.area());

        let log_content = std::fs::read_to_string(&self.cfg.log_path)
            .unwrap_or_else(|_| "No log yet.".to_string());
        let lines: Vec<&str> = log_content.lines().rev().take(200).collect();
        let text = lines.join("\n");
        f.render_widget(
            Paragraph::new(text).style(self.theme.list_text_style()),
            inner,
        );
    }

    fn draw_doctor(&self, f: &mut ratatui::Frame) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_style())
            .title(" Diagnostics (Esc to return) ");
        let inner = block.inner(f.area());
        f.render_widget(block, f.area());

        let checks = crate::doctor::run(&self.cfg);
        let mut text = String::new();
        for check in &checks {
            let status = if check.ok { "PASS" } else { "FAIL" };
            text.push_str(&format!("{status:<5} {}: {}\n", check.label, check.detail));
        }
        f.render_widget(
            Paragraph::new(text).style(self.theme.list_text_style()),
            inner,
        );
    }

    fn draw_help(&self, f: &mut ratatui::Frame) {
        let help_text = "\
Keybindings:
  ↑/↓ or ^P/^N  Navigate album list
  Enter         Save cover and open COV
  ^E            Save & embed cover, open COV
  ^F            Cycle filter: All -> Missing -> Needs Embed
  ^R            Rescan library
  ^O            Open overrides form
  ^L            View COVIT log
  ^D            Run diagnostics
  ?             Show this help
  Esc           Back / clear input
  q / ^C        Quit
  Type          Filter albums by name";
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_focused_style())
            .title(" Help (Esc to return) ");
        let inner = block.inner(f.area());
        f.render_widget(block, f.area());
        f.render_widget(
            Paragraph::new(help_text).style(self.theme.list_text_style()),
            inner,
        );
    }

    fn draw_firstrun(&self, f: &mut ratatui::Frame) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_focused_style())
            .title(" Welcome to COV Toolkit! ");
        let inner = block.inner(f.area());
        f.render_widget(block, f.area());

        let text = format!(
            "No music library configured yet.\n\n\
             Enter the path to your music library below and press Enter.\n\n\
             > {}",
            self.input
        );
        f.render_widget(
            Paragraph::new(text).style(self.theme.input_text_style()),
            inner,
        );
    }
}

/// Messages that can be sent to the App reducer.
pub enum AppMsg {
    Key(KeyEvent),
    Scan(ScanMsg),
}

impl Filter {
    fn glyph(&self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Missing => "Missing",
            Filter::NeedsEmbed => "Needs Embed",
        }
    }
}
