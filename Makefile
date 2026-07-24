.PHONY: check doctor install tui

check:
	zsh -n bin/* install.sh
	/opt/homebrew/opt/python@3.12/libexec/bin/python3 -m compileall -q lib
	plutil -lint "integrations/keyboard-maestro/COV Toolkit.kmmacros"

doctor:
	./bin/cov-doctor

install:
	./install.sh

tui:
	./bin/cov-tui
