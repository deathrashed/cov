use clap::Parser;
use cov::cli::{self, Cli, Command, EmbedArgs, LogMode, Mode, OpenArgs, ScanArgs, ScanCommand};
use cov::config::Config;
use cov::doctor;
use cov::embed;
use cov::launcher::{self, LaunchOptions};
use cov::macos;
use cov::paths;
use cov::scan;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let cli = Cli::parse();

    let config_path = cli.config.as_deref();
    let cfg = match Config::load_with_override(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("cov: failed to load config: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = dispatch(&cli, &cfg) {
        eprintln!("cov {}: {e:#}", subcommand_name(&cli.command));
        std::process::exit(1);
    }
}

fn subcommand_name(cmd: &Command) -> &'static str {
    match cmd {
        Command::Open(_) => "open",
        Command::Embed(_) => "embed",
        Command::Context { .. } => "context",
        Command::Swinsian { .. } => "swinsian",
        Command::Finder { .. } => "finder",
        Command::Choose { .. } => "choose",
        Command::Clipboard { .. } => "clipboard",
        Command::Log { .. } => "log",
        Command::Doctor => "doctor",
        Command::Scan(_) => "scan",
        Command::Tui(_) => "tui",
        Command::Ghostty => "ghostty",
    }
}

fn dispatch(cli: &Cli, cfg: &Config) -> anyhow::Result<()> {
    match &cli.command {
        Command::Open(args) => cmd_open(args, cfg),
        Command::Embed(args) => cmd_embed(args, cfg),
        Command::Context { mode } => cmd_context(*mode, cfg),
        Command::Swinsian { mode } => {
            let path = macos::swinsian_track_path()?;
            run_open_for_path(&path, *mode, cfg)
        }
        Command::Finder { mode } => {
            let path = macos::finder_selection()?;
            run_open_for_path(&path, *mode, cfg)
        }
        Command::Choose { mode } => {
            let path = macos::choose_folder()?;
            run_open_for_path(&path, *mode, cfg)
        }
        Command::Clipboard { mode } => {
            let clip = macos::pbpaste()?;
            let trimmed = clip.trim();
            if trimmed.is_empty() {
                anyhow::bail!("Clipboard is empty.");
            }
            let expanded = cov::paths::expand_tilde(trimmed);
            if !expanded.exists() {
                anyhow::bail!("Clipboard path does not exist: {expanded:?}");
            }
            run_open_for_path(&expanded.to_string_lossy(), *mode, cfg)
        }
        Command::Log { mode } => cmd_log(*mode, cfg),
        Command::Doctor => cmd_doctor(cfg),
        Command::Scan(args) => cmd_scan(args, cfg),
        Command::Tui(args) => cmd_tui(args, cfg, cli.debug),
        Command::Ghostty => cmd_ghostty(),
    }
}

fn cmd_scan(args: &ScanArgs, cfg: &Config) -> anyhow::Result<()> {
    let configured_root = match &args.command {
        ScanCommand::MissingSidecar { root } | ScanCommand::MissingEmbedded { root } => {
            root.as_ref().or(cfg.library_root.as_ref())
        }
    };
    let root = configured_root
        .map(|path| cov::paths::expand_tilde(&path.to_string_lossy()))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No library root configured. Pass ROOT or set library_root in the config."
            )
        })?;
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    let mut print_path = |path: &std::path::Path| -> anyhow::Result<()> {
        writeln!(output, "{}", path.display()).map_err(anyhow::Error::from)
    };

    let result = match &args.command {
        ScanCommand::MissingSidecar { .. } => scan::missing_sidecar(&root, &mut print_path),
        ScanCommand::MissingEmbedded { .. } => scan::missing_embedded(&root, &mut print_path),
    };

    match result {
        Err(error)
            if error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|io_error| io_error.kind() == std::io::ErrorKind::BrokenPipe) =>
        {
            Ok(())
        }
        other => other,
    }
}

fn cmd_context(mode: Option<Mode>, cfg: &Config) -> anyhow::Result<()> {
    let (_source, path) = cov::context::detect()?;
    run_open_for_path(&path.to_string_lossy(), mode, cfg)
}

fn cmd_open(args: &OpenArgs, cfg: &Config) -> anyhow::Result<()> {
    let opts = LaunchOptions {
        path: args.path.clone(),
        embed: args.embed,
        output: args
            .output
            .clone()
            .unwrap_or_else(|| cfg.output_basename.clone()),
        covit: args.covit.clone().unwrap_or_else(|| cfg.covit_path.clone()),
        log: args.log.clone().unwrap_or_else(|| cfg.log_path.clone()),
        artist: args.artist.clone(),
        album: args.album.clone(),
        identifier: args.identifier.clone(),
        country: args.country.clone(),
        resolution: args
            .resolution
            .clone()
            .or_else(|| cfg.default_resolution.map(|value| value.to_string())),
        sources: args.sources.clone().or_else(|| cfg.default_sources.clone()),
        foreground: args.foreground,
    };
    let (_audio_path, argv) = launcher::launch(&opts)?;
    println!("COV opened with {} arguments", argv.len());
    Ok(())
}

fn cmd_embed(args: &EmbedArgs, _cfg: &Config) -> anyhow::Result<()> {
    if !args.artwork.exists() {
        anyhow::bail!("Artwork not found: {}", args.artwork.display());
    }
    let data = std::fs::read(&args.artwork)?;
    let mime = guess_mime(&args.artwork)?;

    let targets = paths::target_files(&args.target);
    if targets.is_empty() {
        let dir = cov::paths::expand_tilde(&args.target);
        if dir.is_dir() {
            anyhow::bail!("No supported audio files found: {}", args.target);
        }
        anyhow::bail!("No supported audio files found: {}", args.target);
    }

    let mut updated = 0usize;
    let mut failed = Vec::new();
    for target in &targets {
        if args.dry_run {
            println!(
                "WOULD EMBED {} <- {}",
                target.display(),
                args.artwork.display()
            );
            updated += 1;
            continue;
        }
        match embed::embed_file(target, &data, mime) {
            Ok(()) => {
                println!("EMBEDDED {}", target.display());
                updated += 1;
            }
            Err(e) => {
                eprintln!("FAILED {}: {e}", target.display());
                failed.push((target.clone(), e.to_string()));
            }
        }
    }

    let total = targets.len();
    println!(
        "\nSUMMARY: {updated} updated, {} failed, {total} total",
        failed.len()
    );

    if args.rescan_swinsian && failed.is_empty() && updated > 0 {
        macos::rescan_swinsian(&targets).ok();
    }

    if !failed.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

fn guess_mime(path: &std::path::Path) -> anyhow::Result<&'static str> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => Ok("image/jpeg"),
        Some("png") => Ok("image/png"),
        _ => anyhow::bail!("Unsupported artwork format: {}", path.display()),
    }
}

fn run_open_for_path(path_str: &str, mode: Option<Mode>, cfg: &Config) -> anyhow::Result<()> {
    let embed_mode = mode.map(|m| matches!(m, Mode::Embed)).unwrap_or(false);
    let opts = LaunchOptions {
        path: path_str.to_string(),
        embed: embed_mode,
        output: "cover".to_string(),
        covit: cfg.covit_path.clone(),
        log: cfg.log_path.clone(),
        artist: None,
        album: None,
        identifier: None,
        country: None,
        resolution: None,
        sources: None,
        foreground: false,
    };
    let (_audio_path, argv) = launcher::launch(&opts)?;
    println!("COV opened with {} arguments", argv.len());
    Ok(())
}

fn cmd_log(mode: Option<LogMode>, cfg: &Config) -> anyhow::Result<()> {
    let log_path = &cfg.log_path;
    if !log_path.exists() {
        println!("No COV log exists yet at {}", log_path.display());
        return Ok(());
    }

    match mode.unwrap_or(LogMode::Show) {
        LogMode::Show => {
            let content = std::fs::read_to_string(log_path)?;
            print!("{content}");
        }
        LogMode::Follow => {
            let status = std::process::Command::new("/usr/bin/tail")
                .arg("-f")
                .arg(log_path)
                .status()?;
            if !status.success() {
                anyhow::bail!("tail -f exited with status {status}");
            }
        }
    }
    Ok(())
}

fn cmd_doctor(cfg: &Config) -> anyhow::Result<()> {
    let checks = doctor::run(cfg);
    doctor::print_report(&checks);
    Ok(())
}

fn cmd_tui(args: &cli::TuiArgs, cfg: &Config, debug: bool) -> anyhow::Result<()> {
    let mut app_cfg = cfg.clone();
    if let Some(ref lib) = args.library {
        app_cfg.library_root = Some(lib.clone());
    }
    if args.no_cache {
        app_cfg.cache.enabled = false;
    }
    if let Some(path) = &args.cache_path {
        app_cfg.cache.path = Some(path.clone());
    }
    if args.rescan {
        app_cfg.cache.refresh = cov::config::CacheRefresh::Startup;
    }
    cov::tui::run(app_cfg, debug)
}

fn cmd_ghostty() -> anyhow::Result<()> {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("cov"));
    let status = std::process::Command::new("/usr/bin/open")
        .arg("-na")
        .arg("Ghostty.app")
        .arg("--args")
        .arg("-e")
        .arg(&exe)
        .arg("tui")
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to open Ghostty");
    }
    Ok(())
}
