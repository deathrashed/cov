#!/bin/zsh
set -eu

SCRIPT_DIR=${0:A:h}
BIN_DIR=${HOME}/.local/bin
/bin/mkdir -p "$BIN_DIR"

for command_name in cov-open cov-open-embed cov-embed cov-finder cov-choose cov-swinsian cov-clipboard cov-context cov-tui cov-ghostty cov-log cov-doctor; do
  target="$SCRIPT_DIR/bin/$command_name"
  link="$BIN_DIR/$command_name"
  if [[ -e "$link" && ! -L "$link" ]]; then
    print -u2 "Refusing to replace a real file: $link"
    exit 1
  fi
  /bin/ln -sfn "$target" "$link"
  print "Installed: $link -> $target"
done

print
print "COV toolkit installed. Restart the shell or run: rehash"
