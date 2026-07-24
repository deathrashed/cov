# COV Toolkit — Rust Rewrite Design

**Date:** 2026-07-24
**Status:** Approved (pending implementation plan)
**Scope:** Full rewrite of the COV Integration Toolkit (Python + zsh) as a single Rust binary with a television-inspired TUI.

---

## 1. Context & Goals

The current toolkit is 12 zsh entry points over two Python libraries (`cov_launcher.py`, `embed_art.py`) plus a Textual TUI. It wraps the COVIT binary (`~/.local/bin/covit`) to open interactive cover search at `https://covers.musichoarders.xyz/` and embeds selected artwork into audio files via mutagen.

Goals of the rewrite:

1. Single self-contained Rust binary `cov` with subcommands, replacing all Python/zsh.
2. Behavioural parity for the COVIT launch contract and the embedding engine.
3. A beautiful TUI inspired by [television](https://github.com/alexpasmantier/television): fuzzy album finder + preview pane + restyled overrides form.
4. Meaningful new features: artwork-state badges over the whole library, missing/partial-artwork filters, in-terminal cover preview.

Non-goals (v1): persistent library index/cache, batch queue, multi-disc collapsing with recursive embed, prebuilt binaries, non-macOS platforms.

## 2. Locked Decisions

| # | Decision |
|---|---|
| D1 | One binary, subcommands only. The 12 `bin/cov-*` scripts are deleted; integrations are updated to call `cov <sub>`. |
| D2 | TUI = fuzzy album finder (main) + restyled overrides form (screen) + log/doctor/help screens. |
| D3 | Tagging via `lofty`; MP3 ID3v2.3 parity verified in spike, `id3` crate fallback if needed. |
| D4 | Fuzzy matcher: `nucleo` vs `frizbee` decided by a committed benchmark (`benches/matcher.rs`). Default candidate `nucleo` (streaming-friendly worker). |
| D5 | Image preview fallback chain: Kitty graphics → halfblocks → text placeholder. Halfblocks is first-class. |
| D6 | Config resolution via `directories::ProjectDirs`; nothing hard-codes `~/.config`. |
| D7 | Multi-disc parity boundary: one finder entry per audio-bearing directory (`Album/CD1`, `Album/CD2` separate), matching today's non-recursive launch/embed semantics. |
| D8 | Artwork state is whole-set verified with `Checking`/`Partial` states; never green from the first track. |
| D9 | CLI flags follow an explicit override matrix (§5); env vars limited to `COV_LIBRARY_ROOT` and `COV_CONFIG`. |
| D10 | Two log streams (§9): COVIT output truncated per launch; TUI tracing appended, debug-gated. |
| D11 | Spike fate: matcher work → committed bench; tagging checks → integration tests; all other spike code deleted before release. No `spikes/` dir survives. |
| D12 | v1 requires a Rust toolchain (source build in `install.sh`). Prebuilt checksum-verified binaries are a follow-up milestone. |
| D13 | Frecency is session-only (in-memory); no history file in v1. |
| D14 | No tokio. Concurrency = `std::thread` + `crossbeam` channels + `Arc<AtomicU64>` epochs. |

## 3. Architecture

Single binary crate at repo root:

```
cov/
├── Cargo.toml
├── src/
│   ├── main.rs          # clap dispatch
│   ├── cli.rs           # subcommand definitions (clap derive)
│   ├── launcher.rs      # COVIT command building + detached launch (pure fns + thin exec)
│   ├── paths.rs         # deterministic audio-path resolution
│   ├── embed.rs         # lofty embedding engine
│   ├── context.rs       # frontmost-app detection, clipboard (pbpaste)
│   ├── macos.rs         # osascript helpers: Swinsian track/rescan, Finder selection, folder chooser
│   ├── doctor.rs        # environment checks
│   ├── config.rs        # ProjectDirs config + precedence
│   └── tui/
│       ├── app.rs       # state machine over screens
│       ├── screens/     # finder, form, log, doctor, help, first-run
│       ├── widgets/     # input, list, preview, footer
│       ├── scanner.rs   # library walk worker
│       ├── matcher.rs   # fuzzy filter worker
│       ├── artwork.rs   # whole-set artwork inspection worker
│       ├── images.rs    # Kitty/halfblock/text preview
│       └── theme.rs     # TOML themes
├── themes/              # TOML themes (default: brand cyan #01acd7 on #101319)
├── benches/matcher.rs   # nucleo vs frizbee, 5k-album synthetic corpus
└── tests/               # unit, integration, fixtures, golden launch contracts
```

Key crates: `clap` (derive), `anyhow`, `thiserror`, `lofty`, `ratatui`, `crossterm`, `ratatui-image`, `serde`, `toml`, `directories`, `walkdir`, `crossbeam`, `image` (dimension/decode), plus the matcher winner (D4).

## 4. CLI Surface & Migration

| Old command | New subcommand |
|---|---|
| `cov-open <PATH> [opts]` | `cov open <PATH> [opts]` |
| `cov-open-embed <PATH> [opts]` | `cov open --embed <PATH> [opts]` |
| `cov-embed <ART> <TARGET> [--dry-run] [--rescan-swinsian]` | `cov embed <ART> <TARGET> […]` |
| `cov-context [save\|embed]` | `cov context [save\|embed]` |
| `cov-swinsian [save\|embed]` | `cov swinsian [save\|embed]` |
| `cov-finder [save\|embed]` | `cov finder [save\|embed]` |
| `cov-choose [save\|embed]` | `cov choose [save\|embed]` |
| `cov-clipboard [save\|embed]` | `cov clipboard [save\|embed]` |
| `cov-tui` | `cov tui` |
| `cov-ghostty` | `cov ghostty` |
| `cov-log [show\|follow]` | `cov log [show\|follow]` |
| `cov-doctor` | `cov doctor` |

`cov open` flags (parity): `--embed --output --covit --log --artist --album --identifier --country --resolution --sources --foreground`.

## 5. Config & Override Matrix

Config file: `<ProjectDirs config_dir>/cov/config.toml`.

```toml
library_root    = "~/Music"
default_mode    = "save"     # "save" | "embed" — enter-key action in TUI
theme           = "default"  # name in themes/ or absolute path
covit_path      = "~/.local/bin/covit"
log_path        = "~/Library/Logs/cov-toolkit.log"
output_basename = "cover"
```

Precedence per key (high → low): CLI flag > env var > config file > built-in default.

| Key | Env var | CLI flag | Applies to |
|---|---|---|---|
| (config file path) | `COV_CONFIG` | `--config` | global |
| `library_root` | `COV_LIBRARY_ROOT` | `--library` | `tui` only |
| `covit_path` | — | `--covit` | `open` |
| `log_path` | — | `--log` | `open`, `log` |
| `output_basename` | — | `--output` | `open` |
| `default_mode` | — | — | `tui` (config only) |
| `theme` | — | — | `tui` (config only) |

No other env vars or flags. First run: missing config → `cov tui` writes defaults; invalid/absent `library_root` → first-run setup screen (validated path input: exists, is dir, contains ≥1 audio-bearing directory) that persists the choice. `cov doctor` validates `library_root`. Non-TUI commands never require it.

## 6. `paths.rs` Specification

Extension sets (case-insensitive):

- `LAUNCH_EXTS` = `.aiff .aif .ape .dsf .flac .m4a .mp3 .mp4 .ogg .opus .tak .wav .wv`
- `EMBED_EXTS` = `LAUNCH_EXTS` minus `.tak`

`resolve_audio_path(p)`:

1. Expand `~`, canonicalize; error if missing.
2. File → require `LAUNCH_EXTS` membership, else `Unsupported audio file`.
3. Dir → immediate children only (non-recursive), files in `LAUNCH_EXTS`, excluding AppleDouble `._*`, sorted by full path (identical to Python `sorted(Path)` for siblings); return first; empty → `No supported audio files found directly inside`.

`target_files(p)`: same rules with `EMBED_EXTS`; returns the whole sorted list (single file → vec of one if supported).

## 7. Launcher Contract (Behavioural Parity)

`launcher.rs` splits into pure constructors returning `(argv: Vec<String>, applescript_payload: String)` and a thin executor. Contract pinned by golden tests:

- Exact COVIT argv: `--address covers.musichoarders.xyz --input <audio> --primary-output <base> --primary-overwrite --remote-agent "Riley COV Toolkit" --remote-text <attribution>`, plus `--primary-command '<cov embed> "@covit_path@" <album> --rescan-swinsian'` when embedding (forced-quote algorithm preserved), plus `--query-*` options when set.
- Attribution/remote strings always contain the full `https://covers.musichoarders.xyz/` address.
- Detached launch: log truncated, fixed sanitized `PATH`, `nohup … >> log 2>&1 < /dev/null &` executed via `/usr/bin/osascript`; `--foreground` runs in-process.
- Exit codes and stdout messages match today.

Cover selection remains interactive in the browser; no COV internal API use.

## 8. Embedding Engine

`lofty` with mutagen-equivalent semantics — replace front cover only, preserve all other tags, per-file error isolation, `WOULD EMBED`/`EMBEDDED`/`FAILED`/`SUMMARY` output, exit 1 on any failure, `--dry-run`, `--rescan-swinsian` via osascript.

| Format | Container / frame |
|---|---|
| MP3 | ID3v2 `APIC` (type 3, desc "Front Cover") — **v2.3 write verified in spike; `id3` crate fallback** |
| FLAC | Native picture block (type 3) |
| M4A/MP4 | `covr` (png/jpeg by mime) |
| Ogg/Opus | `METADATA_BLOCK_PICTURE` |
| WAV/AIFF/DSF | ID3 `APIC` |
| APE/WavPack | APEv2 binary item `Cover Art (Front)` with `cover\0` prefix |

Spike (promoted to `tests/embed_matrix.rs`): per format — read, strip old front cover, write picture, preserve unrelated tags. Fixtures: tiny real files generated once (ffmpeg/afconvert), committed under `tests/fixtures/`, round-trip asserted via re-read; mutagen cross-check where cheap.

## 9. Logging

Two distinct streams:

| Stream | Path | Mode |
|---|---|---|
| COVIT process output | `~/Library/Logs/cov-toolkit.log` (config `log_path`, `--log`) | **truncated per `cov open` launch** (parity) |
| TUI internal tracing | `~/Library/Logs/cov-tui.log` | **appended**; created only when `--debug` is passed |

`cov log [show|follow]` reads the COVIT stream, as today.

## 10. TUI Design

Skeleton (television layout, cov content): fuzzy input + album count on top; album list left with badges; preview pane right (image + metadata + artwork state); keybinding footer.

**Screens** — one `App` state machine:

1. **Finder** (home): type to filter; badges per album.
2. **Overrides form** (`^o`): mode toggle, artist, album, identifier, resolution, sources; applies to next launch; Esc returns.
3. **Log viewer** (`^l`): tail of the COVIT stream.
4. **Doctor panel** (`^d`): pass/fail checks.
5. **Help popup** (`?`): keybinding overlay.
6. **First-run setup**: shown only when `library_root` is unset/invalid.

**Keys** (television conventions): `↑/↓`/`^p/^n` navigate, `enter` save, `^e` embed, `^o` options, `^f` cycle filter, `^r` rescan, `^l` log, `^d` doctor, `?` help, `esc`/`q`/`^c` back/quit.

**Scanner/matcher**: walkdir worker recursively walks `library_root`; each directory containing ≥1 audio file (AppleDouble excluded) = one album entry, streamed over a channel. Matcher worker incrementally re-filters as entries arrive (live UI during scan; no persistent index in v1). Display name = path relative to `library_root` (`Fleet Foxes/Shore/CD1` → `Shore · CD1`); matching runs on the full relative path so multi-disc releases surface together (D7). Sort = fuzzy score with session-only frecency boost (D13).

**Epoch discipline**: `scan_epoch` bumped by `^r`/config change; scanner+matcher messages tagged, stale dropped. `preview_epoch` bumped on selection move; artwork worker checks `Arc<AtomicU64>` before decoding or delivering — no stale previews.

**Artwork state** (D8) — two dimensions, whole-set verified by the artwork worker (tag-header reads of **every** track; visible albums first; session-memoized):

- Sidecar: `Present(path)` | `Absent` (`cover.jpg|jpeg|png` in album dir)
- Embedded: `Checking` → `None` | `Partial { done, total }` | `All`

Badges: `…` checking, `○` missing (red: no sidecar, no embedded), `◆` sidecar-only (yellow: needs embed), `◐` partial (orange: needs embed completion), `●` all-embedded (green). `^f` cycles **All → Missing → Needs embed (sidecar-only ∪ partial) → All**. Preview pane spells out state + counts.

**Images** (D5): sidecar cover preferred, else first embedded picture found, else placeholder. Protocol picker: Kitty (Ghostty) → halfblocks → text.

**Themes**: TOML files in `themes/`, television-style keys (`input`, `border`, `match`, `selection`, `badge_ok`, `badge_warn`, `badge_missing`, `footer`…). Built-in default: `#01acd7` accents on `#101319`/`#1e1e1e`, rounded borders.

## 11. Repo Migration

- **Delete:** `lib/`, `bin/`, `pyproject.toml`.
- **Add:** `Cargo.toml`, `src/`, `themes/`, `tests/`, `benches/`.
- `Makefile` rewritten around cargo: `build test lint fmt verify`.
- `install.sh` rewritten: requires Rust toolchain (README documents rustup prerequisite), `cargo build --release`, installs `cov` into `~/.local/bin` (D12).
- `integrations/*.applescript`: updated to call `~/.local/bin/cov open` / `cov open --embed`; wrappers still contain zero COVIT arguments (delegation rule).
- Docs updated: README (commands, badges, build), `INTEGRATIONS.md`, `REPOSITORY_SETUP.md`, `TROUBLESHOOTING.md`, `TESTING.md`, `IMPLEMENTATION_SUMMARY.md`; CHANGELOG entry.
- `AGENTS.md` updated: architecture → Rust; verification block → `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cov --help`, `cov embed --help`, embed-on-copy rule kept. **Safety rules unchanged**: interactive cover selection only, no COV internal API, visible attribution + full address, wrappers delegate.

## 12. Testing

- **Golden contract tests**: exact COVIT argv, exact AppleScript/nohup payload, forced-quote algorithm, attribution strings, embed callback with `@covit_path@`.
- **Unit**: `paths.rs` fixture tree (sorting, `._*`, case-insensitivity), config parse/precedence, theme parse, artwork-state classification incl. `Partial`, epoch stale-drop logic, filter cycling.
- **Integration**: embed round-trip per format on temp copies (picture present, unrelated tags preserved, dry-run untouched, summary counts, exit codes); `assert_cmd` CLI tests (help/errors/doctor).
- **TUI**: ratatui `TestBackend` layout tests per screen; state-machine tests.
- **Bench**: `benches/matcher.rs` — nucleo vs frizbee on 5k-album synthetic corpus; decides D4.
- **Manual matrix** (recorded in `docs/TESTING.md`): real copied album end-to-end, every audio format, Ghostty (Kitty) vs Terminal.app (halfblock).

## 13. Build Order

1. Spikes: tagging matrix (→ tests), matcher bench.
2. Core: `paths`, `config`, `launcher`, `embed`, `macos`.
3. CLI commands — parity checkpoint vs old toolkit.
4. TUI: scanner/matcher → finder → preview/artwork worker → form/log/doctor → themes.
5. install.sh, integrations, docs, AGENTS.md.
6. Full verification pass + manual matrix.

## 14. v2 Candidates (Out of Scope)

Persistent index cache + watch mode; batch queue; multi-disc collapse with recursive embed; prebuilt binaries with checksums; persistent frecency history; log rotation.

## 15. Safety Invariants (Carried Over)

- Never automate cover selection or use COV's internal API; selection is always interactive in the browser banner.
- Preserve the visible COV attribution and the full `https://covers.musichoarders.xyz/` address.
- Embedding changes audio tags: verify on copied files before expanding format support.
