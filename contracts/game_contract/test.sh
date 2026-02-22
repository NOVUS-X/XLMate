#!/bin/bash

# Game Contract Test Script
# This script tests basic contract functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🧪 XLMate Game Contract Test Script${NC}"
echo "====================================="

# Check if contract ID file exists
if [ ! -f "contract_id.txt" ]; then
    echo -e "${RED}❌ contract_id.txt not found. Please deploy the contract first.${NC}"
    echo "Run: ./deploy.sh <YOUR_SECRET_KEY>"
    exit 1
fi

CONTRACT_ID=$(cat contract_id.txt)
echo -e "${GREEN}📋 Using Contract ID: $CONTRACT_ID${NC}"

# Check if secret keys are provided
if [ -z "$1" ] || [ -z "$2" ]; then
    echo -e "${RED}❌ Please provide two secret keys for testing${NC}"
    echo "Usage: ./test.sh <PLAYER1_SECRET> <PLAYER2_SECRET>"
    exit 1
fi

PLAYER1_SECRET=$1
PLAYER2_SECRET=$2

# Generate addresses
PLAYER1_ADDRESS=$(soroban keys address "$PLAYER1_SECRET")
PLAYER2_ADDRESS=$(soroban keys address "$PLAYER2_SECRET")

echo -e "${YELLOW}👥 Player 1 Address: $PLAYER1_ADDRESS${NC}"
echo -e "${YELLOW}👥 Player 2 Address: $PLAYER2_ADDRESS${NC}"

# Test 1: Create a game
echo -e "${YELLOW}🎮 Test 1: Creating game...${NC}"
GAME_ID=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$PLAYER1_SECRET" \
  --network testnet \
  --create_game \
  --player1 "$PLAYER1_ADDRESS" \
  --wager-amount 10000000  # 0.1 XLM in stroops)

echo -e "${GREEN}✅ Game created with ID: $GAME_ID${NC}"

# Test 2: Get game info
echo -e "${YELLOW}🔍 Test 2: Getting game info...${NC}"
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$PLAYER1_SECRET" \
  --network testnet \
  --get_game \
  --game-id "$GAME_ID"

# Test 3: Join game
echo -e "${YELLOW}🎮 Test 3: Player 2 joining game...${NC}"
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$PLAYER2_SECRET" \
  --network testnet \
  --join_game \
  --game-id "$GAME_ID" \
  --player2 "$PLAYER2_ADDRESS"

echo -e "${GREEN}✅ Player 2 joined the game${NC}"

# Test 4: Submit a move
echo -e "${YELLOW}🎮 Test 4: Player 1 submitting move...${NC}"
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$PLAYER1_SECRET" \
  --network testnet \
  --submit_move \
  --game-id "$GAME_ID" \
  --player "$PLAYER1_ADDRESS" \
  --move-data "[1,2,3,4]"

echo -e "${GREEN}✅ Move submitted successfully${NC}"

# Test 5: Get updated game info
echo -e "${YELLOW}🔍 Test 5: Getting updated game info...${NC}"
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$PLAYER1_SECRET" \
  --network testnet \
  --get_game \
  --game-id "$GAME_ID"

# Test 6: Forfeit game
echo -e "${YELLOW}🎮 Test 6: Player 1 forfeiting game...${NC}"
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$PLAYER1_SECRET" \
  --network testnet \
  --forfeit \
  --game-id "$GAME_ID" \
  --player "$PLAYER1_ADDRESS"

echo -e "${GREEN}✅ Game forfeited${NC}"

echo -e "${GREEN}🎉 All tests completed successfully!${NC}"
echo -e "${YELLOW}💡 You can now integrate this contract with your frontend application${NC}"
