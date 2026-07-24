# Troubleshooting

## Chrome opens with no page

Check:

```sh
tail -100 ~/Library/Logs/cov-toolkit.log
```

If COVIT reports that `open` is missing, ensure the launcher supplies:

```text
/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin
```

The shared launcher already does this.

## The page opens and immediately closes

COV puppet mode closes when its local WebSocket disconnects. Do not run COVIT as an ordinary Raycast child process.

The shared launcher starts a new process session so COVIT survives after its caller exits.

## Safari shows a `file://` URL

Do not force COVIT's Safari adapter. This COVIT build can hand Safari an HTTPS URL as a local file path.

## Explicit Chrome opens a blank window

Do not force COVIT's Chrome adapter when Chrome is already running. The adapter's helper process can exit immediately, causing COVIT to close its socket.

Use the default-browser route. If Chrome is the macOS default, COV still opens in Chrome.

## Browser windows repeatedly reopen

Do not use `launchctl submit` for this COVIT build. Submitted jobs can restart after COVIT exits.

Inspect and remove stale jobs:

```sh
launchctl list | grep com.deathrashed.swinsian.covit
launchctl remove EXACT_JOB_LABEL
```

## Integration connects but clicking does not save

Use:

```text
--primary-output cover
```

Do not force `cover.jpg`. COVIT appends the selected image's real extension.

## Embedding fails

Run the callback directly:

```sh
cov-embed "/path/to/cover.jpg" "/path/to/album"
```

Check:

- The cover is JPEG or PNG.
- The files are writable.
- The format is listed in README.
- Mutagen is installed for Python 3.12.

Use `--dry-run` to verify target discovery without writing tags.
