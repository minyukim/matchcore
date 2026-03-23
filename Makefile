# Makefile for the project

.PHONY: help
help:								## Show this help message
	@grep -E '^[a-zA-Z0-9_-]+:.*## ' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":.*## "}; {t[NR]=$$1; d[NR]=$$2; if (length($$1)>w) w=length($$1)} \
	END {for (i=1;i<=NR;i++) printf "%-*s  %s\n", w, t[i], d[i]}'

.PHONY: build
build:								## Build the project examples
	cargo build --examples

.PHONY: clean
clean:								## Clean the project
	cargo clean

.PHONY: fmt
fmt:								## Format the code
	cargo fmt --all

.PHONY: fmt-check
fmt-check:							## Check formatting
	cargo fmt --all -- --check

.PHONY: lint
lint:								## Lint the code
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: lint-fix
lint-fix: 							## Fix linting issues
	cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged -- -D warnings

.PHONY: test
test:								## Run all tests
	cargo test --all-features

.PHONY: check
check: fmt-check lint test build	## Run pre-push checks

.PHONY: bench
bench:								## Run all benchmarks
	cargo bench --bench benches

.PHONY: readme
readme: check-cargo-reedme			## Generate the README.md file
	cargo +nightly reedme

.PHONY: check-cargo-reedme
check-cargo-reedme:					## Check if cargo-reedme is installed
	@command -v cargo-reedme > /dev/null || (echo "Installing cargo-reedme..."; cargo install cargo-reedme)

.PHONY: docs
docs:								## Build docs and open in browser
	cargo doc --no-deps --document-private-items --open