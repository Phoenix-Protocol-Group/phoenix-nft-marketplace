SUBDIRS := contracts/token contracts/collections contracts/auctions contracts/deployer
BUILD_FLAGS ?=

default: build

all: test

build:
	stellar contract build --package soroban-token-contract
	stellar contract build --package phoenix-nft-collections
	stellar contract build --package phoenix-nft-auctions
	stellar contract build --package phoenix-nft-deployer

test: build
	@for dir in $(SUBDIRS) ; do \
		$(MAKE) -C $$dir test BUILD_FLAGS=$(BUILD_FLAGS) || exit 1; \
	done

fmt:
	@for dir in $(SUBDIRS) ; do \
		$(MAKE) -C $$dir fmt || exit 1; \
	done

lints: fmt
	@for dir in $(SUBDIRS) ; do \
		$(MAKE) -C $$dir clippy || exit 1; \
	done

clean:
	@for dir in $(SUBDIRS) ; do \
		$(MAKE) -C $$dir clean || exit 1; \
	done
