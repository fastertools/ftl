.PHONY: build clean test check format lint

build:
	cargo build --target wasm32-wasip1 --release

clean:
	cargo clean

test:
	cargo test

check:
	cargo check --target wasm32-wasip1

format:
	cargo fmt

lint:
	cargo clippy --target wasm32-wasip1 -- -D warnings

dev: format lint test
	@echo "Development checks passed!"