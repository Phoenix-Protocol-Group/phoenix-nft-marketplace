#!/bin/bash
set -euo pipefail

retry() {
    local max_attempts=3
    local delay=5
    local attempt=1

    while [ $attempt -le $max_attempts ]; do
        if [ $attempt -gt 1 ]; then
            echo "  Retry attempt $attempt/$max_attempts (waiting ${delay}s)..."
            sleep $delay
            delay=$((delay * 2))
        fi

        if "$@"; then
            return 0
        fi

        local exit_code=$?
        attempt=$((attempt + 1))

        if [ $attempt -gt $max_attempts ]; then
            echo "  Failed after $max_attempts attempts."
            return $exit_code
        fi
    done
}

if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    echo "Usage: $0 <identity_string> [network]"
    echo "  network: testnet (default) or mainnet"
    exit 1
fi

IDENTITY_STRING=$1
NETWORK="${2:-testnet}"
ADMIN_ADDRESS=$(stellar keys address "$IDENTITY_STRING")

AUCTION_TOKEN="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
AUCTION_CREATION_FEE="100"
MIN_BID_INCREMENT="1"

WASM_DIR="target/wasm32v1-none/release"

echo "========================================"
echo "Phoenix NFT Marketplace Deployment"
echo "========================================"
echo "Network:  $NETWORK"
echo "Admin:    $ADMIN_ADDRESS"
echo "========================================"

echo ""
echo "Building contracts..."
make build
echo "Contracts built."

for wasm in phoenix_nft_deployer.wasm phoenix_nft_collections.wasm phoenix_nft_auctions.wasm; do
    if [ ! -f "$WASM_DIR/$wasm" ]; then
        echo "ERROR: $WASM_DIR/$wasm not found"
        exit 1
    fi
done

echo ""
echo "Deploying deployer contract..."
DEPLOYER_ADDR=$(retry stellar contract deploy \
    --wasm "$WASM_DIR/phoenix_nft_deployer.wasm" \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Deployer deployed: $DEPLOYER_ADDR"

echo ""
echo "Uploading collections WASM..."
COLLECTIONS_WASM_HASH=$(retry stellar contract upload \
    --wasm "$WASM_DIR/phoenix_nft_collections.wasm" \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Collections WASM hash: $COLLECTIONS_WASM_HASH"

echo ""
echo "Initializing deployer..."
retry stellar contract invoke \
    --id "$DEPLOYER_ADDR" \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK" \
    -- \
    initialize \
    --collections_wasm_hash "$COLLECTIONS_WASM_HASH"
echo "Deployer initialized."

echo ""
echo "Deploying marketplace contract..."
MARKETPLACE_ADDRESS=$(retry stellar contract deploy \
    --wasm "$WASM_DIR/phoenix_nft_auctions.wasm" \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Marketplace deployed: $MARKETPLACE_ADDRESS"

echo ""
echo "Initializing marketplace..."
retry stellar contract invoke \
    --id "$MARKETPLACE_ADDRESS" \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK" \
    -- \
    initialize \
    --admin "$ADMIN_ADDRESS" \
    --auction_token "$AUCTION_TOKEN" \
    --auction_creation_fee "$AUCTION_CREATION_FEE" \
    --min_bid_increment "$MIN_BID_INCREMENT"
echo "Marketplace initialized."

echo ""
echo "========================================"
echo "Deployment Complete!"
echo "========================================"
echo "Deployer address:       $DEPLOYER_ADDR"
echo "Collections WASM hash:  $COLLECTIONS_WASM_HASH"
echo "Marketplace address:    $MARKETPLACE_ADDRESS"
echo "========================================"
