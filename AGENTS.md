# COV Toolkit Guide

This folder is a self-contained integration toolkit for the official COV website and COVIT binary.

## Architecture

- `lib/cov_launcher.py`: canonical path resolution and COVIT process launcher.
- `lib/embed_art.py`: canonical artwork-tag embedding implementation.
- `bin/`: stable command-line entry points.
- `integrations/`: AppleScript and host-application wrappers.
- `docs/`: installation, troubleshooting, and implementation history.

Wrappers must delegate to `bin/cov-open` or `bin/cov-open-embed`; do not duplicate COVIT arguments.

## Safety

- Opening COV and saving a sidecar cover is interactive.
- Embedding changes audio tags. Test embedding on copied files before expanding format support.
- Never automate cover selection or use COV's internal API.
- Preserve the visible COV attribution and full `https://covers.musichoarders.xyz/` address.

## Verification

```sh
zsh -n bin/* install.sh
python3 -m compileall -q lib
bin/cov-open --help
bin/cov-embed --help
```

For embedding verification, copy a representative audio file into a temporary directory and run `cov-embed` against the copy.
