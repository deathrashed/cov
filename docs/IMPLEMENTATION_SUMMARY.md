# Implementation Summary

## Unified entry points

The official COVIT launcher remains the single integration core. Thin shell
entry points now resolve Swinsian, Finder, clipboard, automatic frontmost-app,
direct path, and Ghostty/TUI contexts. The Keyboard Maestro macro group invokes
those same commands rather than embedding duplicate AppleScript or shell logic.

## Terminal interface

`src/tui/` provides a Rust TUI (ratatui) with source and action selection,
direct-path metadata overrides, Doctor output, and log viewing. `cov ghostty`
opens it in a dedicated Ghostty window.

## Repository preparation

The directory includes Rust package metadata (Cargo.toml), a Makefile,
changelog, expanded ignore rules, integration documentation, and explicit notes about the
AMI-COV licensing boundary. Legacy Python files are preserved in `lib/` for
reference but are no longer the primary implementation.

## Working launch sequence

1. Resolve an audio file from a supplied file or album directory.
2. Run the official ARM64 COVIT binary.
3. Pass the audio file through `--input`, allowing COVIT to read metadata.
4. Use COVIT's default-browser route.
5. Pass `--primary-output cover --primary-overwrite`.
6. Start COVIT in an independent process session.
7. Keep visible attribution to `https://covers.musichoarders.xyz/`.
8. Optionally supply `--primary-command` to embed the selected artwork.

## Problems found and fixes

### JSON/URL-only integration could not receive selections

The old Swinsian action opened a pre-filled website URL, Finder, and Mp3tag. It could search but had no COV remote session, so clicks could not return artwork to a local handler.

**Fix:** pass the actual audio file to COVIT and let it establish the WebSocket integration.

### Raycast child process exited

Launching COVIT directly from Raycast caused the local socket to disappear when Raycast finished the action. COV then closed its puppet page.

**Fix:** use a shared external launcher that starts a new process session.

### `launchctl submit` reopened the browser repeatedly

The submitted job restarted when COVIT exited.

**Fix:** use a one-shot detached process, not a persistent launchd job.

### Safari received a malformed local path

COVIT's explicit Safari adapter produced a URL beginning with `file:///.../https:/`.

**Fix:** do not force Safari.

### Explicit Chrome exited too early

When Chrome was already running, its explicit adapter returned immediately. COVIT then stopped listening.

**Fix:** omit `--browsers` and use the system-default route. Chrome still opens when it is the default browser.

### COVIT could not find `open`

AppleScript and Raycast provide a restricted PATH. COVIT logged:

```text
exec: "open": executable file not found in $PATH
```

**Fix:** supply a full macOS PATH including `/usr/bin`.

### Click connected but artwork was not saved

Passing `--primary-output cover.jpg` did not work correctly with this build.

**Fix:** use the documented extensionless output `cover`; COVIT appends JPEG or PNG according to the selected image.

## Embedding

The official COVIT tool saves files but does not itself write audio tags. The toolkit attaches its own post-selection callback when `--embed` is requested. The callback uses `lofty`/`image`, replaces only the front-cover frame, leaves other tags intact, and then asks Swinsian to rescan updated tracks.

## Verification completed

- Rust compilation for the full crate.
- Rust unit and integration tests pass (embedding and launcher contract).
- Shell syntax validation for every launcher and installer.
- AppleScript compilation for all Swinsian and Finder wrappers.
- Dry-run discovery against a real six-track album.
- Real embedding into temporary MP3, FLAC, M4A, Ogg/Opus, WAV, and AIFF fixtures.
- Replacement of the cover frame in a copied real Disrotter MP3 while preserving all 19 non-artwork ID3 frames.
- No source music file was modified during testing.
