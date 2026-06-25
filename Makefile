.DEFAULT_GOAL := help

.PHONY: help fmt lint test build install release-local

help: ## Show available make targets
	@awk 'BEGIN {FS = ":.*## "}; /^[[:alnum:]_-]+:.*## / {printf "make %-13s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

fmt: ## Format Rust code
	cargo fmt --all

lint: ## Run clippy with warnings denied
	cargo clippy --all-targets --all-features -- -D warnings

test: ## Run tests
	cargo test --all

build: ## Build debug binary
	cargo build --locked

install: ## Install local source build
	cargo build --release
	mkdir -p "$$HOME/.local/bin"
	cp target/release/ccs "$$HOME/.local/bin/ccs"
	"$$HOME/.local/bin/ccs" init --hooks-only

release-local: ## Build release binary
	cargo build --release --locked
