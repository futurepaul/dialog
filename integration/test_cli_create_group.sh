#!/bin/bash

# Dialog CLI test script for creating a group and inviting dialog_tui

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Load environment
export $(cat .env.local | xargs)

echo -e "${YELLOW}=== Dialog CLI Group Creation Test ===${NC}"
echo

# Alice's pubkey (from dialog_tui) - this would be provided by the automation script
if [ -z "$1" ]; then
    echo "Usage: $0 <alice_pubkey_hex>"
    echo "Get Alice's pubkey from dialog_tui automation first"
    exit 1
fi

ALICE_PUBKEY=$1

echo "Using Bob's key from .env.local"
echo "Alice's pubkey: $ALICE_PUBKEY"
echo

# Initialize Bob's profile
echo -e "${YELLOW}Initializing Bob's profile...${NC}"
cargo run --bin dialog_cli -- --key bob profile --name "Bob" --display "Bob Test" --nip05 "bob@test.local"

# Add Alice as a contact
echo -e "${YELLOW}Adding Alice as a contact...${NC}"
cargo run --bin dialog_cli -- --key bob contacts add $ALICE_PUBKEY --name "Alice"

# List contacts to verify
echo -e "${YELLOW}Listing contacts...${NC}"
cargo run --bin dialog_cli -- --key bob contacts list

# Publish key packages
echo -e "${YELLOW}Publishing Bob's key packages...${NC}"
cargo run --bin dialog_cli -- --key bob keypackage publish

# Create a group with Alice
echo -e "${YELLOW}Creating group 'Test Interop Group' with Alice...${NC}"
cargo run --bin dialog_cli -- --key bob groups create "Test Interop Group" --members $ALICE_PUBKEY

# List groups to get the group ID
echo -e "${YELLOW}Listing groups...${NC}"
cargo run --bin dialog_cli -- --key bob groups list

# Get the group ID (assuming it's the first/only group)
GROUP_ID=$(cargo run --bin dialog_cli -- --key bob groups list 2>/dev/null | grep -oE '[0-9a-f]{64}' | head -1)

if [ -z "$GROUP_ID" ]; then
    echo -e "${RED}Failed to get group ID${NC}"
    exit 1
fi

echo -e "${GREEN}Group created with ID: $GROUP_ID${NC}"
echo

# Send initial message
echo -e "${YELLOW}Sending initial message...${NC}"
cargo run --bin dialog_cli -- --key bob messages send $GROUP_ID "Hello from dialog_cli! Welcome to the test group."

# Wait a bit for dialog_tui to respond
echo -e "${YELLOW}Waiting for dialog_tui to join and respond...${NC}"
sleep 5

# Fetch messages
echo -e "${YELLOW}Fetching messages...${NC}"
cargo run --bin dialog_cli -- --key bob messages list $GROUP_ID

echo
echo -e "${GREEN}Test setup complete!${NC}"
echo "Group ID: $GROUP_ID"
echo
echo "To continue testing:"
echo "1. Send more messages: cargo run --bin dialog_cli -- --key bob messages send $GROUP_ID \"Your message\""
echo "2. Fetch messages: cargo run --bin dialog_cli -- --key bob messages list $GROUP_ID"