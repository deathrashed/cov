# COV Integration Toolkit Visual README Revamp Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform `README.md` into a visual document following Archetype 4 (Go CLI style with Cyan `%2301acd7` accents and dark `#1e1e1e` badges).

**Architecture:** Replace existing text-heavy `README.md` with visual hero block, Iconify SVG section headers, formatted command tables, visual ASCII tree, and collapsible details blocks for app integrations and embedding rules.

**Tech Stack:** GitHub Flavored Markdown, Shields.io badges, Iconify API SVG graphics.

---

### Task 1: Draft the Visual README Content

**Files:**
- Modify: `README.md:1-178`

- [ ] **Step 1: Replace README.md with Archetype 4 Visual Layout**

Write the visual `README.md` content matching all requirements from the design spec:

```markdown
<div align="center">
  <img src="assets/icon.png" alt="COV Integration Toolkit Logo" width="128">

  <h1>COV INTEGRATION TOOLKIT</h1>

  <p><strong>A reusable macOS toolkit for opening official COV cover search and embedding high-res artwork directly into audio files.</strong></p>

  <p>
    <a href="https://apple.com/macos"><img src="https://img.shields.io/badge/macOS-1e1e1e?style=for-the-badge&logo=apple&logoColor=01acd7" alt="macOS"></a>
    <a href="https://python.org"><img src="https://img.shields.io/badge/python-3.12-1e1e1e?style=for-the-badge&logo=python&logoColor=01acd7" alt="Python 3.12"></a>
    <a href="https://covers.musichoarders.xyz/"><img src="https://img.shields.io/badge/executable-COVIT-1e1e1e?style=for-the-badge&logo=codecrafters&logoColor=01acd7" alt="COVIT"></a>
    <a href="https://github.com/beetbox/mutagen"><img src="https://img.shields.io/badge/tagging-mutagen-1e1e1e?style=for-the-badge&logo=pypi&logoColor=01acd7" alt="Mutagen"></a>
  </p>

  <p>
    <a href="#-quick-start">Quick Start</a> |
    <a href="#-commands">Commands</a> |
    <a href="#-repository-structure">Structure</a> |
    <a href="#-integrations">Integrations</a> |
    <a href="#-embedding--audio-support">Embedding</a> |
    <a href="#-documentation--logs">Docs & Logs</a>
  </p>
</div>

---

## <img src="https://api.iconify.design/mdi:rocket-launch-outline.svg?color=%2301acd7" height="22"> Quick Start

The toolkit uses the official [COVIT process launcher](https://covers.musichoarders.xyz/) at `/Users/rd/.local/bin/covit`. It never calls COV's internal API directly and always leaves final cover selection to the interactive browser banner.

1. **Install stable symlinks into `~/.local/bin`:**

   ```bash
   cd /Users/rd/Scripts/Riley/audio/cov
   ./install.sh
   rehash
   ```

2. **Verify system dependencies and tools:**

   ```bash
   cov-doctor
   ```

> [!NOTE]
> Requirements: Python 3.12 (`/opt/homebrew/opt/python@3.12/libexec/bin/python3`), `mutagen`, `textual`, `/Users/rd/.local/bin/covit`, and a default macOS browser.

---

## <img src="https://api.iconify.design/mdi:terminal.svg?color=%2301acd7" height="22"> Commands

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

## <img src="https://api.iconify.design/mdi:folder-tree.svg?color=%2301acd7" height="22"> Repository Structure

```text
/Users/rd/Scripts/Riley/audio/cov/
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

## <img src="https://api.iconify.design/mdi:puzzle-outline.svg?color=%2301acd7" height="22"> Integrations

<details>
<summary><strong>⌘ Swinsian Integration</strong></summary>

Standalone AppleScripts:
- [swinsian-cov.applescript](integrations/swinsian-cov.applescript)
- [swinsian-cov-embed.applescript](integrations/swinsian-cov-embed.applescript)

Both scripts inspect the currently selected track in Swinsian (falling back to the currently playing track). For Keyboard Maestro, paste the content directly into an **Execute AppleScript** action.
</details>

<details>
<summary><strong>📁 Finder & Folder Chooser</strong></summary>

Run directly from terminal or bind to Keyboard Maestro / Quick Actions:
- [finder-cov.applescript](integrations/finder-cov.applescript)
- [finder-cov-embed.applescript](integrations/finder-cov-embed.applescript)

```bash
cov-finder save
cov-choose embed
```
</details>

<details>
<summary><strong>⚡ Raycast Integration</strong></summary>

Execute the stable binary directly from Raycast extensions:

```ts
await runAppleScript(
  `
    on run argv
      do shell script quoted form of (item 1 of argv) & " " & quoted form of (item 2 of argv)
    end run
  `,
  ["/Users/rd/Scripts/Riley/audio/cov/bin/cov-open-embed", trackPath],
);
```
</details>

<details>
<summary><strong>👻 Ghostty & TUI</strong></summary>

Open the interactive terminal interface:

```bash
cov-tui
```

From outside Ghostty/Terminal, `cov-ghostty` launches a dedicated window automatically.
</details>

---

## <img src="https://api.iconify.design/mdi:music-box-multiple-outline.svg?color=%2301acd7" height="22"> Embedding & Audio Support

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

## <img src="https://api.iconify.design/mdi:book-open-page-variant-outline.svg?color=%2301acd7" height="22"> Documentation & Logs

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
```

- [ ] **Step 2: Verify Syntax & Links**

Run markdown check command:
```bash
make check
```

- [ ] **Step 3: Commit Changes**

```bash
git add README.md
git commit -m "docs: revamp README with visual Archetype 4 design"
```
