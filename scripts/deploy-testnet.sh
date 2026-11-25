#!/usr/bin/env bash
#
# Builds the contracts, deploys them to Stellar testnet and initializes the
# subscription contract straight away, so nobody else can initialize it first.
#
# Requirements: Rust with the wasm32v1-none target, and the Stellar CLI
# (https://developers.stellar.org/docs/tools/cli).
#
# Environment variables (all optional):
#   ADMIN               CLI identity that deploys and becomes admin   (default: lumensub-admin)
#   TREASURY            address that receives protocol fees          (default: the admin)
#   FEE_BPS             protocol fee in basis points                 (default: 100, i.e. 1%)
#   KEEPER_REWARD_BPS   keeper reward in basis points                (default: 50, i.e. 0.5%)
#   NETWORK             CLI network name                             (default: testnet)
#
# The deployed contract ids are written to .env.testnet.

set -euo pipefail

ADMIN="${ADMIN:-lumensub-admin}"
NETWORK="${NETWORK:-testnet}"
FEE_BPS="${FEE_BPS:-100}"
KEEPER_REWARD_BPS="${KEEPER_REWARD_BPS:-50}"
WASM_DIR="target/wasm32v1-none/release"

cd "$(dirname "$0")/.."

if ! stellar keys address "$ADMIN" >/dev/null 2>&1; then
  echo "Creating and funding identity '$ADMIN' on $NETWORK"
  stellar keys generate "$ADMIN" --network "$NETWORK" --fund
fi
ADMIN_ADDRESS="$(stellar keys address "$ADMIN")"
TREASURY="${TREASURY:-$ADMIN_ADDRESS}"

echo "Building contracts"
cargo build --workspace --target wasm32v1-none --release

deploy() {
  stellar contract deploy \
    --wasm "$WASM_DIR/$1.wasm" \
    --source "$ADMIN" \
    --network "$NETWORK"
}

echo "Deploying subscription contract"
SUBSCRIPTION_ID="$(deploy lumensub_subscription)"

echo "Initializing subscription contract"
stellar contract invoke --id "$SUBSCRIPTION_ID" --source "$ADMIN" --network "$NETWORK" -- \
  initialize \
  --admin "$ADMIN_ADDRESS" \
  --treasury "$TREASURY" \
  --fee_bps "$FEE_BPS" \
  --keeper_reward_bps "$KEEPER_REWARD_BPS"

echo "Deploying splitter contract"
SPLITTER_ID="$(deploy lumensub_splitter)"

cat > .env.testnet <<EOF
NETWORK=$NETWORK
SUBSCRIPTION_CONTRACT_ID=$SUBSCRIPTION_ID
SPLITTER_CONTRACT_ID=$SPLITTER_ID
ADMIN_ADDRESS=$ADMIN_ADDRESS
TREASURY_ADDRESS=$TREASURY
EOF

echo
echo "Subscription: $SUBSCRIPTION_ID"
echo "Splitter:     $SPLITTER_ID"
echo "Saved to .env.testnet"
