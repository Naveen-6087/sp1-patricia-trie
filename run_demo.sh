#!/bin/bash

# Configuration
ANVIL_PORT=8545
RPC_URL="http://127.0.0.1:$ANVIL_PORT"
PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" # Standard Anvil Account #0

# Paths
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTRACT_DIR="$ROOT_DIR/simpleStorage"
SERVER_DIR="$ROOT_DIR/sp1-lightclient/script"

# Cleanup function
cleanup() {
    echo "Cleaning up processes..."
    if [ -n "$ANVIL_PID" ]; then
        kill $ANVIL_PID || true
    fi
}
trap cleanup EXIT

echo "------------------------------------------------------------------"
echo "Starting Automated SP1 Oracle Demo"
echo "------------------------------------------------------------------"

# 1. Start Anvil
echo "[1/4] Starting local Anvil node..."
pkill -f "anvil --port $ANVIL_PORT" || true # Kill existing instance
anvil --port $ANVIL_PORT > /dev/null &
ANVIL_PID=$!
sleep 2

# 2. Deploy Contract
echo "[2/4] Deploying SimpleStorage contract..."
cd "$CONTRACT_DIR"
DEPLOY_OUTPUT=$(forge create src/SimpleStorage.sol:SimpleStorage \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast)

# Extract Contract Address (Grep for "Deployed to: 0x...")
CONTRACT_ADDR=$(echo "$DEPLOY_OUTPUT" | grep "Deployed to:" | awk '{print $3}')
echo "      Contract Deployed at: $CONTRACT_ADDR"

if [ -z "$CONTRACT_ADDR" ]; then
    echo "Error: Failed to capture contract address."
    echo "$DEPLOY_OUTPUT"
    # Don't exit immediately so user can see
    read -p "Press Enter to exit..."
    exit 1
fi

# 3. Set Initial Value
echo "[3/4] Setting storage value to 42..."
cast send $CONTRACT_ADDR "set(uint256)" 42 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY > /dev/null
echo "      Value set successfully."

# 4. Run Oracle Server
echo "[4/4] Launching Oracle Server..."
echo "      Use Ctrl+C to stop."
echo "------------------------------------------------------------------"

cd "$SERVER_DIR"
export SOURCE_RPC_URL=$RPC_URL
export TARGET_CONTRACT=$CONTRACT_ADDR
export TARGET_SLOT=0x0000000000000000000000000000000000000000000000000000000000000000

# ENABLE REAL PROOOFS
export USE_REAL_PROOF=true

# Run in release mode for SP1
RUST_LOG=info cargo run --release --bin server || echo "Server crashed or stopped."

echo "------------------------------------------------------------------"
echo "Process finished. Press Enter to exit."
read -r
