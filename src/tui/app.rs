use crate::config::CacheRefresh;
use crate::config::{Config, Mode};
use crate::launcher::{self, LaunchOptions};
use crate::tui::artwork::{self, ArtworkMsg, ArtworkStatus, Badge, Filter, InspectJob};
use crate::tui::cache;
use crate::tui::matcher::AlbumMatcher;
use crate::tui::scanner::{self, Album, ScanMsg};
use crate::tui::screens;
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

/// Screen states for the TUI state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    FirstRun,
    Finder,
    Settings(SettingsState),
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Library,
    DefaultAction,
    OutputName,
    Resolution,
    Providers,
}

impl SettingsField {
    fn next(self) -> Self {
        match self {
            Self::Library => Self::DefaultAction,
            Self::DefaultAction => Self::OutputName,
            Self::OutputName => Self::Resolution,
            Self::Resolution => Self::Providers,
            Self::Providers => Self::Library,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Library => Self::Providers,
            Self::DefaultAction => Self::Library,
            Self::OutputName => Self::DefaultAction,
            Self::Resolution => Self::OutputName,
            Self::Providers => Self::Resolution,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsState {
    pub library_root: String,
    pub default_mode: Mode,
    pub output_basename: String,
    pub resolution: String,
    pub sources: String,
    pub focus: SettingsField,
}

impl SettingsState {
    fn from_config(config: &Config) -> Self {
        Self {
            library_root: config
                .library_root
                .as_ref()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_default(),
            default_mode: config.default_mode,
            output_basename: config.output_basename.clone(),
            resolution: config
                .default_resolution
                .map(|value| value.to_string())
                .unwrap_or_default(),
            sources: config.default_sources.clone().unwrap_or_default(),
            focus: SettingsField::Library,
        }
    }
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
    pub replace_cached_index_on_batch: bool,
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

        let artwork_job_tx = artwork::spawn_inspector_pool(scan_epoch.clone(), artwork_tx_chan, 1);

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
        if let Some(root) = cfg.library_root.as_ref() {
            match cache::load_cache(root, &cfg.cache) {
                Ok(Some(cached)) => {
                    albums = cached;
                    matcher.replace_items(albums.clone());
                    matcher.query("");
                    filtered = matcher.results();
                    status_line = format!("Loaded {} albums from cache", albums.len());

                    let epoch = scan_epoch.load(Ordering::Relaxed);
                    for album in &albums {
                        let _ = artwork_job_tx.send(InspectJob {
                            epoch,
                            dir: album.dir.clone(),
                            tracks: album.tracks.clone(),
                        });
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    status_line = format!("Ignoring unreadable cache: {error}");
                }
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
            replace_cached_index_on_batch: false,
            scan_tx: Some(scan_tx),
            scan_rx,
            artwork_tx: Some(artwork_job_tx),
            artwork_rx,
        };

        // If no cache was loaded, start disk scan immediately
        if app.screen != Screen::FirstRun && app.albums.is_empty() {
            app.start_scan();
        } else if app.cfg.cache.refresh == CacheRefresh::Startup {
            app.start_cache_refresh();
        }

        Ok(app)
    }

    /// Start (or restart) the recursive library scan.
    pub fn start_scan(&mut self) {
        self.clear_index();
        self.replace_cached_index_on_batch = false;
        self.begin_scan();
    }

    fn start_cache_refresh(&mut self) {
        self.replace_cached_index_on_batch = true;
        self.begin_scan();
    }

    fn begin_scan(&mut self) {
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

    fn clear_index(&mut self) {
        self.albums.clear();
        self.filtered.clear();
        self.matcher.replace_items(Vec::new());
    }

    /// Handle a scan message.
    pub fn handle_scan(&mut self, msg: ScanMsg) {
        match msg {
            ScanMsg::Batch { ref albums, epoch } => {
                if epoch != self.scan_epoch.load(Ordering::Relaxed) {
                    return; // stale
                }
                if self.replace_cached_index_on_batch {
                    self.clear_index();
                    self.replace_cached_index_on_batch = false;
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
                if self.replace_cached_index_on_batch {
                    self.clear_index();
                    self.replace_cached_index_on_batch = false;
                }
                self.is_scanning = false;
                self.status_line = format!("{} albums indexed", total);

                // Save updated index to cache for future instant launches
                if let Some(ref root) = self.cfg.library_root {
                    let _ = cache::save_cache(root, &self.albums, &self.cfg.cache);
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
            Screen::Settings(_) => {
                let mut settings = match std::mem::replace(&mut self.screen, Screen::Finder) {
                    Screen::Settings(settings) => settings,
                    _ => unreachable!(),
                };
                let (action, close) = self.handle_settings_key(&mut settings, key);
                if !close {
                    self.screen = Screen::Settings(settings);
                }
                action
            }
            Screen::Help => match key.code {
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
                        self.status_line = format!("🚀 Launched COV (embed) for {}", album.display);
                        return Action::Launch {
                            album_dir: album.dir.clone(),
                            embed: true,
                        };
                    }
                    return Action::None;
                }
                KeyCode::Char('s') => {
                    if let Some(album) = self.filtered.get(self.selected) {
                        self.status_line = format!("Launched COV for {}", album.display);
                        return Action::Launch {
                            album_dir: album.dir.clone(),
                            embed: false,
                        };
                    }
                    return Action::None;
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
                    self.screen = Screen::Settings(SettingsState::from_config(&self.cfg));
                    return Action::None;
                }
                KeyCode::Char('c') => {
                    return Action::Quit;
                }
                _ => return Action::None,
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
                        embed: self.cfg.default_mode == Mode::Embed,
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

    fn handle_settings_key(
        &mut self,
        settings: &mut SettingsState,
        key: KeyEvent,
    ) -> (Action, bool) {
        match key.code {
            KeyCode::Esc => (Action::None, true),
            KeyCode::Enter => (Action::None, self.save_settings(settings)),
            KeyCode::Tab | KeyCode::Down => {
                settings.focus = settings.focus.next();
                (Action::None, false)
            }
            KeyCode::BackTab | KeyCode::Up => {
                settings.focus = settings.focus.previous();
                (Action::None, false)
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Char(' ')
                if settings.focus == SettingsField::DefaultAction =>
            {
                settings.default_mode = match settings.default_mode {
                    Mode::Save => Mode::Embed,
                    Mode::Embed => Mode::Save,
                };
                (Action::None, false)
            }
            KeyCode::Backspace => {
                match settings.focus {
                    SettingsField::Library => {
                        settings.library_root.pop();
                    }
                    SettingsField::OutputName => {
                        settings.output_basename.pop();
                    }
                    SettingsField::Resolution => {
                        settings.resolution.pop();
                    }
                    SettingsField::Providers => {
                        settings.sources.pop();
                    }
                    SettingsField::DefaultAction => {}
                }
                (Action::None, false)
            }
            KeyCode::Char(character) => {
                match settings.focus {
                    SettingsField::Library => settings.library_root.push(character),
                    SettingsField::OutputName => settings.output_basename.push(character),
                    SettingsField::Resolution => settings.resolution.push(character),
                    SettingsField::Providers => settings.sources.push(character),
                    SettingsField::DefaultAction => {}
                };
                (Action::None, false)
            }
            _ => (Action::None, false),
        }
    }

    fn save_settings(&mut self, settings: &SettingsState) -> bool {
        let library_root = PathBuf::from(settings.library_root.trim());
        if !library_root.is_dir() {
            self.status_line = "Library folder does not exist".to_string();
            return false;
        }
        if settings.output_basename.trim().is_empty()
            || settings.output_basename.contains('/')
            || settings.output_basename.contains('\\')
        {
            self.status_line = "Cover name must be a filename, not a path".to_string();
            return false;
        }
        let resolution = match settings.resolution.trim() {
            "" => None,
            value => match value.parse::<u32>() {
                Ok(value) if value > 0 => Some(value),
                _ => {
                    self.status_line = "Resolution must be a positive number".to_string();
                    return false;
                }
            },
        };
        let sources = match settings.sources.trim() {
            "" => None,
            value if value.split(',').all(|source| !source.trim().is_empty()) => {
                Some(value.to_string())
            }
            _ => {
                self.status_line = "Providers must be comma-separated source IDs".to_string();
                return false;
            }
        };
        let library_changed = self.cfg.library_root.as_ref() != Some(&library_root);
        self.cfg.library_root = Some(library_root);
        self.cfg.default_mode = settings.default_mode;
        self.cfg.output_basename = settings.output_basename.trim().to_string();
        self.cfg.default_resolution = resolution;
        self.cfg.default_sources = sources;
        if let Err(error) = self.cfg.save() {
            self.status_line = format!("Could not save settings: {error}");
            return false;
        }
        if library_changed {
            self.start_scan();
        } else {
            self.status_line = "Settings saved".to_string();
        }
        true
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

    fn launch_cov(&mut self, opts: LaunchOptions) {
        if let Err(error) = launcher::launch(&opts) {
            self.status_line = format!("COV launch failed: {error:#}");
        }
    }

    /// Run the main TUI event loop.
    pub fn run(&mut self, debug: bool) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stderr = std::io::stderr();
        if let Err(error) = execute!(stderr, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }

        let backend = CrosstermBackend::new(stderr);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => {
                let _ = disable_raw_mode();
                let _ = execute!(std::io::stderr(), LeaveAlternateScreen);
                return Err(error.into());
            }
        };
        // Optional debug log
        if debug {
            // Could set up tracing here
        }

        // Draw loop
        let res = self.event_loop(&mut terminal);

        // Restore terminal
        let cleanup = (|| -> anyhow::Result<()> {
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            Ok(())
        })();

        match res {
            Ok(()) => cleanup,
            Err(error) => {
                cleanup?;
                Err(error)
            }
        }
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
                            Action::Launch { album_dir, embed } => {
                                let path = album_dir.to_string_lossy().to_string();
                                let opts = LaunchOptions {
                                    path,
                                    embed,
                                    output: self.cfg.output_basename.clone(),
                                    covit: self.cfg.covit_path.clone(),
                                    log: self.cfg.log_path.clone(),
                                    artist: None,
                                    album: None,
                                    identifier: None,
                                    country: None,
                                    resolution: self
                                        .cfg
                                        .default_resolution
                                        .map(|value| value.to_string()),
                                    sources: self.cfg.default_sources.clone(),
                                    foreground: false,
                                };
                                self.launch_cov(opts);
                            }
                            Action::None => {}
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }

            for _ in 0..8 {
                let Ok(msg) = self.scan_rx.try_recv() else {
                    break;
                };
                self.reduce(AppMsg::Scan(msg));
            }

            for _ in 0..32 {
                let Ok(msg) = self.artwork_rx.try_recv() else {
                    break;
                };
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
            Screen::Settings(ref settings) => screens::settings::draw(self, f, settings),
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
            Filter::Missing => "Needs Cover",
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
        let config_path = temp.path().join("config.toml");
        let cfg = Config::load_with_override(Some(&config_path)).unwrap();

        let mut app = App::new(cfg).unwrap();
        assert_eq!(app.screen, Screen::FirstRun);

        app.input = temp.path().to_string_lossy().to_string();
        let key = KeyEvent::from(KeyCode::Enter);
        app.reduce(AppMsg::Key(key));

        assert_eq!(app.screen, Screen::Finder);
        assert!(
            app.input.is_empty(),
            "input should be cleared on transition to Finder"
        );
    }

    #[test]
    fn test_failed_covit_launch_updates_the_status_line() {
        let temp = tempfile::tempdir().unwrap();
        let mut cfg = Config::default();
        cfg.covit_path = temp.path().join("missing-covit");
        let mut app = App::new(cfg.clone()).unwrap();

        app.launch_cov(LaunchOptions {
            path: temp.path().to_string_lossy().to_string(),
            embed: false,
            output: "cover".to_string(),
            covit: cfg.covit_path,
            log: cfg.log_path,
            artist: None,
            album: None,
            identifier: None,
            country: None,
            resolution: None,
            sources: None,
            foreground: false,
        });

        assert!(app.status_line.starts_with("COV launch failed:"));
    }

    #[test]
    fn test_plain_arrow_navigation() {
        let cfg = Config::default();
        let mut app = App::new(cfg).unwrap();
        app.filtered = vec![
            create_test_album("A1"),
            create_test_album("A2"),
            create_test_album("A3"),
        ];
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
        app.filtered = (0..25)
            .map(|i| create_test_album(&format!("A{}", i)))
            .collect();
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
    fn finder_ignores_removed_context_shortcuts() {
        let cfg = Config::default();
        let mut app = App::new(cfg).unwrap();
        app.screen = Screen::Finder;
        app.status_line = "Ready".to_string();

        app.reduce(AppMsg::Key(KeyEvent::new(
            KeyCode::Char('k'),
            KeyModifiers::CONTROL,
        )));

        assert_eq!(app.screen, Screen::Finder);
        assert_eq!(app.status_line, "Ready");
    }

    #[test]
    fn finder_launches_embed_for_selected_album() {
        let cfg = Config::default();
        let mut app = App::new(cfg).unwrap();
        let album = create_test_album("A1");
        app.filtered = vec![album.clone()];
        app.screen = Screen::Finder;

        let action = app.reduce(AppMsg::Key(KeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::CONTROL,
        )));

        match action {
            Action::Launch {
                album_dir, embed, ..
            } => {
                assert_eq!(album_dir, album.dir);
                assert!(embed);
            }
            other => panic!("expected an embed launch, got {other:?}"),
        }
    }

    #[test]
    fn finder_opens_settings_with_control_o() {
        let cfg = Config::default();
        let mut app = App::new(cfg).unwrap();
        app.screen = Screen::Finder;

        app.reduce(AppMsg::Key(KeyEvent::new(
            KeyCode::Char('o'),
            KeyModifiers::CONTROL,
        )));

        assert!(matches!(app.screen, Screen::Settings(_)));
    }

    #[test]
    fn finder_uses_embed_as_the_configured_default_action() {
        let mut cfg = Config::default();
        cfg.default_mode = crate::config::Mode::Embed;
        let mut app = App::new(cfg).unwrap();
        app.filtered = vec![create_test_album("A1")];
        app.screen = Screen::Finder;

        let action = app.reduce(AppMsg::Key(KeyEvent::from(KeyCode::Enter)));

        match action {
            Action::Launch { embed, .. } => assert!(embed),
            other => panic!("expected a launch, got {other:?}"),
        }
    }
}
