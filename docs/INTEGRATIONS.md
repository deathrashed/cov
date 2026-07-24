# Integration Recipes

## Keyboard Maestro

Import `integrations/keyboard-maestro/COV Toolkit.kmmacros`. It provides five
macros with no default hotkeys, so importing it cannot overwrite your existing
shortcuts:

- COV — Automatic Context
- COV — Swinsian (Save)
- COV — Swinsian (Save & Embed)
- COV — Finder (Save & Embed)
- COV — Open Toolkit in Ghostty

Each macro only calls a stable toolkit command. Editing or fixing the core
scripts therefore updates every integration at once.

### Swinsian selection

1. Create a macro available while Swinsian is active.
2. Add **Execute AppleScript**.
3. Paste `integrations/swinsian-cov.applescript` to save a sidecar cover.
4. Use `integrations/swinsian-cov-embed.applescript` for save-and-embed.
5. Assign a hotkey.

### Finder selection

Use the same process with `integrations/finder-cov.applescript` or `integrations/finder-cov-embed.applescript`.

## Finder Quick Action

1. Open Automator and create a **Quick Action**.
2. Set “Workflow receives current” to **files or folders** in **Finder**.
3. Add **Run Shell Script**.
4. Choose “Pass input: as arguments”.
5. Use:

```sh
cov-open "$1"
```

For embedding:

```sh
cov-open-embed "$1"
```

The action then appears in Finder's Quick Actions and Services menus.

## Ghostty

After running `install.sh`, the commands are globally available:

```sh
cov-open "/path/to/audio-or-album"
cov-open-embed "/path/to/audio-or-album"
cov-finder embed
cov-choose save
cov-choose embed
cov-clipboard save
cov-context save
cov-tui
```

Useful zsh aliases:

```sh
alias cov='cov-open'
alias cove='cov-open-embed'
alias covui='cov-tui'
```

## Raycast

Resolve the Swinsian or Finder path in Raycast, then invoke the shared script through AppleScript `do shell script`. This keeps COVIT outside Raycast's child-process cleanup and avoids reproducing its environment bugs.

Do not pass `--browsers safari` or `--browsers chrome`. Let COVIT use the default-browser route.

## Other applications

Any application capable of passing a file path can call:

```text
cov-open "<path>"
```

or:

```text
cov-open-embed "<path>"
```

The application should wait only for the launcher script. The COVIT session continues independently.
