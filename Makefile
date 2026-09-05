# codecov-mcp - local gates (match CI)

CLIPPY_FLAGS := -D warnings -D clippy::all -D clippy::pedantic -D clippy::nursery
CARGO ?= cargo +stable

.PHONY: build release lint test ci run help

help:
	@echo "Targets: build release lint test ci run"

build:
	$(CARGO) build

release:
	$(CARGO) build --release

lint:
	$(CARGO) fmt --check
	$(CARGO) clippy --all-targets -- $(CLIPPY_FLAGS)

test:
	$(CARGO) test --all-targets

ci: lint test

run: build
	$(CARGO) run --quiet
