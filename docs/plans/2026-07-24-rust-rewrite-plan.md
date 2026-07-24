# COV Toolkit Rust Rewrite — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the Python/zsh COV toolkit with a single Rust binary `cov` offering subcommand parity plus a television-inspired TUI (fuzzy album finder, whole-set artwork states, Kitty/halfblock cover preview).

**Architecture:** One binary crate at repo root. Pure, golden-tested constructors for the COVIT launch contract; `lofty`-based embedding; TUI with scanner/matcher/artwork workers on std threads + crossbeam, epoch-tagged results; config via `directories`. Full spec: `docs/plans/2026-07-24-rust-rewrite-design.md`.

**Tech Stack:** Rust (edition 2024, toolchain 1.97), clap 4, lofty, ratatui 0.30 + crossterm 0.29, ratatui-image 11, nucleo/frizbee (bench decides), walkdir, crossbeam, serde/toml, directories, image, criterion, assert_cmd.

**Environment facts (verified):** cargo 1.97.1 (Homebrew), ffmpeg/flac/lame/afconvert present, repo on `main` at `f4bc8df`, legacy files live until Phase 5 cutover (old toolkit stays functional during the build).

**Commit policy:** each task ends with a commit on branch `rust-rewrite`. If the user has not authorized commits, skip commit steps and report instead.

---

## Phase 0 — Scaffold

### Task 0.1: Branch + Cargo scaffold

**Files:**
- Create: `Cargo.toml`, `src/main.rs`, `.gitignore` (modify)

**Step 1: Create branch**

```bash
git -C /Users/rd/Projects/cov checkout -b rust-rewrite
```

**Step 2: Write Cargo.toml**

```toml
[package]
name = "cov"
version = "2.0.0"
edition = "2024"
description = "macOS integration toolkit for COV cover search (https://covers.musichoarders.xyz/)"
license = "MIT"

[dependencies]
anyhow = "1"
thiserror = "2"
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.9"
directories = "6"
walkdir = "2"
crossbeam = "0.8"
lofty = "0.22"
ratatui = { version = "0.30", features = ["serde"] }
crossterm = { version = "0.28", features = ["serde"] }
ratatui-image = "8"
image = "0.25"
nucleo = "0.5"
frizbee = "0.11"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
criterion = { version = "0.8", features = ["html_reports"] }

[[bench]]
name = "matcher"
harness = false
```

**Step 3: Minimal main.rs**

```rust
fn main() {
    println!("cov: not implemented yet");
}
```

**Step 4: `.gitignore` — append `/target`**

**Step 5: Verify**

Run: `cargo build 2>&1 | tail -3`
Expected: `Finished` (resolve any version pins that fail; bump to nearest compatible)

**Step 6: Commit** `chore: scaffold Rust crate for cov rewrite`

---

## Phase 1 — Spikes (decisions D4, MP3 v2.3)

### Task 1.1: Fixture generation script

**Files:**
- Create: `tests/fixtures/generate.sh`, `tests/fixtures/.gitignore` note (fixtures are committed, script is for regeneration)

**Step 1: Write the script** (uses system AIFF as source audio; transcodes to every format; ~2s sine is fine too — ffmpeg `sine` source keeps fixtures tiny and copyright-free)

```zsh
#!/bin/zsh
set -eu
cd ${0:A:h}
SINE=(-f lavfi -i "sine=frequency=440:duration=1")
ffmpeg -y -loglevel error $SINE -c:a aac fixture.m4a
ffmpeg -y -loglevel error $SINE -c:a libmp3lame -b:a 64k fixture.mp3
ffmpeg -y -loglevel error $SINE -c:a flac fixture.flac
ffmpeg -y -loglevel error $SINE -c:a libvorbis fixture.ogg
ffmpeg -y -loglevel error $SINE -c:a libopus fixture.opus
ffmpeg -y -loglevel error $SINE -c:a pcm_s16le fixture.wav
ffmpeg -y -loglevel error $SINE -c:a pcm_s16be -f aiff fixture.aiff
ffmpeg -y -loglevel error $SINE -c:a wavpack fixture.wv
ffmpeg -y -loglevel error $SINE -c:a ape fixture.ape
ffmpeg -y -loglevel error $SINE -c:a dsd_lsbf_planar -ar 2822400 fixture.dsf || print "SKIP: dsf encoder unavailable"
# 256x256 test artwork
ffmpeg -y -loglevel error -f lavfi -i "color=c=0x01ACD7:size=256x256:duration=0.1" -frames:v 1 artwork.jpg
ffmpeg -y -loglevel error -f lavfi -i "color=c=0x1E1E1E:size=256x256:duration=0.1" -frames:v 1 artwork.png
```

**Step 2: Run it; verify all fixtures exist**

Run: `zsh tests/fixtures/generate.sh && ls tests/fixtures/`
Expected: all 10 audio fixtures + 2 artwork files (dsf optional)

**Step 3: Commit** `test: add audio fixture generation`

### Task 1.2: Tagging spike → `tests/embed_matrix.rs` skeleton

**Files:**
- Test: `tests/embed_matrix.rs`

**Step 1: Write the failing matrix test** (drive via the *planned* embed API; it will not compile until Task 2.4 — that is the spike: iterate on lofty calls here until every format round-trips)

```rust
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures") }

#[rstest_formats]
fn embed_roundtrip_preserves_cover_and_tags(ext: &str) {
    let src = fixtures().join(format!("fixture.{ext}"));
    if !src.exists() { eprintln!("skipping {ext}"); return; }
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join(format!("track.{ext}"));
    std::fs::copy(&src, &target).unwrap();
    // seed a non-artwork tag that must survive
    cov::testutil::seed_title(&target, "Keep Me").unwrap();

    cov::embed::embed_file(&target, &std::fs::read(fixtures().join("artwork.jpg")).unwrap(), "image/jpeg").unwrap();

    let (covers, title) = cov::testutil::read_cover_count_and_title(&target).unwrap();
    assert_eq!(covers, 1, "{ext}: expected exactly one front cover");
    assert_eq!(title.as_deref(), Some("Keep Me"), "{ext}: unrelated tag lost");
}
```

(Implement as a plain `for ext in [...]` loop inside one `#[test]` if rstest is not added; keep dev-deps as listed — use a loop.)

**Step 2: Spike in the open** — create `src/lib.rs` with `pub mod embed;` and `pub mod testutil;`; iterate `embed_file` until the matrix passes per format. Record per-format findings in `docs/plans/spike-findings.md` (temporary; deleted in Phase 6 after promotion).

**Key questions the spike MUST answer:**
1. Does lofty write ID3v2.3 for MP3? If not, MP3 uses the `id3` crate (`tag.write_to_path(path, id3::Version::Id3v23)`), added as a dependency.
2. lofty picture API per format (`push_picture` / `remove_picture_type(PictureType::CoverFront)`), and behavior when no tag exists (`insert_tag` with the file's primary tag type).
3. APEv2: confirm lofty maps pictures to `Cover Art (Front)` binary item (mutagen compat).
4. WAV/AIFF/DSF: confirm ID3 chunk round-trip.
5. MP4: confirm `covr` with JPEG + PNG.

**Step 3: Run matrix**

Run: `cargo test --test embed_matrix -- --nocapture`
Expected: PASS for all present fixtures

**Step 4: Commit** `feat(embed): lofty embedding engine spike — full format matrix green`

### Task 1.3: Matcher benchmark → decide D4

**Files:**
- Create: `benches/matcher.rs`

**Step 1: Write the bench** — synthetic 5,000 album names (`Artist NN — Album Title NN`), needle `"fleet mac"`; measure full re-filter per keypress:

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn corpus() -> Vec<String> { /* generate 5_000 album display strings */ vec![] }

fn bench_matchers(c: &mut Criterion) {
    let items = corpus();
    c.bench_function("frizbee_refilter", |b| b.iter(|| {
        frizbee::match_list("fleet mac", items.iter().map(String::as_str), Some(0))
    }));
    c.bench_function("nucleo_refilter", |b| b.iter(|| {
        let mut n = nucleo::Nucleo::new(nucleo::Config::DEFAULT, || {}, None, 1);
        let inj = n.injector();
        for (i, s) in items.iter().enumerate() { inj.push(i, |_, cols| cols[0] = s.clone().into()); }
        n.tick(0);
        n.pattern.reparse(0, "fleet mac", nucleo::pattern::CaseMatching::Smart, nucleo::pattern::Normalization::Smart, false);
        while n.tick(10).running() {}
        n.snapshot().matched_item_count()
    }));
}

criterion_group!(benches, bench_matchers);
criterion_main!(benches);
```

**Step 2: Run** `cargo bench --bench matcher`

**Step 3: Decision rule** — if frizbee full-refilter is < 16ms (one frame) for 5k items, **choose frizbee** (simpler: recompute per keypress, no incremental worker state); else nucleo. Record choice + numbers in the commit message; update design doc D4.

**Step 4: Commit** `bench: matcher comparison — chose <winner> (<numbers>)`

---

## Phase 2 — Core modules

`src/lib.rs` exports: `pub mod paths; pub mod config; pub mod launcher; pub mod embed; pub mod macos; pub mod context; pub mod doctor;`

### Task 2.1: `paths.rs`

**Files:** Create `src/paths.rs`

**Step 1: Failing tests** (in-module, tempfile fixture tree)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn dir_returns_first_audio_sorted() { /* create b.flac, a.mp3, ._a.mp3, c.txt → expect a.mp3 */ }
    #[test] fn dir_without_audio_errors() { /* expect "No supported audio files found directly inside" */ }
    #[test] fn file_with_bad_ext_errors() { /* .txt → "Unsupported audio file" */ }
    #[test] fn missing_path_errors() { /* → "Path does not exist" */ }
    #[test] fn tak_launches_but_does_not_embed() { assert!(LAUNCH_EXTS.contains(".tak")); assert!(!EMBED_EXTS.contains(".tak")); }
    #[test] fn target_files_returns_all_sorted() { /* dir → vec of all audio, sorted, no ._* */ }
}
```

**Step 2: Run** `cargo test paths` → FAIL (not implemented)

**Step 3: Implementation**

```rust
use std::path::{Path, PathBuf};

pub const LAUNCH_EXTS: &[&str] = &[".aiff", ".aif", ".ape", ".dsf", ".flac", ".m4a", ".mp3", ".mp4", ".ogg", ".opus", ".tak", ".wav", ".wv"];
pub const EMBED_EXTS: &[&str] = &[".aiff", ".aif", ".ape", ".dsf", ".flac", ".m4a", ".mp3", ".mp4", ".ogg", ".opus", ".wav", ".wv"];

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("Path does not exist: {0}")]
    Missing(PathBuf),
    #[error("Unsupported audio file: {0}")]
    Unsupported(PathBuf),
    #[error("No supported audio files found directly inside: {0}")]
    Empty(PathBuf),
}

fn has_ext(path: &Path, exts: &[&str]) -> bool {
    path.extension().and_then(|e| e.to_str())
        .is_some_and(|e| exts.contains(&format!(".{}", e.to_lowercase()).as_str()))
}

fn expand(raw: &str) -> Result<PathBuf, PathError> {
    let expanded = if let Some(rest) = raw.strip_prefix("~/") {
        std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join(rest)
    } else { PathBuf::from(raw) };
    let canonical = expanded.canonicalize().map_err(|_| PathError::Missing(expanded.clone()))?;
    Ok(canonical)
}

fn audio_children(dir: &Path, exts: &[&str]) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir).into_iter().flatten().flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && has_ext(p, exts) && !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("._")))
        .collect();
    files.sort();
    files
}

/// First audio file for COVIT launch (parity with cov_launcher.resolve_audio_path).
pub fn resolve_audio_path(raw: &str) -> Result<PathBuf, PathError> {
    let candidate = expand(raw)?;
    if candidate.is_file() {
        return if has_ext(&candidate, LAUNCH_EXTS) { Ok(candidate) } else { Err(PathError::Unsupported(candidate)) };
    }
    audio_children(&candidate, LAUNCH_EXTS).into_iter().next().ok_or(PathError::Empty(candidate))
}

/// All embed targets (parity with embed_art.target_files).
pub fn target_files(raw: &str) -> Vec<PathBuf> {
    let Ok(candidate) = expand(raw) else { return vec![] };
    if candidate.is_file() {
        return if has_ext(&candidate, EMBED_EXTS) { vec![candidate] } else { vec![] };
    }
    audio_children(&candidate, EMBED_EXTS)
}

pub fn is_audio(path: &Path) -> bool { has_ext(path, LAUNCH_EXTS) }
```

**Step 4: Run** `cargo test paths` → PASS

**Step 5: Commit** `feat(paths): deterministic audio path resolution`

### Task 2.2: `config.rs`

**Files:** Create `src/config.rs`

**Step 1: Failing tests** — parse defaults, partial TOML merge, env override (`COV_LIBRARY_ROOT`, `COV_CONFIG`), `~` expansion, `default_mode` validation.

**Step 2: Implementation**

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub library_root: PathBuf,
    pub default_mode: Mode,
    pub theme: String,
    pub covit_path: PathBuf,
    pub log_path: PathBuf,
    pub output_basename: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Mode { Save, Embed }

impl Default for Config {
    fn default() -> Self {
        let home = || std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        Self {
            library_root: home().join("Music"),
            default_mode: Mode::Save,
            theme: "default".into(),
            covit_path: home().join(".local/bin/covit"),
            log_path: home().join("Library/Logs/cov-toolkit.log"),
            output_basename: "cover".into(),
        }
    }
}

impl Config {
    pub fn path(override_path: Option<&std::path::Path>) -> PathBuf {
        if let Some(p) = override_path { return p.to_path_buf(); }
        if let Some(env) = std::env::var_os("COV_CONFIG") { return PathBuf::from(env); }
        directories::ProjectDirs::from("xyz", "musichoarders", "cov")
            .map(|d| d.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from(".cov.toml"))
    }

    pub fn load(override_path: Option<&std::path::Path>) -> anyhow::Result<Self> {
        let path = Self::path(override_path);
        let mut cfg: Config = match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => { Self::write_default(&path)?; Config::default() }
            Err(e) => return Err(e.into()),
        };
        cfg.expand_tildes();
        if let Ok(root) = std::env::var("COV_LIBRARY_ROOT") { cfg.library_root = root.into(); }
        Ok(cfg)
    }

    fn write_default(path: &std::path::Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        std::fs::write(path, toml::to_string_pretty(&Config::default())?)?;
        Ok(())
    }

    fn expand_tildes(&mut self) { /* expand ~ for library_root, covit_path, log_path */ }
}
```

**Step 3:** `cargo test config` → PASS. **Step 4: Commit** `feat(config): ProjectDirs config with env overrides`

### Task 2.3: `launcher.rs` — golden contract

**Files:** Create `src/launcher.rs`, `tests/launch_contract.rs`

**Step 1: Failing golden tests** — exact argv, exact payload, quoting, attribution.

```rust
use cov::launcher::*;

fn req(embed: bool) -> LaunchRequest {
    LaunchRequest {
        audio_path: "/Music/Artist/Album/01 Song.mp3".into(),
        embed, output: "cover".into(),
        covit: "/Users/test/.local/bin/covit".into(),
        self_exe: "/Users/test/.local/bin/cov".into(),
        artist: None, album: None, identifier: None, country: None, resolution: None, sources: None,
    }
}

#[test]
fn save_argv_matches_legacy() {
    assert_eq!(covit_argv(&req(false)), vec![
        "/Users/test/.local/bin/covit", "--address", "covers.musichoarders.xyz",
        "--input", "/Music/Artist/Album/01 Song.mp3",
        "--primary-output", "cover", "--primary-overwrite",
        "--remote-agent", "Riley COV Toolkit",
        "--remote-text", "Using COV at https://covers.musichoarders.xyz/ — select artwork to save it beside this album.",
    ]);
}

#[test]
fn embed_argv_includes_primary_command() {
    let argv = covit_argv(&req(true));
    let pos = argv.iter().position(|a| a == "--primary-command").unwrap();
    assert_eq!(argv[pos + 1], "'/Users/test/.local/bin/cov' 'embed' '\"@covit_path@\"' '/Music/Artist/Album' '--rescan-swinsian'");
    assert!(argv.iter().any(|a| a.contains("https://covers.musichoarders.xyz/")));
}

#[test]
fn forced_quote_escapes_single_quotes() {
    assert_eq!(forced_quote("it's"), "'it'\"'\"'s'");
}

#[test]
fn background_payload_matches_legacy_shape() {
    let payload = background_payload(&covit_argv(&req(false)), "/Users/test/Library/Logs/cov-toolkit.log");
    assert!(payload.starts_with(": > '/Users/test/Library/Logs/cov-toolkit.log'; /usr/bin/env 'PATH=/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin' /usr/bin/nohup "));
    assert!(payload.ends_with(" >> '/Users/test/Library/Logs/cov-toolkit.log' 2>&1 < /dev/null &"));
}

#[test]
fn query_options_appended_when_set() { /* artist/album/identifier/country/resolution/sources → --query-* pairs */ }
```

**Step 2: Implementation** (pure constructors + thin exec)

```rust
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LaunchRequest {
    pub audio_path: PathBuf, pub embed: bool, pub output: String,
    pub covit: PathBuf, pub self_exe: PathBuf,
    pub artist: Option<String>, pub album: Option<String>, pub identifier: Option<String>,
    pub country: Option<String>, pub resolution: Option<String>, pub sources: Option<String>,
}

pub const FIXED_PATH: &str = "/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin";
pub const AGENT: &str = "Riley COV Toolkit";
pub fn remote_text(embed: bool) -> String {
    format!("Using COV at https://covers.musichoarders.xyz/ — {}.",
        if embed { "select artwork to save and embed it into this album" } else { "select artwork to save it beside this album" })
}

pub fn forced_quote(v: &str) -> String { format!("'{}'", v.replace('\'', "'\"'\"'")) }

/// Python shlex.quote equivalent.
pub fn shlex_quote(v: &str) -> String {
    if !v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric() || "@%_+=:,./-".contains(c)) { return v.into(); }
    forced_quote(v)
}

pub fn embed_callback(self_exe: &Path, album_dir: &Path) -> String {
    format!("{} 'embed' '\"@covit_path@\"' {} '--rescan-swinsian'",
        forced_quote(&self_exe.display().to_string()), forced_quote(&album_dir.display().to_string()))
}

pub fn covit_argv(r: &LaunchRequest) -> Vec<String> {
    let mut argv: Vec<String> = [
        r.covit.display().to_string(), "--address".into(), "covers.musichoarders.xyz".into(),
        "--input".into(), r.audio_path.display().to_string(),
        "--primary-output".into(), r.output.clone(), "--primary-overwrite".into(),
        "--remote-agent".into(), AGENT.into(), "--remote-text".into(), remote_text(r.embed),
    ].into_iter().collect();
    if r.embed {
        argv.extend(["--primary-command".into(), embed_callback(&r.self_exe, &r.audio_path.parent().unwrap().to_path_buf())]);
    }
    for (flag, val) in [("--query-artist", &r.artist), ("--query-album", &r.album), ("--query-identifier", &r.identifier),
                        ("--query-country", &r.country), ("--query-resolution", &r.resolution), ("--query-sources", &r.sources)] {
        if let Some(v) = val { argv.extend([flag.into(), v.clone()]); }
    }
    argv
}

pub fn background_payload(argv: &[String], log: &str) -> String {
    let joined = argv.iter().map(|a| shlex_quote(a)).collect::<Vec<_>>().join(" ");
    format!(": > {}; /usr/bin/env 'PATH={FIXED_PATH}' /usr/bin/nohup {joined} >> {} 2>&1 < /dev/null &",
        shlex_quote(log), shlex_quote(log))
}

/// Thin executor: detached via osascript, or foreground when debugging.
pub fn launch(r: &LaunchRequest, log: &Path, foreground: bool) -> anyhow::Result<()> { /* validate covit executable; spawn per contract; error strings identical to legacy: "COVIT is missing or not executable: {path}", "Background launcher exited with status {n}" */ }
```

**Step 3:** `cargo test --test launch_contract` → PASS
**Step 4: Commit** `feat(launcher): COVIT launch contract with golden tests`

### Task 2.4: `embed.rs` — finalize from spike

**Files:** Create `src/embed.rs` (spike code from Task 1.2, cleaned), extend `tests/embed_matrix.rs` with dry-run + summary behavior via the CLI in Phase 3.

**Step 1: Public API**

```rust
pub struct EmbedReport { pub updated: Vec<PathBuf>, pub failed: Vec<(PathBuf, String)>, pub total: usize }

pub fn embed_file(path: &Path, data: &[u8], mime: &str) -> anyhow::Result<()> {
    // MP3: per spike — id3 crate with Version::Id3v23 if lofty cannot emit v2.3
    // all other formats: lofty push_picture path
}

pub fn embed_album(artwork: &Path, target: &str, dry_run: bool) -> anyhow::Result<EmbedReport> {
    // validate artwork file, mime ∈ {image/jpeg, image/png} (from extension, parity with mimetypes guess)
    // files = paths::target_files(target); empty → error "no supported audio files found: {target}"
    // per file: print WOULD EMBED (dry) | EMBEDDED | FAILED (stderr), never abort the album
    // print SUMMARY: {updated} updated, {failed} failed, {total} total
}

pub fn rescan_swinsian(files: &[PathBuf]) { /* macos::osascript(SWINSIAN_RESCAN, paths) */ }
```

**Step 2:** matrix green incl. second-run replace (embed artwork.png after artwork.jpg → still exactly 1 cover).
**Step 3: Commit** `feat(embed): full embedding engine`

### Task 2.5: `macos.rs` — osascript helpers

**Files:** Create `src/macos.rs`

Implementation (script bodies copied verbatim from legacy `bin/cov-swinsian`, `bin/cov-finder`, `bin/cov-choose`, `bin/cov-context`, `lib/embed_art.py:rescan_swinsian`):

```rust
pub fn osascript(script: &str, args: &[String]) -> anyhow::Result<String> {
    let mut cmd = std::process::Command::new("/usr/bin/osascript");
    cmd.arg("-e").arg(script).arg("--").args(args);
    let out = cmd.output()?;
    if !out.status.success() {
        anyhow::bail!(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub const FRONTMOST_SCRIPT: &str = r#"tell application "System Events" to name of first application process whose frontmost is true"#;
pub const SWINSIAN_TRACK_SCRIPT: &str = r#"tell application "Swinsian"
  if not running then error "Swinsian is not running."
  set chosenTrack to missing value
  try
    set selectedTracks to selection of front window
    if selectedTracks is not {} then set chosenTrack to item 1 of selectedTracks
  end try
  if chosenTrack is missing value then
    try
      set chosenTrack to current track
    end try
  end if
  if chosenTrack is missing value then error "Select or play a track in Swinsian first."
  return path of chosenTrack as text
end tell"#;
pub const FINDER_SELECTION_SCRIPT: &str = r#"tell application "Finder"
  set selectedItems to selection
  if selectedItems is {} then error "Select an audio file or album folder in Finder first."
  return POSIX path of (item 1 of selectedItems as alias)
end tell"#;
pub const CHOOSE_FOLDER_SCRIPT: &str = r#"set chosenFolder to choose folder with prompt "Choose an album folder for COV"
return POSIX path of chosenFolder"#;
pub const SWINSIAN_RESCAN_SCRIPT: &str = /* verbatim from embed_art.py */;

pub fn frontmost_app() -> anyhow::Result<String>;
pub fn swinsian_track_path() -> anyhow::Result<String>;
pub fn finder_selection() -> anyhow::Result<String>;
pub fn choose_folder() -> anyhow::Result<String>;
pub fn pbpaste() -> anyhow::Result<String>;
```

Tests: script-body unit assertions (contains key phrases, e.g. `rescan tags`); live behavior verified in the manual matrix (needs GUI).

**Commit** `feat(macos): osascript helpers`

### Task 2.6: `context.rs`

**Files:** Create `src/context.rs`

```rust
pub enum ContextSource { Swinsian, Finder, Clipboard(PathBuf) }

pub fn route(frontmost: &str, clipboard: &str) -> Option<ContextSource> {
    match frontmost {
        "Swinsian" => return Some(ContextSource::Swinsian),
        "Finder" => return Some(ContextSource::Finder),
        _ => {}
    }
    let expanded = expand_tilde(clipboard.trim());
    if !clipboard.trim().is_empty() && expanded.exists() { return Some(ContextSource::Clipboard(expanded)); }
    None
}

pub fn detect() -> anyhow::Result<(ContextSource, PathBuf)> {
    // frontmost via macos; on Clipboard/Swinsian/Finder resolve the concrete path
    // Err("No usable context found. Select a track in Swinsian or Finder, or copy a path.")
}
```

Unit tests for `route` with temp paths. **Commit** `feat(context): frontmost-app routing`

### Task 2.7: `doctor.rs`

**Files:** Create `src/doctor.rs`

Checks (parity with legacy `cov-doctor`, Python checks dropped, `library_root` added):

```rust
pub struct Check { pub label: String, pub ok: bool, pub detail: String }

pub fn run(cfg: &Config) -> Vec<Check> {
    // executable checks: covit_path, /usr/bin/open, /usr/bin/osascript, /Applications/Ghostty.app/Contents/MacOS/ghostty
    // library_root exists (TUI)
    // info line: log path
}

pub fn print_report(checks: &[Check]) -> bool /* ok */ {
    // "PASS  <label>: <detail>" / "FAIL  ..." / "INFO  Log: <path>"
    // final: "COV toolkit is ready." | "COV toolkit has missing requirements."
}
```

Test: `run` against the real machine asserts covit/open/osascript checks execute and report struct is populated; assert formatting on synthetic checks. **Commit** `feat(doctor): environment diagnostics`

---

## Phase 3 — CLI

### Task 3.1: `cli.rs` + `main.rs`

**Files:** Create `src/cli.rs`; rewrite `src/main.rs`

```rust
#[derive(clap::Parser)]
#[command(name = "cov", version, about = "COV integration toolkit — https://covers.musichoarders.xyz/")]
pub struct Cli {
    /// Path to config file
    #[arg(long, global = true)] pub config: Option<PathBuf>,
    /// Write TUI debug log (appended) to ~/Library/Logs/cov-tui.log
    #[arg(long, global = true)] pub debug: bool,
    #[command(subcommand)] pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Open COV for an audio file or album folder
    Open(OpenArgs),
    /// Embed existing local artwork without opening the browser
    Embed(EmbedArgs),
    /// Auto-detect source from Swinsian, Finder, or clipboard
    Context { mode: Option<Mode> },
    /// Use selected/playing Swinsian track
    Swinsian { mode: Option<Mode> },
    /// Use Finder selection
    Finder { mode: Option<Mode> },
    /// Prompt with a native folder chooser
    Choose { mode: Option<Mode> },
    /// Use a path copied to the clipboard
    Clipboard { mode: Option<Mode> },
    /// View or tail the COVIT log
    Log { mode: Option<LogMode> },
    /// Check environment
    Doctor,
    /// Launch the interactive TUI
    Tui(TuiArgs),
    /// Open the TUI in a dedicated Ghostty window
    Ghostty,
}

#[derive(clap::Args)]
pub struct OpenArgs {
    pub path: String,
    #[arg(long)] pub embed: bool,
    #[arg(long, default_value = "cover")] pub output: String,
    #[arg(long)] pub covit: Option<PathBuf>,
    #[arg(long)] pub log: Option<PathBuf>,
    #[arg(long)] pub artist: Option<String>,
    #[arg(long)] pub album: Option<String>,
    #[arg(long)] pub identifier: Option<String>,
    #[arg(long)] pub country: Option<String>,
    #[arg(long)] pub resolution: Option<String>,
    #[arg(long)] pub sources: Option<String>,
    #[arg(long)] pub foreground: bool,
}

#[derive(clap::Args)]
pub struct EmbedArgs {
    pub artwork: PathBuf,
    pub target: String,
    #[arg(long)] pub dry_run: bool,
    #[arg(long)] pub rescan_swinsian: bool,
}

#[derive(clap::Args)]
pub struct TuiArgs {
    /// Music library root (overrides config)
    #[arg(long)] pub library: Option<PathBuf>,
}

#[derive(Clone, clap::ValueEnum)]
pub enum LogMode { Show, Follow }
```

`main.rs`: parse → `Config::load(cli.config)` → dispatch; errors printed as `cov <sub>: {msg}` (usage errors exit 2 via clap; runtime errors exit 1). Mode-routing commands (`context/swinsian/finder/choose/clipboard`) resolve a path then funnel into the same `run_open(path, mode, …)` used by `open`.

**Step: failing CLI tests** (`tests/cli.rs`, assert_cmd): `--help` lists all subcommands; `open /nonexistent` → exit 1 + "Path does not exist"; `embed missing.jpg /tmp` → exit 1; `log show` on fresh `HOME` → "No COV log exists yet."; `doctor` runs and prints `PASS`/`INFO` lines.

**Step:** run green. **Commit** `feat(cli): full subcommand surface`

### Task 3.2: Parity checkpoint

Manual + scripted diff vs legacy toolkit (legacy still in tree):

**Step 1:** Write `tests/parity.sh` (not committed long-term; run locally): for a fixture album dir, run legacy `bin/cov-open --foreground` and new `cargo run -- open --foreground` against a **stub covit** (a temp script that echoes its argv) and diff the argv. Same for `cov-embed --dry-run` output.

**Step 2:** Record results in commit message. **Commit** `test: CLI parity checkpoint vs legacy toolkit`

---

## Phase 4 — TUI

Module layout under `src/tui/`: `mod.rs` (re-exports + `run()`), `app.rs`, `theme.rs`, `scanner.rs`, `matcher.rs`, `artwork.rs`, `images.rs`, `screens/{finder,form,logview,doctor,help,first_run}.rs`, `widgets/{input,list,preview,footer}.rs`.

### Task 4.1: `theme.rs` + `themes/default.toml`

**Files:** Create both.

```toml
# themes/default.toml
[general]
background = "#101319"
foreground = "#e8eaf0"
border = "#334155"
border_focused = "#01acd7"

[input]
prompt = "#01acd7"
text = "#e8eaf0"
cursor = "#87d7ff"

[list]
text = "#e8eaf0"
selected_bg = "#1e2a3a"
selected_fg = "#87d7ff"
match_fg = "#01acd7"
badge_missing = "#e06c75"
badge_sidecar = "#e5c07b"
badge_partial = "#d19a66"
badge_complete = "#98c379"
badge_checking = "#5c6370"

[preview]
title = "#87d7ff"
label = "#5c6370"
value = "#e8eaf0"

[footer]
key = "#01acd7"
text = "#5c6370"
```

theme.rs: `Theme` structs deserialize + `ratatui::style::Style` accessors; `Theme::load(name_or_path)` — builtin default via `include_str!("../../themes/default.toml")`, else `<config_dir>/themes/<name>.toml`, else absolute path. Tests: default parses; custom file loads; missing → fallback default.

**Commit** `feat(tui): theme system + default theme`

### Task 4.2: `scanner.rs`

```rust
pub struct Album {
    pub dir: PathBuf,
    pub rel: String,      // relative to library_root — match text
    pub display: String,  // "Shore · CD1" (last two components, or last one)
    pub tracks: Vec<PathBuf>, // sorted, EMBED_EXTS-filtered… use LAUNCH_EXTS for finder parity
}

pub enum ScanMsg { Batch { epoch: u64, albums: Vec<Album> }, Done { epoch: u64, total: usize } }

pub fn spawn_scan(root: PathBuf, epoch: u64, cancel: Arc<AtomicU64>, tx: crossbeam::channel::Sender<ScanMsg>) -> JoinHandle<()>;
```

Rules: walkdir recursive, skip hidden dirs, files matching `LAUNCH_EXTS` and not `._*`; group by parent dir; emit batches of 200; check `cancel` (current epoch) between batches. Tests: temp tree (`A/Album1/01.mp3`, `A/Album2/CD1/01.flac`, `A/Album2/CD2/01.flac`, `._junk.mp3`, `.hidden/x.mp3`, `empty/`) → 3 albums, correct rel/display (`Album2 · CD1`).

**Commit** `feat(tui): library scanner worker`

### Task 4.3: `matcher.rs`

Wrap the Task 1.3 winner behind:

```rust
pub struct Matcher { /* nucleo worker or frizbee recompute */ }
impl Matcher {
    pub fn new() -> Self;
    pub fn replace_items(&mut self, items: Vec<Arc<Album>>);           // scan batch appended
    pub fn query(&mut self, pattern: &str);                            // re-filter
    pub fn results(&self) -> &[Arc<Album>];                            // ranked
}
```

Tests: fuzzy match ordering (`"shore"` ranks `Fleet Foxes/Shore` above `Shoreline` noise), empty query = all items in scan order, append during search works.

**Commit** `feat(tui): fuzzy matcher`

### Task 4.4: `artwork.rs` — whole-set inspector

```rust
#[derive(Debug, Clone, Default)]
pub struct ArtworkStatus { pub sidecar: Option<PathBuf>, pub embedded: EmbeddedState }

#[derive(Debug, Clone, Default)]
pub enum EmbeddedState { #[default] Checking, None, Partial { with: usize, total: usize }, All { total: usize } }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Badge { Checking, Missing, SidecarOnly, Partial, Complete }

impl ArtworkStatus {
    pub fn badge(&self) -> Badge {
        match (&self.embedded, &self.sidecar) {
            (EmbeddedState::Checking, _) => Badge::Checking,
            (EmbeddedState::None, None) => Badge::Missing,
            (EmbeddedState::None, Some(_)) => Badge::SidecarOnly,
            (EmbeddedState::Partial { .. }, _) => Badge::Partial,
            (EmbeddedState::All { .. }, _) => Badge::Complete,
        }
    }
}

pub enum ArtworkMsg { Status { epoch: u64, dir: PathBuf, status: ArtworkStatus, preview: Option<Vec<u8>> } }

pub fn inspect(album: &Album) -> (ArtworkStatus, Option<Vec<u8>>) {
    // sidecar: cover.{jpg,jpeg,png} (case-insensitive) in album.dir
    // embedded: lofty tag-header read of EVERY track in album.tracks → count with ≥1 picture
    // preview bytes: sidecar file bytes, else first embedded picture bytes, else None
}

pub fn spawn_worker(rx: Receiver<InspectReq>, epoch: Arc<AtomicU64>, tx: Sender<ArtworkMsg>) -> JoinHandle<()>;
```

Tests with fixtures: album of 2 flac — neither embedded → `None`; embed one → `Partial { with: 1, total: 2 }` + `Badge::Partial`; embed both → `All`. Sidecar-only dir → `SidecarOnly`. Epoch: stale message dropped by reducer test in app (Task 4.6).

**Commit** `feat(tui): whole-set artwork inspector`

### Task 4.5: `images.rs`

```rust
pub enum PreviewKind { Kitty(/* protocol */), Halfblock(/* protocol */), Text(String) }

pub fn make_picker() -> ratatui_image::picker::Picker {
    // Picker::from_query_stdio() with 100ms timeout; on failure → Picker::halfblocks()
}

pub fn protocol_for(picker: &mut Picker, bytes: &[u8]) -> PreviewKind {
    // image::load_from_memory → picker.new_resize_protocol(img)
    // decode failure → PreviewKind::Text("no artwork")
}
```

Unit test: text fallback on garbage bytes. Kitty vs halfblock verified manually (Ghostty vs Terminal.app) — recorded in manual matrix.

**Commit** `feat(tui): image preview with fallback chain`

### Task 4.6: `app.rs` — state machine + reducer

```rust
pub enum Screen { FirstRun(FirstRunState), Finder, Form(FormState), Log, Doctor, Help }

pub struct App {
    pub cfg: Config, pub theme: Theme,
    pub albums: Vec<Arc<Album>>,          // scan-accumulated
    pub matcher: Matcher,
    pub input: String, pub cursor: usize,
    pub selected: usize, pub scroll: usize,
    pub statuses: HashMap<PathBuf, ArtworkStatus>,
    pub previews: HashMap<PathBuf, PreviewKind>,
    pub filter: Filter,                    // All | Missing | NeedsEmbed
    pub screen: Screen,
    pub scan_epoch: Arc<AtomicU64>, pub preview_epoch: Arc<AtomicU64>,
    pub status_line: Option<String>,       // transient messages ("COV opened for …")
}

pub enum Filter { All, Missing, NeedsEmbed }
impl Filter { pub fn next(self) -> Self } // All→Missing→NeedsEmbed→All
    pub fn allows(&self, badge: Badge) -> bool // Missing→badge==Missing; NeedsEmbed→SidecarOnly|Partial
}

pub enum Msg { Key(KeyEvent), Scan(ScanMsg), Artwork(ArtworkMsg), Tick }

impl App {
    pub fn reduce(&mut self, msg: Msg) -> Option<Action>; // Action = Launch { album, embed } | Quit | Rescan…
}
```

Reducer rules (unit-tested):
- Scan msg with `epoch != *scan_epoch` → dropped. Same for Artwork vs `preview_epoch`.
- Key char → input append + `preview_epoch` bump? no — input change re-matches; selection move bumps `preview_epoch` and enqueues inspect for newly selected album.
- `^f` cycles filter; filtered view = matcher results ∩ filter.allows(badge(status or Checking)).
- Enter/^E on selection → `Action::Launch` (embed flag per key; overrides from FormState if set).
- `^r` → bump `scan_epoch`, clear albums/statuses, `Action::Rescan`.
- `?`/`^d`/`^l`/`^o` → push screens; Esc pops. `q`/`^c` on Finder → Quit.
- Frecency: successful launch boosts album's session score (matched rank bonus); session-only.

Tests: full reducer suite per rule above (no terminal needed).

**Commit** `feat(tui): app state machine + reducer`

### Task 4.7: Finder screen + widgets render

**Files:** `screens/finder.rs`, `widgets/{input,list,preview,footer}.rs`

Layout (ratatui `Layout`):

```text
┌─────────────────────────────────────────────┐
│ input (height 3, rounded, focused border)   │
├──────────────────────┬──────────────────────┤
│ list (60%)           │ preview (40%)        │
├──────────────────────┴──────────────────────┤
│ footer (height 1)                           │
└─────────────────────────────────────────────┘
```

- **input**: `> {query}` + `{n} albums` right-aligned; cursor block.
- **list**: per row `"{badge} {display}"`; badge glyph/color from theme (`… ○ ◆ ◐ ●`); selected row style; fuzzy match positions highlighted with `match_fg`.
- **preview**: image widget (top, aspect-fit) + metadata lines: title/album/artist/year/format/track count, then `sidecar: ✓/✗  embedded: n/m`. Text fallback renders `PreviewKind::Text` centered.
- **footer**: `enter: save  ^e: embed  ^o: options  ^f: {filter}  ^r: rescan  ^l: log  ^d: doctor  ?: help  q: quit` with `key` color on keys.

Tests: ratatui `TestBackend::assert_buffer` — seeded App renders album names, badges, footer; empty state renders "scanning…" / "no albums".

**Commit** `feat(tui): finder screen + widgets`

### Task 4.8: Form, Log, Doctor, Help, FirstRun screens

- **Form** (`screens/form.rs`): fields mode (toggle), artist, album, identifier, resolution, sources; Tab/Shift-Tab moves focus, Space toggles mode, Enter confirms (returns to Finder, values stored for next launch), Esc cancels. TestBackend: renders labels + values; reducer: focus cycle + confirm path.
- **Log**: read-only scrollback of the COVIT log (last 200 lines, re-read on open and on `r`); `Esc` back.
- **Doctor**: renders `doctor::run(cfg)` checks as PASS/FAIL list; `r` re-runs.
- **Help**: key table overlay.
- **FirstRun**: shown when `library_root` missing/invalid: single path input, validation on Enter (exists / is dir / contains ≥1 audio-bearing dir via scanner quick probe), success writes config + starts scan; error shown inline.

**Commit** `feat(tui): secondary screens`

### Task 4.9: `tui/mod.rs` — event loop + launch wiring

```rust
pub fn run(cfg: Config, debug: bool) -> anyhow::Result<()> {
    // crossterm raw mode + alternate screen; tracing to ~/Library/Logs/cov-tui.log when debug (append)
    // spawn scanner (skip when FirstRun active), artwork worker
    // loop: crossbeam select! { crossterm event (30ms poll), scan rx, artwork rx } → Msg → reduce → Action
    // Action::Launch → launcher::launch (foreground=false) using album dir + form overrides → status_line = "COV opened for …" + frecency boost
    // restore terminal on exit (Drop guard)
}
```

`cov ghostty` = `open -na Ghostty.app --args -e <current_exe> tui` (exact parity with legacy wrapper).

Manual smoke: `cargo run -- tui` on Ghostty. **Commit** `feat(tui): event loop + launch integration`

---

## Phase 5 — Cutover & docs

### Task 5.1: `install.sh` rewrite + Makefile

```zsh
#!/bin/zsh
set -eu
SCRIPT_DIR=${0:A:h}
BIN_DIR=${HOME}/.local/bin
command -v cargo >/dev/null 2>&1 || { print -u2 "Rust toolchain required (https://rustup.rs)"; exit 1; }
(cd "$SCRIPT_DIR" && cargo build --release)
/bin/mkdir -p "$BIN_DIR"
target="$SCRIPT_DIR/target/release/cov"
link="$BIN_DIR/cov"
if [[ -e "$link" && ! -L "$link" ]]; then print -u2 "Refusing to replace a real file: $link"; exit 1; fi
/bin/ln -sfn "$target" "$link"
print "Installed: $link -> $target"
# Remove legacy Python-toolkit symlinks that point into this repo's old bin/
for name in cov-open cov-open-embed cov-embed cov-finder cov-choose cov-swinsian cov-clipboard cov-context cov-tui cov-ghostty cov-log cov-doctor; do
  old="$BIN_DIR/$name"
  if [[ -L "$old" && "$(/usr/bin/readlink "$old")" == "$SCRIPT_DIR/bin/"* ]]; then
    /bin/rm "$old"; print "Removed legacy: $old"
  fi
done
```

Makefile:

```make
.PHONY: build test lint fmt verify install tui bench fixtures
build: ; cargo build --release
test: ; cargo test
lint: ; cargo clippy --all-targets -- -D warnings
fmt: ; cargo fmt --check
verify: fmt lint test
install: ; ./install.sh
tui: ; cargo run --release -- tui
bench: ; cargo bench --bench matcher
fixtures: ; zsh tests/fixtures/generate.sh
```

**Commit** `chore: cargo-based install.sh + Makefile`

### Task 5.2: Update integrations

- `integrations/*.applescript` (4 files): replace `bin/cov-open` → `~/.local/bin/cov open`, `bin/cov-open-embed` → `~/.local/bin/cov open --embed` (read each file; keep every other line identical).
- `integrations/keyboard-maestro/COV Toolkit.kmmacros`: plist — update command strings the same way (it is XML; edit as text).
- `docs/INTEGRATIONS.md`: update all command examples (`cov open`, `cov open --embed`, `cov embed`, `cov tui`…); Raycast snippet updated.

**Commit** `refactor(integrations): route wrappers through cov binary`

### Task 5.3: README + docs

- `README.md`: badges (Python → Rust), Quick Start (rustup prerequisite + `./install.sh`), Commands table → subcommand table (from design §4), Structure section → new tree, Embedding section unchanged in substance (note lofty), requirements note updated (no Python/mutagen/textual).
- `docs/REPOSITORY_SETUP.md`, `docs/TROUBLESHOOTING.md`: cargo workflow; remove Python references.
- `docs/IMPLEMENTATION_SUMMARY.md`: add Rust rewrite entry.
- `docs/TESTING.md`: manual matrix template (formats × embed; Ghostty Kitty vs Terminal.app halfblock; parity results).
- `CHANGELOG.md`: `## 2.0.0` entry (full rewrite, TUI, subcommand migration table).

**Commit** `docs: Rust rewrite documentation`

### Task 5.4: AGENTS.md

Update: Architecture (Rust modules), Verification block:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cov --help
cov embed --help
```

Keep Safety section verbatim (interactive selection, no internal API, attribution + full address, embed-on-copies, wrappers delegate — now to `cov open`).

**Commit** `docs: update AGENTS.md for Rust toolkit`

### Task 5.5: Delete legacy

```bash
git rm -r lib bin pyproject.toml
```

Final grep for stragglers: `rg -n "cov-open|cov-embed |bin/cov|python|mutagen|textual" --glob '!docs/plans/**' --glob '!CHANGELOG.md'` → fix or justify each hit.

**Commit** `chore: remove legacy Python/zsh toolkit`

---

## Phase 6 — Verification

### Task 6.1: Automated gate

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
./install.sh && cov --help && cov embed --help && cov doctor
```

Delete `docs/plans/spike-findings.md` (promoted into tests), keep `benches/matcher.rs`.

### Task 6.2: Manual matrix (record in `docs/TESTING.md`)

1. Copy a real album to `/tmp`, `cov open --embed <copy>` → COV opens, cover saves, tracks embedded (verify via `cov embed --dry-run` + Swinsian rescan).
2. Per-format embed on fixture copies; re-read covers via `cov embed` second run (replace semantics).
3. TUI on Ghostty: Kitty image renders; scanning a large root is live/incremental; `^f` cycles; form launch works; first-run screen appears with bogus `library_root`.
4. TUI on Terminal.app: halfblock fallback renders.
5. Swinsian + Finder context commands against real selections.
6. `cov log follow` during a launch.

**Commit** `test: manual verification matrix`, then PR/merge per user choice.

---

## Execution notes

- Legacy toolkit stays functional until Task 5.5; never delete earlier.
- Every `launcher` change must keep `tests/launch_contract.rs` golden strings green — they ARE the parity contract.
- If lofty API differs from §Task 1.2 expectations, the spike adjusts the implementation, not the test expectations (one cover, tags preserved, per-format container semantics).
- `.dsf` fixture optional: if the ffmpeg dsd encoder is unavailable, DSF embed runs in the manual matrix against a real file instead.
