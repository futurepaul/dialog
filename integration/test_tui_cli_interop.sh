#!/bin/bash

# Test script for dialog_tui and dialog_cli interoperability

set -e

echo "=== Dialog TUI/CLI Interoperability Test ==="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Load environment
export $(cat .env.local | xargs)

# Step 1: Start the local relay
echo "Starting local Nostr relay..."
nak serve --verbose &
RELAY_PID=$!
sleep 2

# Cleanup function
cleanup() {
    echo
    echo "Cleaning up..."
    kill $RELAY_PID 2>/dev/null || true
    exit
}
trap cleanup EXIT

# Step 2: Build both binaries
echo "Building dialog_cli and dialog_tui..."
cargo build --bin dialog_cli --bin dialog_tui

echo
echo "Test environment ready!"
echo "Relay running on ws://localhost:10547"
echo
echo "To run the test:"
echo "1. In terminal 1: cargo run --bin dialog_tui -- --key alice"
echo "2. In terminal 2: Run the ht-mcp automation script"
echo "3. In terminal 3: Use dialog_cli to create groups and send messages"
echo
echo "Press Ctrl+C to stop the relay and exit"

# Keep the relay running
wait $RELAY_PID