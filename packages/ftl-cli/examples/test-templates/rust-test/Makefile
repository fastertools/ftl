.PHONY: build test clean

build:
	cd handler && cargo build --release --target wasm32-wasip1

test:
	cd handler && cargo test

clean:
	cd handler && cargo clean

serve:
	ftl serve

deploy:
	ftl deploy