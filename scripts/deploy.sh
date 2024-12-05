# Ensure the script exits on any errors
set -e

# Check if the arguments are provided
if [ $# -ne 3 ]; then
    echo "Usage: $0 <identity_string> <admin_address> <auction token address>"
    exit 1
fi

IDENTITY_STRING=$1
ADMIN_ADDR=$2
AUCTION_TOKEN_ADDR=$3
NETWORK="testnet"

echo "Build and optimize the contracts...";
echo "Building the contracts...";

make build > /dev/null
cd target/wasm32-unknown-unknown/release

echo "Contracts compiled."
echo "Optimizing contracts..."

soroban contract optimize --wasm phoenix_nft_collections.wasm
soroban contract optimize --wasm phoenix_nft_deployer.wasm
soroban contract optimize --wasm phoenix_nft_auctions.wasm

echo "Contracts optimized."

echo "Deploy and install the deployer contract and capture its contract ID and hash..."

DEPLOYER_ADDR=$(
soroban contract deploy \
  --wasm phoenix_nft_deployer.optimized.wasm \
  --source $IDENTITY_STRING \
  --network $NETWORK
)

DEPLOYER_WASM_HASH=$(
soroban contract install \
    --wasm phoenix_nft_deployer.optimized.wasm \
    --source $IDENTITY_STRING  \
    --network $NETWORK
)

echo "Deployer contract deployed and installed."

echo "Deploy and install the collections contract and capture its contract ID and hash..."

COLLECTIONS_ADDR=$(
soroban contract deploy \
  --wasm phoenix_nft_collections.optimized.wasm \
  --source $IDENTITY_STRING \
  --network $NETWORK
)

COLLECTIONS_WASM_HASH=$(
soroban contract install \
    --wasm phoenix_nft_collections.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK
)

echo "Collections contract deployed and installed."

echo "Deploy and install the auctions contract and capture its contract ID and hash..."

AUCTION_ADDR=$(
stellar contract deploy \
  --wasm phoenix_nft_auctions.optimized.wasm \
  --source $IDENTITY_STRING \
  --network $NETWORK
)

AUCTION_WASM_HASH=$(
soroban contract install \
    --wasm phoenix_nft_auctions.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK
)

echo "Auctions contract deployed and installed."

echo "Initialize deployer with the collections hash..."

soroban contract invoke \
    --id $DEPLOYER_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    initialize \
    --collections_wasm_hash $COLLECTIONS_WASM_HASH

echo "Deployer initialized."

echo "Initialize auctions with the auction token address"

soroban contract invoke \
    --id $AUCTION_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    initialize \
    --admin $ADMIN_ADDR
    --auction_token $AUCTION_TOKEN_ADDR

echo "Auction initialized"

echo "#############################"

echo "Setup complete!"
echo "Deployer address: $DEPLOYER_ADDR"
echo "Deployer wasm hash: $DEPLOYER_WASM_HASH"
echo "Collections address: $COLLECTIONS_ADDR"
echo "Collections wasm hash: $COLLECTIONS_WASM_HASH"
