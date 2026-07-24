# Testing Record

## 2026-07-24 — toolkit expansion

- `make check`: passed zsh syntax, Python byte-compilation, and plist lint.
- Keyboard Maestro validator: passed plist and semantic checks for
  `integrations/keyboard-maestro/COV Toolkit.kmmacros`.
- Textual headless test at 120 × 40: rendered, selected default context and
  mode, clicked Doctor, observed the ready result, and exercised the Quit
  binding.
- `cov-doctor`: passed COVIT, Python 3.12, Ghostty, Mutagen, Textual,
  AppleScript, and every installed command.
- All standalone AppleScript integrations compiled with `osacompile`.
- `cov-open --help`: confirmed the artist, album, identifier, country,
  resolution, and source override interface.
- `git diff --check`: passed.

Keyboard Maestro import and live hotkey execution were not performed
automatically. The macro group has no default hotkeys and is safe to import for
interactive testing.

Date: 2026-07-24

## Non-writing checks

- All zsh launchers and the installer passed `zsh -n`.
- Both Python modules compiled successfully.
- All four AppleScript integrations compiled successfully with `osacompile`.
- `cov-doctor` passed every installed requirement.
- Album-directory resolution selected the first real audio file directly inside the album.
- A dry run discovered all six tracks in Disrotter's `Perish Forth` album.

## Temporary embedding matrix

Short synthetic audio files were generated in a temporary directory. No library file was used as a write target.

| Format | Tag representation | Result |
| --- | --- | --- |
| MP3 | ID3 APIC | Pass |
| FLAC | Native picture block | Pass |
| M4A | MP4 `covr` | Pass |
| Ogg/Opus | `METADATA_BLOCK_PICTURE` | Pass |
| Opus | `METADATA_BLOCK_PICTURE` | Pass |
| WAV | ID3 APIC | Pass |
| AIFF | ID3 APIC | Pass |

## Real-file copy test

A Disrotter MP3 was copied to a temporary directory before testing.

- Front-cover frames before: 1
- Non-artwork ID3 frames before: 19
- Front-cover frames after: 1
- Non-artwork ID3 frames after: 19
- Original source file: untouched

This confirms that the MP3 implementation replaces the front cover without discarding unrelated metadata.

## Limits

- APE, WavPack, and DSF support follows Mutagen's documented tag models but no local fixture was available for a live write test.
- COV selection remains deliberately manual.
- The Codex execution supervisor reaps child processes after tool calls, so long-lived browser-session persistence must be confirmed from Raycast, Keyboard Maestro, Finder, or Ghostty itself. The launcher uses the same AppleScript plus one-shot `nohup` pattern that succeeded in the live Raycast integration.
