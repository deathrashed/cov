# Testing Record

## 2026-07-24 — Rust rewrite verification

- `cargo check`: passes with no errors.
- `cargo test`: all Rust tests pass (embedding matrix, launch contract, debug).
- Shell syntax validation for every launcher and installer (`zsh -n`).
- `cov doctor`: passes COVIT, macOS open, AppleScript, Ghostty checks.
- `cov --help`: confirmed all subcommands listed.
- AppleScript integrations compile with `osacompile`.

Keyboard Maestro import and live hotkey execution were not performed
automatically. The macro group has no default hotkeys and is safe to import for
interactive testing.

Date: 2026-07-24

## Non-writing checks

- Rust crate compiles cleanly (`cargo check`).
- All Rust unit and integration tests pass.
- Shell syntax validation passed for all launchers and the installer.
- All AppleScript integrations compile successfully with `osacompile`.
- Album-directory resolution selects the first real audio file directly inside the album.
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
| APE | APEv2 Cover Art | Pass |
| WavPack | APEv2 Cover Art | Pass |

## Real-file copy test

A Disrotter MP3 was copied to a temporary directory before testing.

- Front-cover frames before: 1
- Non-artwork ID3 frames before: 19
- Front-cover frames after: 1
- Non-artwork ID3 frames after: 19
- Original source file: untouched

This confirms that the MP3 implementation replaces the front cover without discarding unrelated metadata.

## Limits

- COV selection remains deliberately manual.
- Long-lived browser-session persistence must be confirmed from Raycast, Keyboard Maestro, Finder, or Ghostty itself. The launcher uses the same AppleScript plus one-shot `nohup` pattern that succeeded in the live Raycast integration.
