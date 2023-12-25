.PHONY: usage build install uninstall

usage:
	@echo "usage:"
	@echo "    build:     build the command"
	@echo "    install:   installs the command binary executable into $HOME/.local/bin/"
	@echo "    uninstall: removes the command binary executable from $HOME/.local/bin/s"

build:
	cargo build --release

install: build uninstall
	@cp target/release/ocd ${HOME}/.local/bin/

uninstall:
	@rm -f ${HOME}/.local/bin/ocd
