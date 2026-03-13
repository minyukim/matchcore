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
test:								## Run tests
	cargo test --all-features

.PHONY: check
check: fmt-check lint test build	## Run pre-push checks