# Vamp — terminal-native dev environment for Claude Code
# Usage: make [target]

INSTALL_DIR   ?= $(HOME)/.local/bin
SHARE_DIR     ?= $(HOME)/.local/share/vamp
SIDEBAR_DIR    = sidebar
BATS           = ./tests/bats/bats-core/bin/bats

.PHONY: build install test test-bash test-sidebar dev clean lint demo help

## Build sidebar binary (release)
build:
	cd $(SIDEBAR_DIR) && cargo build --release

## Build + install vamp script, utils, and sidebar binary
install: build
	@mkdir -p $(INSTALL_DIR) $(SHARE_DIR)
	cp bin/vamp $(INSTALL_DIR)/vamp
	cp lib/vamp-utils.sh $(SHARE_DIR)/vamp-utils.sh
	cp $(SIDEBAR_DIR)/target/release/vamp-sidebar $(INSTALL_DIR)/vamp-sidebar
	@echo "Installed vamp to $(INSTALL_DIR)"

## Run all tests (sidebar + bash)
test: test-sidebar test-bash

## Run bats unit tests
test-bash:
	$(BATS) tests/unit/

## Run sidebar Rust tests
test-sidebar:
	cd $(SIDEBAR_DIR) && cargo test

## Quick dev iteration: build + install
dev: install

## Remove build artifacts
clean:
	cd $(SIDEBAR_DIR) && cargo clean

## Run clippy linter
lint:
	cd $(SIDEBAR_DIR) && cargo clippy -- -W warnings

## Run sidebar in demo mode with fixture data
demo: build
	cd $(SIDEBAR_DIR) && cargo run -- --demo

## Show available targets
help:
	@echo "Vamp Makefile targets:"
	@echo "  build         Build sidebar binary (release)"
	@echo "  install       Build + install vamp, utils, sidebar"
	@echo "  test          Run all tests (sidebar + bash)"
	@echo "  test-bash     Run bats unit tests"
	@echo "  test-sidebar  Run sidebar Rust tests"
	@echo "  dev           Quick dev iteration (build + install)"
	@echo "  clean         Remove build artifacts"
	@echo "  lint          Run clippy linter"
	@echo "  demo          Run sidebar in demo mode"
	@echo "  help          Show this help"
