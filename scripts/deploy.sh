# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

IDENTITY_STRING=$1
NETWORK="testnet"

echo "Build and optimize the contracts...";
echo "Building the contracts...";

make build > /dev/null
cd target/wasm32-unknown-unknown/release

echo "Contracts compiled."
echo "Optimizing contracts..."

soroban contract optimize --wasm phoenix_nft_collections.wasm
soroban contract optimize --wasm phoenix_nft_deployer.wasm

echo "Contracts optimized."

echo "Deploy and install the deployer contract and capture its contract ID and hash..."

DEPLOYER_ADDR=$(
stellar contract deploy \
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

echo "Deployer set."

echo "Deploy and install the collections contract and capture its contract ID and hash..."

COLLECTIONS_ADDR=$(
stellar contract deploy \
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

echo "Collections set."

echo "Initialize deployer with the collections hash..."


soroban contract invoke \
    --id $DEPLOYER_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    initialize \
    --collections_wasm_hash $COLLECTIONS_WASM_HASH

echo "Deployer initialized."

echo "#############################"

echo "Setup complete!"
echo "Deployer address: $DEPLOYER_ADDR"
echo "Deployer wasm hash: $DEPLOYER_WASM_HASH"
echo "Collections address: $COLLECTIONS_ADDR"
echo "Collections wasm hash: $COLLECTIONS_WASM_HASH"
