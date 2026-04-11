default: all

all: lint build test

test: build # because of token dependency
	cargo test

build:
	cd ../.. && stellar contract build --package phoenix-nft-auctions

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
