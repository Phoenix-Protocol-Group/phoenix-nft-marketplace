default: all

all: lint build test

test: build
	$(MAKE) -C ../deployer build || break;
	cargo test

build:
	cargo build --target wasm32-unknown-unknown --release

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
