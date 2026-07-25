use crate::config::Config;
use crate::launcher::{self, LaunchOptions};
use crate::macos;
use crate::tui::artwork::{self, ArtworkMsg, ArtworkStatus, Badge, Filter, InspectJob};
use crate::tui::cache;
use crate::tui::matcher::AlbumMatcher;
use crate::tui::scanner::{self, Album, ScanMsg};
use crate::tui::screens;
use crate::tui::theme::Theme;
use crossbeam::channel::{self, Receiver, Sender};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

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
    Launch {
        album_dir: PathBuf,
        embed: bool,
        artist: Option<String>,
        album: Option<String>,
        identifier: Option<String>,
        country: Option<String>,
        resolution: Option<String>,
        sources: Option<String>,
    },
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
    pub artwork_tx: Option<Sender<InspectJob>>,
    pub artwork_rx: Receiver<ArtworkMsg>,
}

impl App {
    /// Create a new App from config. Loads cached albums for instant startup if available.
    pub fn new(cfg: Config) -> anyhow::Result<Self> {
        let theme = Theme::load(&cfg.theme);
        let (scan_tx, scan_rx) = channel::unbounded();
        let (artwork_tx_chan, artwork_rx) = channel::unbounded();

        let scan_epoch = Arc::new(AtomicU64::new(0));

        // Spawn artwork inspector worker pool (4 workers)
        let artwork_job_tx =
            artwork::spawn_inspector_pool(scan_epoch.clone(), artwork_tx_chan, 4);

        let has_root = cfg
            .library_root
            .as_ref()
            .map(|p| p.exists())
            .unwrap_or(false);

        let mut albums = Vec::new();
        let mut filtered = Vec::new();
        let mut matcher = AlbumMatcher::new();
        let mut status_line = String::new();

        // Load disk cache for instant startup
        if let Some(ref root) = cfg.library_root
            && let Some(cached) = cache::load_cache(root)
        {
            albums = cached;
            matcher.replace_items(albums.clone());
            matcher.query("");
            filtered = matcher.results();
            status_line = format!("Loaded {} albums instantly from cache", albums.len());

            // Queue initial artwork inspection for cached items
            let epoch = scan_epoch.load(Ordering::Relaxed);
            for album in &albums {
                let _ = artwork_job_tx.send(InspectJob {
                    epoch,
                    dir: album.dir.clone(),
                    tracks: album.tracks.clone(),
                });
            }
        }

        let mut app = App {
            theme,
            cfg,
            albums,
            filtered,
            matcher,
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
            scan_epoch,
            status_line,
            is_scanning: false,
            scan_tx: Some(scan_tx),
            scan_rx,
            artwork_tx: Some(artwork_job_tx),
            artwork_rx,
        };

        // If no cache was loaded, start disk scan immediately
        if app.screen != Screen::FirstRun && app.albums.is_empty() {
            app.start_scan();
        }

        Ok(app)
    }

    /// Start (or restart) the recursive library scan.
    pub fn start_scan(&mut self) {
        let epoch = self.scan_epoch.fetch_add(1, Ordering::Relaxed) + 1;
        let cancel = self.scan_epoch.clone();
        let tx = self.scan_tx.clone().unwrap();

        self.statuses.clear();

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
                let arc_albums: Vec<Arc<Album>> = albums.iter().cloned().map(Arc::new).collect();

                // Dispatch artwork inspection jobs to worker pool
                if let Some(ref tx) = self.artwork_tx {
                    for album in &arc_albums {
                        let job = InspectJob {
                            epoch,
                            dir: album.dir.clone(),
                            tracks: album.tracks.clone(),
                        };
                        let _ = tx.send(job);
                    }
                }

                self.albums.extend(arc_albums);
                // Re-run query with current input
                self.matcher.replace_items(self.albums.clone());
                self.matcher.query(&self.input);
                self.filtered = self.matcher.results();
                if self.filter != Filter::All {
                    self.apply_filter();
                }
            }
            ScanMsg::Done { epoch, total } => {
                if epoch != self.scan_epoch.load(Ordering::Relaxed) {
                    return;
                }
                self.is_scanning = false;
                self.status_line = format!("{} albums indexed", total);

                // Save updated index to cache for future instant launches
                if let Some(ref root) = self.cfg.library_root {
                    let _ = cache::save_cache(root, &self.albums);
                }
            }
        }
    }

    /// Handle an artwork inspection message.
    pub fn handle_artwork(&mut self, msg: ArtworkMsg) {
        if msg.epoch != self.scan_epoch.load(Ordering::Relaxed) {
            return; // stale
        }
        self.statuses.insert(msg.dir, msg.status);

        // Re-apply active filters when status changes
        if self.filter != Filter::All {
            self.apply_filter();
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
            AppMsg::Artwork(am) => {
                self.handle_artwork(am);
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
                let selected_dir = self.filtered.get(self.selected).map(|a| a.dir.clone());
                let leaves_form = matches!(key.code, KeyCode::Enter | KeyCode::Esc);
                let action = Self::handle_form_key(&mut form, selected_dir, key);
                if !leaves_form {
                    self.screen = Screen::Form(form);
                }
                action
            }
            Screen::Help | Screen::Log | Screen::Doctor => match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                    self.screen = Screen::Finder;
                    Action::None
                }
                _ => Action::None,
            },
            Screen::FirstRun => match key.code {
                KeyCode::Enter => {
                    // Accept library root, save permanently to config file, clear search input, move to finder
                    let path = PathBuf::from(self.input.trim());
                    if path.exists() && path.is_dir() {
                        self.cfg.library_root = Some(path.clone());
                        let _ = self.cfg.save(); // Permanently write config file!
                        self.input.clear();
                        self.cursor = 0;
                        self.screen = Screen::Finder;
                        self.start_scan();
                    } else {
                        self.status_line = format!("Directory not found: {}", self.input.trim());
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
        // Handle Ctrl-shortcut characters vs navigation keys cleanly
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('p') => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    return Action::None;
                }
                KeyCode::Char('n') => {
                    if self.selected + 1 < self.filtered.len() {
                        self.selected += 1;
                    }
                    return Action::None;
                }
                KeyCode::Char('e') => {
                    if let Some(album) = self.filtered.get(self.selected) {
                        self.status_line =
                            format!("🚀 Launched COV (embed) for {}", album.display);
                        return Action::Launch {
                            album_dir: album.dir.clone(),
                            embed: true,
                            artist: None,
                            album: None,
                            identifier: None,
                            country: None,
                            resolution: None,
                            sources: None,
                        };
                    }
                    return Action::None;
                }
                KeyCode::Char('s') => {
                    // Swinsian current track shortcut
                    match macos::swinsian_track_path() {
                        Ok(p) => {
                            let path_buf = PathBuf::from(&p);
                            let parent = path_buf.parent().unwrap_or(&path_buf).to_path_buf();
                            self.status_line = format!("🚀 Launched COV (Swinsian) for {}", p);
                            return Action::Launch {
                                album_dir: parent,
                                embed: false,
                                artist: None,
                                album: None,
                                identifier: None,
                                country: None,
                                resolution: None,
                                sources: None,
                            };
                        }
                        Err(e) => {
                            self.status_line = format!("Swinsian error: {e}");
                            return Action::None;
                        }
                    }
                }
                KeyCode::Char('w') => {
                    // Finder current selection shortcut
                    match macos::finder_selection() {
                        Ok(p) => {
                            let path_buf = PathBuf::from(&p);
                            let parent = if path_buf.is_dir() {
                                path_buf
                            } else {
                                path_buf.parent().unwrap_or(&path_buf).to_path_buf()
                            };
                            self.status_line = format!("🚀 Launched COV (Finder) for {}", p);
                            return Action::Launch {
                                album_dir: parent,
                                embed: false,
                                artist: None,
                                album: None,
                                identifier: None,
                                country: None,
                                resolution: None,
                                sources: None,
                            };
                        }
                        Err(e) => {
                            self.status_line = format!("Finder error: {e}");
                            return Action::None;
                        }
                    }
                }
                KeyCode::Char('k') => {
                    // Clipboard path shortcut
                    match macos::pbpaste() {
                        Ok(clip) => {
                            let trimmed = clip.trim();
                            let expanded = crate::paths::expand_tilde(trimmed);
                            if expanded.exists() {
                                let parent = if expanded.is_dir() {
                                    expanded
                                } else {
                                    expanded.parent().unwrap_or(&expanded).to_path_buf()
                                };
                                self.status_line =
                                    format!("🚀 Launched COV (Clipboard) for {}", trimmed);
                                return Action::Launch {
                                    album_dir: parent,
                                    embed: false,
                                    artist: None,
                                    album: None,
                                    identifier: None,
                                    country: None,
                                    resolution: None,
                                    sources: None,
                                };
                            } else {
                                self.status_line =
                                    format!("Clipboard path not found: {}", trimmed);
                                return Action::None;
                            }
                        }
                        Err(e) => {
                            self.status_line = format!("Clipboard error: {e}");
                            return Action::None;
                        }
                    }
                }
                KeyCode::Char('g') => {
                    // Native folder chooser shortcut
                    match macos::choose_folder() {
                        Ok(p) => {
                            let parent = PathBuf::from(&p);
                            self.status_line = format!("🚀 Launched COV (Folder Picker) for {}", p);
                            return Action::Launch {
                                album_dir: parent,
                                embed: false,
                                artist: None,
                                album: None,
                                identifier: None,
                                country: None,
                                resolution: None,
                                sources: None,
                            };
                        }
                        Err(_) => {
                            return Action::None; // User cancelled picker
                        }
                    }
                }
                KeyCode::Char('f') => {
                    self.filter = self.filter.next();
                    self.apply_filter();
                    return Action::None;
                }
                KeyCode::Char('r') => {
                    self.albums.clear();
                    self.filtered.clear();
                    self.statuses.clear();
                    self.selected = 0;
                    return Action::Rescan;
                }
                KeyCode::Char('o') => {
                    self.screen = Screen::Form(FormState::default());
                    return Action::None;
                }
                KeyCode::Char('l') => {
                    self.screen = Screen::Log;
                    return Action::None;
                }
                KeyCode::Char('d') => {
                    self.screen = Screen::Doctor;
                    return Action::None;
                }
                KeyCode::Char('c') => {
                    return Action::Quit;
                }
                _ => {}
            }
        }

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
                self.selected = 0;
                Action::None
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Action::None
            }
            KeyCode::Down => {
                if self.selected + 1 < self.filtered.len() {
                    self.selected += 1;
                }
                Action::None
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(10);
                Action::None
            }
            KeyCode::PageDown => {
                if !self.filtered.is_empty() {
                    self.selected = (self.selected + 10).min(self.filtered.len() - 1);
                }
                Action::None
            }
            KeyCode::Home => {
                self.selected = 0;
                Action::None
            }
            KeyCode::End => {
                if !self.filtered.is_empty() {
                    self.selected = self.filtered.len() - 1;
                }
                Action::None
            }
            KeyCode::Enter => {
                // Save mode: open COV
                if let Some(album) = self.filtered.get(self.selected) {
                    self.status_line = format!("🚀 Launched COV for {}", album.display);
                    return Action::Launch {
                        album_dir: album.dir.clone(),
                        embed: false,
                        artist: None,
                        album: None,
                        identifier: None,
                        country: None,
                        resolution: None,
                        sources: None,
                    };
                }
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
                self.apply_filter();
                self.selected = 0;
                Action::None
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.cursor = self.input.len();
                self.matcher.query(&self.input);
                self.apply_filter();
                self.selected = 0;
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_form_key(
        form: &mut FormState,
        selected_album_dir: Option<PathBuf>,
        key: KeyEvent,
    ) -> Action {
        let fields = 7; // mode + 6 text fields
        match key.code {
            KeyCode::Tab | KeyCode::Down => {
                form.focus = (form.focus + 1) % fields;
                Action::None
            }
            KeyCode::BackTab | KeyCode::Up => {
                form.focus = if form.focus == 0 {
                    fields - 1
                } else {
                    form.focus - 1
                };
                Action::None
            }
            KeyCode::Enter => {
                if let Some(album_dir) = selected_album_dir {
                    let opt_str = |s: &str| {
                        let trimmed = s.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed.to_string())
                        }
                    };
                    Action::Launch {
                        album_dir,
                        embed: form.embed_mode,
                        artist: opt_str(&form.artist),
                        album: opt_str(&form.album),
                        identifier: opt_str(&form.identifier),
                        country: opt_str(&form.country),
                        resolution: opt_str(&form.resolution),
                        sources: opt_str(&form.sources),
                    }
                } else {
                    Action::None
                }
            }
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
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
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
            // Poll for events with 50ms timeout (allows scan/artwork messages to be processed)
            if event::poll(std::time::Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        let action = self.reduce(AppMsg::Key(key));
                        match action {
                            Action::Quit => break,
                            Action::Rescan => {
                                self.start_scan();
                            }
                            Action::Launch {
                                album_dir,
                                embed,
                                artist,
                                album,
                                identifier,
                                country,
                                resolution,
                                sources,
                            } => {
                                let path = album_dir.to_string_lossy().to_string();
                                let opts = LaunchOptions {
                                    path,
                                    embed,
                                    output: "cover".to_string(),
                                    covit: self.cfg.covit_path.clone(),
                                    log: self.cfg.log_path.clone(),
                                    artist,
                                    album,
                                    identifier,
                                    country,
                                    resolution,
                                    sources,
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
            while let Ok(msg) = self.scan_rx.try_recv() {
                self.reduce(AppMsg::Scan(msg));
            }

            // Check artwork channel
            while let Ok(msg) = self.artwork_rx.try_recv() {
                self.reduce(AppMsg::Artwork(msg));
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
            Screen::Finder => screens::finder::draw(self, f),
            Screen::Form(ref form) => screens::form::draw(self, f, form),
            Screen::Log => screens::log::draw(self, f),
            Screen::Doctor => screens::doctor::draw(self, f),
            Screen::Help => screens::help::draw(self, f),
            Screen::FirstRun => screens::firstrun::draw(self, f),
        }
    }
}

/// Messages that can be sent to the App reducer.
pub enum AppMsg {
    Key(KeyEvent),
    Scan(ScanMsg),
    Artwork(ArtworkMsg),
}

impl Filter {
    pub fn glyph(&self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Missing => "Missing",
            Filter::NeedsEmbed => "Needs Embed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_album(name: &str) -> Arc<Album> {
        Arc::new(Album {
            dir: PathBuf::from(format!("/music/{}", name)),
            rel: name.to_string(),
            display: name.to_string(),
            tracks: vec![PathBuf::from(format!("/music/{}/01.mp3", name))],
        })
    }

    #[test]
    fn test_firstrun_clears_input_on_enter() {
        let temp = tempfile::tempdir().unwrap();
        let mut cfg = Config::default();
        cfg.library_root = None;

        let mut app = App::new(cfg).unwrap();
        assert_eq!(app.screen, Screen::FirstRun);

        app.input = temp.path().to_string_lossy().to_string();
        let key = KeyEvent::from(KeyCode::Enter);
        app.reduce(AppMsg::Key(key));

        assert_eq!(app.screen, Screen::Finder);
        assert!(app.input.is_empty(), "input should be cleared on transition to Finder");
    }

    #[test]
    fn test_form_key_returns_launch_action() {
        let mut form = FormState {
            embed_mode: true,
            artist: " The Beatles ".to_string(),
            album: "Abbey Road".to_string(),
            ..Default::default()
        };
        let album_dir = PathBuf::from("/music/the_beatles/abbey_road");

        let key = KeyEvent::from(KeyCode::Enter);
        let action = App::handle_form_key(&mut form, Some(album_dir.clone()), key);

        if let Action::Launch {
            album_dir: dir,
            embed,
            artist,
            album,
            ..
        } = action
        {
            assert_eq!(dir, album_dir);
            assert!(embed);
            assert_eq!(artist, Some("The Beatles".to_string()));
            assert_eq!(album, Some("Abbey Road".to_string()));
        } else {
            panic!("Expected Action::Launch, got {:?}", action);
        }
    }

    #[test]
    fn test_plain_arrow_navigation() {
        let cfg = Config::default();
        let mut app = App::new(cfg).unwrap();
        app.filtered = vec![create_test_album("A1"), create_test_album("A2"), create_test_album("A3")];
        app.screen = Screen::Finder;

        assert_eq!(app.selected, 0);
        app.reduce(AppMsg::Key(KeyEvent::from(KeyCode::Down)));
        assert_eq!(app.selected, 1);
        app.reduce(AppMsg::Key(KeyEvent::from(KeyCode::Down)));
        assert_eq!(app.selected, 2);
        app.reduce(AppMsg::Key(KeyEvent::from(KeyCode::Up)));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_page_and_home_end_navigation() {
        let cfg = Config::default();
        let mut app = App::new(cfg).unwrap();
        app.filtered = (0..25).map(|i| create_test_album(&format!("A{}", i))).collect();
        app.screen = Screen::Finder;

        assert_eq!(app.selected, 0);
        app.reduce(AppMsg::Key(KeyEvent::from(KeyCode::PageDown)));
        assert_eq!(app.selected, 10);
        app.reduce(AppMsg::Key(KeyEvent::from(KeyCode::End)));
        assert_eq!(app.selected, 24);
        app.reduce(AppMsg::Key(KeyEvent::from(KeyCode::Home)));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_form_arrow_field_navigation() {
        let mut form = FormState::default();
        assert_eq!(form.focus, 0);

        App::handle_form_key(&mut form, None, KeyEvent::from(KeyCode::Down));
        assert_eq!(form.focus, 1);

        App::handle_form_key(&mut form, None, KeyEvent::from(KeyCode::Up));
        assert_eq!(form.focus, 0);
    }
}
