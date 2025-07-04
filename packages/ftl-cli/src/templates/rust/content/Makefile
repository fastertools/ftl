.PHONY: build test clean registry-push

build:
	cd handler && cargo component build --release --target wasm32-wasip1

test:
	cd handler && cargo test

clean:
	cd handler && cargo clean

registry-push: build
	@echo "Pushing to registry..."
	@VERSION=$$(cd handler && cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version') && \
	USERNAME=$$(git config user.name | tr '[:upper:]' '[:lower:]' | tr ' ' '-') && \
	PROJECT={{project-name | snake_case}} && \
	echo "Pushing ghcr.io/$$USERNAME/$$PROJECT:$$VERSION" && \
	wkg oci push ghcr.io/$$USERNAME/$$PROJECT:$$VERSION handler/target/wasm32-wasip1/release/$$PROJECT.wasm

up:
	spin up