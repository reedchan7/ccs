.PHONY: fmt lint test build install release-local

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all

build:
	cargo build --locked

install:
	cargo build --release --locked
	mkdir -p "$$HOME/.local/bin"
	cp target/release/ccs "$$HOME/.local/bin/ccs"
	"$$HOME/.local/bin/ccs" init --hooks-only

release-local:
	cargo build --release --locked
