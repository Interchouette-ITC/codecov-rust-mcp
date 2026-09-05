# codecov-mcp - local gates (match CI)

CLIPPY_FLAGS := -D warnings -D clippy::all -D clippy::pedantic -D clippy::nursery
CARGO ?= cargo +stable

.PHONY: build release lint test coverage doc ci run help

help:
	@echo "Targets: build release lint test coverage doc ci run"

build:
	$(CARGO) build

release:
	$(CARGO) build --release

lint:
	$(CARGO) fmt --check
	$(CARGO) clippy --all-targets -- $(CLIPPY_FLAGS)

test:
	$(CARGO) test --all-targets

## Requires cargo-llvm-cov + llvm-tools-preview. Writes coverage/lcov.info.
coverage:
	mkdir -p coverage
	RUSTUP_TOOLCHAIN=stable $(CARGO) llvm-cov --lcov \
		--ignore-filename-regex 'examples/|src/main\.rs' \
		--output-path coverage/lcov.info

doc:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps

ci: lint test doc

run: build
	$(CARGO) run --quiet
