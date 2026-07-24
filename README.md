<div align="center">
  <img src="assets/icon.png" alt="COV Integration Toolkit Logo" width="128">

  <h1>COV INTEGRATION TOOLKIT</h1>

  <p><strong>A reusable macOS toolkit for opening official COV cover search and embedding high-res artwork directly into audio files.</strong></p>

  <p>
    <a href="https://apple.com/macos"><img src="https://img.shields.io/badge/macOS-1e1e1e?style=for-the-badge&logo=apple&logoColor=01acd7" alt="macOS"></a>
    <a href="https://python.org"><img src="https://img.shields.io/badge/python-3.12-1e1e1e?style=for-the-badge&logo=python&logoColor=01acd7" alt="Python 3.12"></a>
    <a href="https://covers.musichoarders.xyz/"><img src="https://img.shields.io/badge/executable-COVIT-1e1e1e?style=for-the-badge&logo=codecrafters&logoColor=01acd7" alt="COVIT"></a>
    <a href="https://github.com/beetbox/mutagen"><img src="https://img.shields.io/badge/tagging-mutagen-1e1e1e?style=for-the-badge&logo=pypi&logoColor=01acd7" alt="Mutagen"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-WTFPL-1e1e1e?style=for-the-badge&logo=opensourceinitiative&logoColor=01acd7" alt="License: WTFPL"></a>
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

1. **Install stable symlinks into `~/.local/bin`:**

   ```bash
   ./install.sh
   rehash
   ```

2. **Verify system dependencies and tools:**

   ```bash
   cov-doctor
   ```

> [!NOTE]
> Requirements: Python 3.12 (`/opt/homebrew/opt/python@3.12/libexec/bin/python3`), `mutagen`, `textual`, `~/.local/bin/covit`, and a default macOS browser.

---

## <a id="commands"></a><img src="https://api.iconify.design/mdi:terminal.svg?color=%2301acd7" height="22"> Commands

| Command | Icon | Purpose | Primary Target |
| --- | --- | --- | --- |
| `bin/cov-open PATH` | <img src="https://api.iconify.design/mdi:folder-search-outline.svg?color=%2301acd7" height="18"> | Open COV & save selected cover beside album | Audio file or Album folder |
| `bin/cov-open-embed PATH` | <img src="https://api.iconify.design/mdi:folder-music-outline.svg?color=%2301acd7" height="18"> | Save cover & embed into all album tracks | Audio file or Album folder |
| `bin/cov-embed ARTWORK PATH` | <img src="https://api.iconify.design/mdi:image-edit-outline.svg?color=%2301acd7" height="18"> | Embed existing local artwork without opening browser | Image file & Album target |
| `bin/cov-context [save\|embed]` | <img src="https://api.iconify.design/mdi:auto-fix.svg?color=%2301acd7" height="18"> | Auto-detect path from Swinsian, Finder, or Clipboard | Active environment |
| `bin/cov-swinsian [save\|embed]` | <img src="https://api.iconify.design/mdi:music-box-outline.svg?color=%2301acd7" height="18"> | Fetch metadata from selected/playing Swinsian track | Swinsian player |
| `bin/cov-finder [save\|embed]` | <img src="https://api.iconify.design/mdi:apple-finder.svg?color=%2301acd7" height="18"> | Process currently selected item in Finder | Finder selection |
| `bin/cov-choose [save\|embed]` | <img src="https://api.iconify.design/mdi:folder-account-outline.svg?color=%2301acd7" height="18"> | Prompt with native macOS directory chooser | User selection |
| `bin/cov-clipboard [save\|embed]` | <img src="https://api.iconify.design/mdi:clipboard-text-outline.svg?color=%2301acd7" height="18"> | Process folder or file path copied to clipboard | Clipboard text |
| `bin/cov-tui` | <img src="https://api.iconify.design/mdi:console.svg?color=%2301acd7" height="18"> | Launch guided Textual TUI interface | Terminal |
| `bin/cov-ghostty` | <img src="https://api.iconify.design/mdi:ghost.svg?color=%2301acd7" height="18"> | Open TUI in a dedicated Ghostty window | External GUI launcher |
| `bin/cov-log [show\|follow]` | <img src="https://api.iconify.design/mdi:file-document-outline.svg?color=%2301acd7" height="18"> | View or tail live toolkit logs | Debugging |
| `bin/cov-doctor` | <img src="https://api.iconify.design/mdi:stethoscope.svg?color=%2301acd7" height="18"> | Check binary, library, and tool availability | Diagnostic |

---

## <a id="repository-structure"></a><img src="https://api.iconify.design/mdi:folder-tree.svg?color=%2301acd7" height="22"> Repository Structure

```text
cov/
├── AGENTS.md                  # Development & architectural safety rules
├── CHANGELOG.md               # Revision history
├── Makefile                   # Validation & maintenance tasks
├── README.md                  # Canonical documentation
├── install.sh                 # Safe symlink installer for ~/.local/bin
├── pyproject.toml             # Python metadata & requirements
├── assets/                    # Project icons & branding assets
│   └── icon.png
├── bin/                       # Stable executable command line entry points
│   ├── cov-open               # Main open & save launcher
│   ├── cov-open-embed         # Open, save, and embed launcher
│   ├── cov-embed              # Standalone Mutagen tag embedder
│   ├── cov-context            # Environment auto-detector
│   ├── cov-tui                # Textual TUI entrypoint
│   └── cov-doctor             # System environment diagnostics
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
│   └── swinsian-cov-embed.applescript
└── lib/                       # Core python execution libraries
    ├── cov_launcher.py        # COVIT launcher & process manager
    └── embed_art.py           # Native audio artwork embedding engine
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
cov-finder save
cov-choose embed
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
  ["/path/to/cov/bin/cov-open-embed", trackPath],
);
```
</details>

<details>
<summary><strong><img src="https://api.iconify.design/mdi:console.svg?color=%2301acd7" height="16" valign="middle"> Ghostty & TUI</strong></summary>

Open the interactive terminal interface:

```bash
cov-tui
```

From outside Ghostty/Terminal, `cov-ghostty` launches a dedicated window automatically.
</details>

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
> cov-embed "/path/to/cover.jpg" "/path/to/album" --dry-run
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
cov-log follow
```
