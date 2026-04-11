default: build

all: test

test: build
	cargo test

build:
	cd ../.. && stellar contract build --package soroban-token-contract

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --tests -- -D warnings

clean:
	cargo clean
