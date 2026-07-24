.PHONY: build test lint fmt verify install tui bench fixtures

build:
	cargo build --release
	/bin/cp -f target/release/cov bin/cov

test:
	cargo test

lint:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --check

verify: fmt lint test

install:
	./install.sh

tui:
	cargo run --release -- tui

bench:
	cargo bench --bench matcher

fixtures:
	zsh tests/fixtures/generate.sh
