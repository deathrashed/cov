# COV Sources and Prior Art

## Source selection

COV combines many artwork providers. Its defaults are a good general starting
point; source filtering is most useful when a release is difficult to identify
or when provenance matters.

The launcher accepts optional COVIT query overrides:

```sh
cov-open "/path/to/album" \
  --artist "Artist" \
  --album "Album" \
  --identifier "CATALOGUE-001" \
  --resolution 1500 \
  --sources "source-a,source-b"
```

The same fields are available in the TUI for direct paths.

The detailed reference used while designing these controls is documented in the Music Hoarders COV specification.

## AMI-COV

The existing AMI-COV Picard integration was reviewed for its useful interaction
model: derive metadata from the selected album, start a local WebSocket bridge,
open COV with populated query parameters, then receive the selected image.

This toolkit deliberately uses the official COVIT executable instead. It gives
Swinsian, Finder, Ghostty, Raycast, and Keyboard Maestro the same interaction
without maintaining a second WebSocket implementation.

AMI-COV is AGPL-3.0 software. No AMI-COV source code has been copied into this
toolkit. If its client/server implementation is incorporated later, the new
repository's licence and distribution obligations must be decided first.
