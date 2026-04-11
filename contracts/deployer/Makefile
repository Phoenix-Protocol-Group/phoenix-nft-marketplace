default: all

all: lint build test

test: build
	$(MAKE) -C ../deployer build || break;
	cargo test

build:
	cd ../.. && stellar contract build --package phoenix-nft-deployer

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
