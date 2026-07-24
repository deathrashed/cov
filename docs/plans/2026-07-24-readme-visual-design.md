# Design Specification: COV Integration Toolkit Visual README Revamp

**Date**: 2026-07-24  
**Archetype**: Archetype 4 (Go CLI Utility Style - Vibrant Cyan `%2301acd7`)  
**Target File**: `README.md` in `/Users/rd/Scripts/Riley/audio/cov`

---

## 1. Overview & Objectives

Transform the `README.md` of the COV Integration Toolkit into a high-impact, visual project document following **Archetype 4 (Go CLI Utility style)**.

Key goals:
- Elevate visual polish using centered hero layout, dark `#1e1e1e` badges with Cyan logo accents, and URL-encoded Iconify SVG headers (`%2301acd7`).
- Structured command table with embedded Iconify category icons.
- Visual ASCII tree with inline descriptions of directory/file roles.
- Collapsible reference blocks (`<details><summary>`) for integration setup (Swinsian, Finder, Raycast, Ghostty/TUI) and advanced audio embedding behavior.
- Use native GitHub alerts (`> [!NOTE]`, `> [!TIP]`, `> [!WARNING]`).

---

## 2. Document Structure & Layout

1. **Centered Hero Block (`<div align="center">`)**:
   - Logo: `assets/icon.png` (centered, styled width)
   - Title: `<h1>COV INTEGRATION TOOLKIT</h1>`
   - Bold summary statement on cover fetching and local tag embedding on macOS.
   - Shields row (macOS, Python 3.12, COVIT executable, Mutagen, Textual).
   - Pipe navigation bar (`[Quick Start](#quick-start) | [Commands](#commands) | [Integrations](#integrations) | [Embedding](#embedding-behavior) | [Architecture](#architecture)`).

2. **Iconify Section Headers (`color=%2301acd7`)**:
   - `## <img src="https://api.iconify.design/mdi:rocket-launch-outline.svg?color=%2301acd7" height="22"> Quick Start`
   - `## <img src="https://api.iconify.design/mdi:terminal.svg?color=%2301acd7" height="22"> Commands`
   - `## <img src="https://api.iconify.design/mdi:folder-tree.svg?color=%2301acd7" height="22"> Repository Structure`
   - `## <img src="https://api.iconify.design/mdi:puzzle-outline.svg?color=%2301acd7" height="22"> Integrations`
   - `## <img src="https://api.iconify.design/mdi:music-box-multiple-outline.svg?color=%2301acd7" height="22"> Embedding & Audio Support`
   - `## <img src="https://api.iconify.design/mdi:book-open-page-variant-outline.svg?color=%2301acd7" height="22"> Documentation & Logs`

3. **Command Grid**:
   - Multi-column table with Iconify category icons for `bin/cov-*` commands.

4. **ASCII Directory Tree**:
   - Annotated tree showing `bin/`, `lib/`, `integrations/`, `docs/`, `assets/`, `install.sh`, `Makefile`, `pyproject.toml`.

5. **Collapsible Details Blocks**:
   - App integrations (Swinsian AppleScript, Finder Quick Actions, Raycast TS snippet, Ghostty/TUI).
   - Format support matrix & embedding frame details.

---

## 3. Verification & Compliance Checklist

- [x] Follows Archetype 4 design rules (`%2301acd7` cyan headers & dark `#1e1e1e` badges).
- [x] Preserves all technical safety rules and attribution notices from `AGENTS.md`.
- [x] Includes clean, clickable markdown links for relative docs.
