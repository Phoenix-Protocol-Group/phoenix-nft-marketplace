#!/bin/bash
# Ensure the script exits on any errors
set -euo pipefail

# Retry wrapper for stellar CLI commands that may timeout
retry() {
    local max_attempts=3
    local delay=5
    local attempt=1
    local cmd="$@"

    while [ $attempt -le $max_attempts ]; do
        echo "  Attempt $attempt/$max_attempts..."
        if output=$(eval "$cmd" 2>&1); then
            echo "$output"
            return 0
        fi

        if echo "$output" | grep -qi "timeout\|timed out\|504\|503\|connection"; then
            echo "  Timeout/connection error. Retrying in ${delay}s..."
            sleep $delay
            delay=$((delay * 2))
            attempt=$((attempt + 1))
        else
            echo "  Error (non-retryable):"
            echo "$output"
            return 1
        fi
    done

    echo "  Failed after $max_attempts attempts."
    return 1
}

# Check if the arguments are provided
if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    echo "Usage: $0 <identity_string> [network]"
    echo "  network: testnet (default) or mainnet"
    exit 1
fi

IDENTITY_STRING=$1
NETWORK="${2:-testnet}"
ADMIN_ADDRESS=$(stellar keys address "$IDENTITY_STRING")

# Token address for auction payments
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

# Build
echo ""
echo "Building contracts with stellar contract build..."
make build
echo "Contracts built."

# Verify WASM files exist
for wasm in phoenix_nft_deployer.wasm phoenix_nft_collections.wasm phoenix_nft_auctions.wasm; do
    if [ ! -f "$WASM_DIR/$wasm" ]; then
        echo "ERROR: $WASM_DIR/$wasm not found"
        exit 1
    fi
done

# Deploy deployer contract
echo ""
echo "Deploying deployer contract..."
DEPLOYER_ADDR=$(retry "stellar contract deploy \
    --wasm $WASM_DIR/phoenix_nft_deployer.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK")
echo "Deployer deployed: $DEPLOYER_ADDR"

# Install collections WASM (needed for deployer to deploy new collections)
echo ""
echo "Installing collections WASM..."
COLLECTIONS_WASM_HASH=$(retry "stellar contract install \
    --wasm $WASM_DIR/phoenix_nft_collections.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK")
echo "Collections WASM hash: $COLLECTIONS_WASM_HASH"

# Initialize deployer with collections hash
echo ""
echo "Initializing deployer..."
retry "stellar contract invoke \
    --id $DEPLOYER_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    initialize \
    --collections_wasm_hash $COLLECTIONS_WASM_HASH"
echo "Deployer initialized."

# Deploy marketplace contract
echo ""
echo "Deploying marketplace contract..."
MARKETPLACE_ADDRESS=$(retry "stellar contract deploy \
    --wasm $WASM_DIR/phoenix_nft_auctions.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK")
echo "Marketplace deployed: $MARKETPLACE_ADDRESS"

# Initialize marketplace
echo ""
echo "Initializing marketplace..."
retry "stellar contract invoke \
    --id $MARKETPLACE_ADDRESS \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --auction_token $AUCTION_TOKEN \
    --auction_creation_fee $AUCTION_CREATION_FEE \
    --min_bid_increment $MIN_BID_INCREMENT"
echo "Marketplace initialized."

# Summary
echo ""
echo "========================================"
echo "Deployment Complete!"
echo "========================================"
echo "Deployer address:       $DEPLOYER_ADDR"
echo "Collections WASM hash:  $COLLECTIONS_WASM_HASH"
echo "Marketplace address:    $MARKETPLACE_ADDRESS"
echo "========================================"
