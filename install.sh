#!/bin/zsh
set -eu

SCRIPT_DIR=${0:A:h}
BIN_DIR=${HOME}/.local/bin
command -v cargo >/dev/null 2>&1 || { print -u2 "Rust toolchain required (https://rustup.rs)"; exit 1; }

print "Building Rust binary..."
(cd "$SCRIPT_DIR" && cargo build --release)

/bin/mkdir -p "$BIN_DIR"
target="$SCRIPT_DIR/target/release/cov"
link="$BIN_DIR/cov"
if [[ -e "$link" && ! -L "$link" ]]; then
  print -u2 "Refusing to replace a real file: $link"
  exit 1
fi
/bin/ln -sfn "$target" "$link"
print "Installed: $link -> $target"

print
print "COV toolkit installed. Restart the shell or run: rehash"
