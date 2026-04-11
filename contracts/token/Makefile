default: build

all: test

test: build
	cargo test

build:
	stellar contract build

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --tests -- -D warnings

clean:
	cargo clean
