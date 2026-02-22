#!/bin/bash

# Game Contract Deployment Script
# This script deploys the XLMate Game Contract to Stellar Testnet

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 XLMate Game Contract Deployment Script${NC}"
echo "=================================="

# Check if soroban-cli is installed
if ! command -v soroban &> /dev/null; then
    echo -e "${RED}❌ Soroban CLI is not installed. Installing...${NC}"
    cargo install --locked soroban-cli
    echo -e "${GREEN}✅ Soroban CLI installed successfully${NC}"
else
    echo -e "${GREEN}✅ Soroban CLI is already installed${NC}"
fi

# Check if WASM file exists
WASM_FILE="target/wasm32-unknown-unknown/release/game_contract.wasm"
if [ ! -f "$WASM_FILE" ]; then
    echo -e "${YELLOW}🔨 Building contract...${NC}"
    cargo build --release --target wasm32-unknown-unknown
    echo -e "${GREEN}✅ Contract built successfully${NC}"
else
    echo -e "${GREEN}✅ Contract WASM file already exists${NC}"
fi

# Check if secret key is provided
if [ -z "$1" ]; then
    echo -e "${RED}❌ Please provide your secret key as an argument${NC}"
    echo "Usage: ./deploy.sh <YOUR_SECRET_KEY>"
    exit 1
fi

SECRET_KEY=$1

# Configure network
echo -e "${YELLOW}🌐 Configuring Stellar Testnet network...${NC}"
soroban config network --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

echo -e "${GREEN}✅ Network configured${NC}"

# Deploy contract
echo -e "${YELLOW}🚀 Deploying contract to Stellar Testnet...${NC}"
CONTRACT_ID=$(soroban contract deploy \
  --wasm "$WASM_FILE" \
  --source "$SECRET_KEY" \
  --network testnet)

echo -e "${GREEN}✅ Contract deployed successfully!${NC}"
echo -e "${GREEN}📋 Contract ID: $CONTRACT_ID${NC}"

# Save contract ID to file
echo "$CONTRACT_ID" > contract_id.txt
echo -e "${GREEN}💾 Contract ID saved to contract_id.txt${NC}"

# Verify deployment
echo -e "${YELLOW}🔍 Verifying deployment...${NC}"
soroban contract info \
  --id "$CONTRACT_ID" \
  --network testnet

echo -e "${GREEN}✅ Deployment verified successfully!${NC}"
echo ""
echo -e "${GREEN}🎮 Next steps:${NC}"
echo "1. Use the contract ID in your frontend application"
echo "2. Test contract functions using soroban contract invoke"
echo "3. Check DEPLOYMENT.md for usage examples"
echo ""
echo -e "${YELLOW}⚠️  Remember: This is deployed to Testnet. Use test XLM only!${NC}"
