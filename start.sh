#!/bin/bash

echo "Starting Solana Transaction Service..."
echo "====================================="

# Set environment variables
export SOLANA_RPC_URL=${SOLANA_RPC_URL:-"https://api.mainnet-beta.solana.com"}

echo "Using RPC URL: $SOLANA_RPC_URL"
echo "Service will be available at: http://localhost:3000"
echo ""

# Build and run the service
cargo run --release
