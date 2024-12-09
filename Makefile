.DEFAULT_GOAL = usage
default: usage

.PHONY: usage
usage:
	@echo "usage:"
	@echo "    build:     build the command"
	@echo "    install:   installs the command binary executable into $HOME/.local/bin/"
	@echo "    uninstall: removes the command binary executable from $HOME/.local/bin/s"

.PHONY: build
build:
	cargo build --release

.PHONY: install
install: uninstall build
	@cp target/release/ocd ${HOME}/.local/bin/

.PHONY: uninstall
uninstall:
	@rm -f ${HOME}/.local/bin/ocd
