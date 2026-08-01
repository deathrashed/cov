<div align="center">
  <img src="assets/icon.png" alt="COV Integration Toolkit Logo" width="128">

  <h1>COV INTEGRATION TOOLKIT</h1>

  <p><strong>A reusable macOS toolkit for opening official COV cover search and embedding high-res artwork directly into audio files.</strong></p>

  <p>
    <a href="https://apple.com/macos"><img src="https://img.shields.io/badge/macOS-1e1e1e?style=for-the-badge&logo=apple&logoColor=01acd7" alt="macOS"></a>
    <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/rust-2024-1e1e1e?style=for-the-badge&logo=rust&logoColor=01acd7" alt="Rust 2024"></a>
    <a href="https://covers.musichoarders.xyz/"><img src="https://img.shields.io/badge/executable-COVIT-1e1e1e?style=for-the-badge&logo=codecrafters&logoColor=01acd7" alt="COVIT"></a>
    <a href="https://github.com/image-rs/image"><img src="https://img.shields.io/badge/embedding-image-1e1e1e?style=for-the-badge&logo=rust&logoColor=01acd7" alt="image"></a>
  </p>

  <p>
    <a href="#quick-start">Quick Start</a> |
    <a href="#commands">Commands</a> |
    <a href="#repository-structure">Structure</a> |
    <a href="#integrations">Integrations</a> |
    <a href="#embedding--audio-support">Embedding</a> |
    <a href="#documentation--logs">Docs & Logs</a>
  </p>
</div>

---

## <a id="quick-start"></a><img src="https://api.iconify.design/mdi:rocket-launch-outline.svg?color=%2301acd7" height="22"> Quick Start

The toolkit uses the official [COVIT process launcher](https://covers.musichoarders.xyz/) (typically installed at `~/.local/bin/covit`). It never calls COV's internal API directly and always leaves final cover selection to the interactive browser banner.

1. **Install the Rust binary into `~/.local/bin`:**

    ```bash
    ./install.sh
    rehash
    ```

2. **Verify system dependencies and tools:**

    ```bash
    cov doctor
    ```

> [!NOTE]
> Requirements: Rust toolchain (`cargo`), `~/.local/bin/covit`, and a default macOS browser.

---

## <a id="commands"></a><img src="https://api.iconify.design/mdi:terminal.svg?color=%2301acd7" height="22"> Commands

| Command |  | Purpose | Primary Target |
| --- | --- | --- | --- |
| `cov open PATH` | <img src="https://api.iconify.design/mdi:folder-search-outline.svg?color=%2301acd7" height="18"> | Open COV & save selected cover beside album | Audio file or Album folder |
| `cov open --embed PATH` | <img src="https://api.iconify.design/mdi:folder-music-outline.svg?color=%2301acd7" height="18"> | Save cover & embed into all album tracks | Audio file or Album folder |
| `cov embed ARTWORK PATH` | <img src="https://api.iconify.design/mdi:image-edit-outline.svg?color=%2301acd7" height="18"> | Embed existing local artwork without opening browser | Image file & Album target |
| `cov context [save\|embed]` | <img src="https://api.iconify.design/mdi:auto-fix.svg?color=%2301acd7" height="18"> | Auto-detect path from Swinsian, Finder, or Clipboard | Active environment |
| `cov swinsian [save\|embed]` | <img src="https://api.iconify.design/mdi:music-box-outline.svg?color=%2301acd7" height="18"> | Fetch metadata from selected/playing Swinsian track | Swinsian player |
| `cov finder [save\|embed]` | <img src="https://api.iconify.design/mdi:apple-finder.svg?color=%2301acd7" height="18"> | Process currently selected item in Finder | Finder selection |
| `cov choose [save\|embed]` | <img src="https://api.iconify.design/mdi:folder-account-outline.svg?color=%2301acd7" height="18"> | Prompt with native macOS directory chooser | User selection |
| `cov clipboard [save\|embed]` | <img src="https://api.iconify.design/mdi:clipboard-text-outline.svg?color=%2301acd7" height="18"> | Process folder or file path copied to clipboard | Clipboard text |
| `cov tui` | <img src="https://api.iconify.design/mdi:console.svg?color=%2301acd7" height="18"> | Launch the cached album picker with artwork filters | Terminal |
| `cov-fzf [query]` | <img src="https://api.iconify.design/mdi:format-list-bulleted.svg?color=%2301acd7" height="18"> | Fast msearch-style folder picker | Terminal |
| `cov scan missing-sidecar [ROOT]` | <img src="https://api.iconify.design/mdi:image-off-outline.svg?color=%2301acd7" height="18"> | Print album folders with no `cover` sidecar | Terminal |
| `cov scan missing-embedded [ROOT]` | <img src="https://api.iconify.design/mdi:music-note-off-outline.svg?color=%2301acd7" height="18"> | Print tracks with no embedded front artwork | Terminal |
| `cov ghostty` | <img src="https://api.iconify.design/mdi:ghost.svg?color=%2301acd7" height="18"> | Open TUI in a dedicated Ghostty window | External GUI launcher |
| `cov log [show\|follow]` | <img src="https://api.iconify.design/mdi:file-document-outline.svg?color=%2301acd7" height="18"> | View or tail live toolkit logs | Debugging |
| `cov doctor` | <img src="https://api.iconify.design/mdi:stethoscope.svg?color=%2301acd7" height="18"> | Check binary, library, and tool availability | Diagnostic |

---

## <a id="repository-structure"></a><img src="https://api.iconify.design/mdi:folder-tree.svg?color=%2301acd7" height="22"> Repository Structure

```text
cov/
├── AGENTS.md                  # Development & architectural safety rules
├── CHANGELOG.md               # Revision history
├── Makefile                   # Validation & maintenance tasks
├── README.md                  # Canonical documentation
├── install.sh                 # Cargo build + symlink installer for ~/.local/bin
├── Cargo.toml                 # Rust package manifest
├── Cargo.lock                 # Locked dependency graph
├── pyproject.toml             # Legacy Python metadata (kept for reference)
├── assets/                    # Project icons & branding assets
│   └── icon.png
├── bin/                       # Stable executable command line entry points
│   ├── cov-open               # -> cov open
│   ├── cov-open-embed         # -> cov open --embed
│   ├── cov-embed              # -> cov embed
│   ├── cov-context            # -> cov context
│   ├── cov-tui                # -> cov tui
│   └── cov-doctor             # -> cov doctor
├── docs/                      # Technical documentation & plans
│   ├── IMPLEMENTATION_SUMMARY.md
│   ├── INTEGRATIONS.md
│   ├── REPOSITORY_SETUP.md
│   ├── SOURCES_AND_PRIOR_ART.md
│   ├── TESTING.md
│   └── TROUBLESHOOTING.md
├── integrations/              # Host application AppleScript wrappers
│   ├── finder-cov.applescript
│   ├── finder-cov-embed.applescript
│   ├── swinsian-cov.applescript
│   ├── swinsian-cov-embed.applescript
│   └── keyboard-maestro/
│       └── COV Toolkit.kmmacros
├── lib/                       # Legacy Python core libraries (deprecated)
│   ├── cov_launcher.py
│   ├── cov_tui.py
│   └── embed_art.py
├── src/                       # Rust source
│   ├── main.rs
│   ├── lib.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── context.rs
│   ├── doctor.rs
│   ├── embed.rs
│   ├── launcher.rs
│   ├── macos.rs
│   ├── paths.rs
│   ├── testutil.rs
│   └── tui/
│       ├── mod.rs
│       ├── app.rs
│       ├── theme.rs
│       ├── scanner.rs
│       ├── matcher.rs
│       ├── artwork.rs
│       ├── images.rs
│       ├── screens/
│       └── widgets/
├── tests/                     # Rust integration tests
│   ├── fixtures/
│   ├── embed_matrix.rs
│   ├── launch_contract.rs
│   ├── debug_ape.rs
│   └── debug_wv_write.rs
└── themes/                    # TUI theme files
    └── default.toml
```

---

## <a id="integrations"></a><img src="https://api.iconify.design/mdi:puzzle-outline.svg?color=%2301acd7" height="22"> Integrations

<details>
<summary><strong><img src="https://api.iconify.design/mdi:music-box-outline.svg?color=%2301acd7" height="16" valign="middle"> Swinsian Integration</strong></summary>

Standalone AppleScripts:
- [swinsian-cov.applescript](integrations/swinsian-cov.applescript)
- [swinsian-cov-embed.applescript](integrations/swinsian-cov-embed.applescript)

Both scripts inspect the currently selected track in Swinsian (falling back to the currently playing track). For Keyboard Maestro, paste the content directly into an **Execute AppleScript** action.
</details>

<details>
<summary><strong><img src="https://api.iconify.design/mdi:folder-outline.svg?color=%2301acd7" height="16" valign="middle"> Finder & Folder Chooser</strong></summary>

Run directly from terminal or bind to Keyboard Maestro / Quick Actions:
- [finder-cov.applescript](integrations/finder-cov.applescript)
- [finder-cov-embed.applescript](integrations/finder-cov-embed.applescript)

```bash
cov finder save
cov choose embed
```
</details>

<details>
<summary><strong><img src="https://api.iconify.design/mdi:lightning-bolt-outline.svg?color=%2301acd7" height="16" valign="middle"> Raycast Integration</strong></summary>

Execute the stable binary directly from Raycast extensions:

```ts
await runAppleScript(
  `
    on run argv
      do shell script quoted form of (item 1 of argv) & " " & quoted form of (item 2 of argv)
    end run
  `,
  ["~/.local/bin/cov open --embed", trackPath],
);
```
</details>

<details>
<summary><strong><img src="https://api.iconify.design/mdi:console.svg?color=%2301acd7" height="16" valign="middle"> Ghostty & TUI</strong></summary>

Open the interactive terminal interface:

```bash
cov tui
```

From outside Ghostty/Terminal, `cov ghostty` launches a dedicated window automatically.
The picker is deliberately small: search, choose an album, press `Enter` to open COV or `Ctrl-E` to embed. Finder, Swinsian, clipboard, and folder workflows remain dedicated commands rather than picker controls.
</details>

### TUI library index

The TUI persists a compact JSON index after its first completed scan. Later launches use that index immediately; artwork status checks continue in the background. Each library receives its own cache file, so switching `library_root` never reuses another library's results.

The macOS configuration file is `~/Library/Application Support/xyz.musichoarders.cov/config.toml`:

```toml
library_root = "/path/to/music"
default_mode = "save"        # "save" or "embed" for Enter in the picker
output_basename = "cover"    # Cover filename without an extension
# default_resolution = 1500   # Optional preferred minimum artwork size
# default_sources = "bandcamp,deezer" # Optional comma-separated COV source IDs

[cache]
enabled = true
# Optional exact file location. Leave unset for COV's platform cache directory.
# path = "/path/to/cov-library-index.json"
refresh = "manual" # "manual" (default) or "startup"
```

`manual` keeps startup instant and refreshes only with `Ctrl-R`. `startup` shows the saved index first, then replaces it with a fresh background scan. The same controls are available for one launch without changing the config:

```bash
cov tui --rescan
cov tui --no-cache
cov tui --cache-path "/path/to/cov-library-index.json"
```

Press `Ctrl-O` in the picker to edit the library folder, default action, cover filename, resolution, and provider defaults without editing TOML manually. Press `Enter` in Settings to save; `Esc` discards the staged changes.

### fzf picker

`cov-fzf` is the intentionally small, msearch-style interface. It searches
album folders directly and does not calculate artwork state while you search:

```bash
cov-fzf "slayer hell awaits"
cov-fzf --library "/path/to/music" "slayer"
```

It uses `COV_LIBRARY_ROOT`, then `MUSIC_DIR`, then the configured
`library_root`. `COV_FZF_PREVIEW_MODE` accepts `top` (default), `right`, or
`off`.

| Key | Action |
| --- | --- |
| `Enter` | Open selected folder in Finder |
| `Ctrl-O` / `Ctrl-R` | Open / reveal selected folder in Finder |
| `Ctrl-Y` | Copy folder path |
| `Ctrl-G` | Open COV for the selected folder |
| `Ctrl-E` | Open COV and embed the selected cover |

The maintenance scans are separate, streaming commands rather than picker
filters, so normal search stays instant. They print one path per line and can
be piped into fzf when needed:

```bash
cov scan missing-sidecar | fzf
cov scan missing-embedded | fzf
```

`missing-sidecar` recognizes `cover.jpg`, `cover.jpeg`, and `cover.png`.
`missing-embedded` checks each supported audio file for a front (or MP4-style
`Other`) picture. Both traverse the library live, so they are deliberately
explicit maintenance commands rather than startup work.

---

## <a id="embedding--audio-support"></a><img src="https://api.iconify.design/mdi:music-box-multiple-outline.svg?color=%2301acd7" height="22"> Embedding & Audio Support

The toolkit replaces front cover frames while strictly preserving all existing metadata tag fields.

| Format | Extension | Container / Tag Frame |
| --- | --- | --- |
| **MP3** | `.mp3` | ID3v2 `APIC` (Front Cover) |
| **FLAC** | `.flac` | Native FLAC Picture Block |
| **M4A / MP4** | `.m4a`, `.mp4` | MP4 Atom `covr` |
| **Ogg / Opus** | `.ogg`, `.opus` | Vorbis Comment `METADATA_BLOCK_PICTURE` |
| **WAV / AIFF** | `.wav`, `.aiff` | ID3 `APIC` |
| **APE / WavPack**| `.ape`, `.wv` | APEv2 `Cover Art (Front)` |

> [!TIP]
> **Dry Run Verification:** Test image embedding safely without editing tags:
> ```bash
> cov embed "/path/to/cover.jpg" "/path/to/album" --dry-run
> ```

---

## <a id="documentation--logs"></a><img src="https://api.iconify.design/mdi:book-open-page-variant-outline.svg?color=%2301acd7" height="22"> Documentation & Logs

- [Integration Recipes](docs/INTEGRATIONS.md)
- [COV Sources & AMI-COV Notes](docs/SOURCES_AND_PRIOR_ART.md)
- [Repository Setup](docs/REPOSITORY_SETUP.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Implementation Summary](docs/IMPLEMENTATION_SUMMARY.md)
- [Testing Record](docs/TESTING.md)

**Log Files:**
Live launcher output is stored at `~/Library/Logs/cov-toolkit.log`. View logs with:
```bash
cov log follow
```
