use std::path::PathBuf;

#[derive(clap::Parser)]
#[command(
    name = "cov",
    version,
    about = "COV integration toolkit — https://covers.musichoarders.xyz/"
)]
pub struct Cli {
    /// Path to config file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Write TUI debug log (appended) to ~/Library/Logs/cov-tui.log
    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Open COV for an audio file or album folder
    Open(OpenArgs),
    /// Embed existing local artwork without opening the browser
    Embed(EmbedArgs),
    /// Auto-detect source from Swinsian, Finder, or clipboard
    Context {
        #[arg(value_enum)]
        mode: Option<Mode>,
    },
    /// Use selected/playing Swinsian track
    Swinsian {
        #[arg(value_enum)]
        mode: Option<Mode>,
    },
    /// Use Finder selection
    Finder {
        #[arg(value_enum)]
        mode: Option<Mode>,
    },
    /// Prompt with a native folder chooser
    Choose {
        #[arg(value_enum)]
        mode: Option<Mode>,
    },
    /// Use a path copied to the clipboard
    Clipboard {
        #[arg(value_enum)]
        mode: Option<Mode>,
    },
    /// View or tail the COVIT log
    Log {
        #[arg(value_enum)]
        mode: Option<LogMode>,
    },
    /// Check environment
    Doctor,
    /// Scan a music library for missing artwork
    Scan(ScanArgs),
    /// Launch the interactive TUI
    Tui(TuiArgs),
    /// Open the TUI in a dedicated Ghostty window
    Ghostty,
}

#[derive(clap::Args)]
pub struct ScanArgs {
    #[command(subcommand)]
    pub command: ScanCommand,
}

#[derive(clap::Subcommand)]
pub enum ScanCommand {
    /// Print album directories that contain audio but no cover sidecar
    MissingSidecar {
        /// Music library root (defaults to configured library_root)
        root: Option<PathBuf>,
    },
    /// Print audio files without embedded front or other cover artwork
    MissingEmbedded {
        /// Music library root (defaults to configured library_root)
        root: Option<PathBuf>,
    },
}

#[derive(clap::Args)]
pub struct OpenArgs {
    /// Path to audio file or album directory
    pub path: String,
    /// Save cover and embed into all album tracks
    #[arg(long)]
    pub embed: bool,
    /// Output basename for cover file (default: cover)
    pub output: Option<String>,
    /// Path to covit binary
    #[arg(long)]
    pub covit: Option<PathBuf>,
    /// Path to log file
    #[arg(long)]
    pub log: Option<PathBuf>,
    /// Query filter: artist
    #[arg(long)]
    pub artist: Option<String>,
    /// Query filter: album
    #[arg(long)]
    pub album: Option<String>,
    /// Query filter: identifier (e.g. release group MBID)
    #[arg(long)]
    pub identifier: Option<String>,
    /// Query filter: country
    #[arg(long)]
    pub country: Option<String>,
    /// Query filter: resolution
    #[arg(long)]
    pub resolution: Option<String>,
    /// Query filter: sources (comma-separated)
    #[arg(long)]
    pub sources: Option<String>,
    /// Run COVIT in foreground (for debugging)
    #[arg(long)]
    pub foreground: bool,
}

#[derive(clap::Args)]
pub struct EmbedArgs {
    /// Path to artwork image file (JPEG or PNG)
    pub artwork: PathBuf,
    /// Target audio file or album directory
    pub target: String,
    /// Dry run: show what would be embedded without writing
    #[arg(long)]
    pub dry_run: bool,
    /// Rescan Swinsian after embedding
    #[arg(long)]
    pub rescan_swinsian: bool,
}

#[derive(clap::Args)]
pub struct TuiArgs {
    /// Music library root (overrides config)
    #[arg(long)]
    pub library: Option<PathBuf>,
    /// Do not load or write the library index cache
    #[arg(long)]
    pub no_cache: bool,
    /// Path for the library index cache file
    #[arg(long)]
    pub cache_path: Option<PathBuf>,
    /// Refresh the index in the background after loading a cache
    #[arg(long)]
    pub rescan: bool,
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Mode {
    Save,
    Embed,
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum LogMode {
    Show,
    Follow,
}
