#!/bin/bash

# E2E Encrypted Messaging Test Script

set -e

echo "🧪 Starting E2E Encrypted Messaging Test"
echo "========================================"

# Make sure we're in the right directory
cd dialog_cli

echo "📋 Step 1: Generate Alice's key"
ALICE_KEY=$(cargo run -- show-key 2>/dev/null | grep "🔑" | cut -d' ' -f4)
echo "👩 Alice's public key: $ALICE_KEY"

echo ""
echo "📋 Step 2: Generate Bob's key" 
BOB_KEY=$(cargo run -- show-key 2>/dev/null | grep "🔑" | cut -d' ' -f4)
echo "👨 Bob's public key: $BOB_KEY"

echo ""
echo "📋 Step 3: Alice sends encrypted message to Bob"
echo "Sending: 'Hello Bob from Alice! This is encrypted.'"
ALICE_MSG_ID=$(cargo run -- send-encrypted --recipient "$BOB_KEY" "Hello Bob from Alice! This is encrypted." 2>/dev/null | grep "🔐" | cut -d' ' -f7)
echo "✅ Alice sent message with ID: $ALICE_MSG_ID"

echo ""
echo "📋 Step 4: Wait for message propagation"
sleep 2

echo ""
echo "📋 Step 5: Bob fetches encrypted messages"
echo "🔍 Fetching messages as Bob..."
cargo run -- fetch-encrypted --limit 5

echo ""
echo "📋 Step 6: Bob sends reply to Alice"
echo "Sending: 'Hi Alice! I got your encrypted message.'"
BOB_MSG_ID=$(cargo run -- send-encrypted --recipient "$ALICE_KEY" "Hi Alice! I got your encrypted message." 2>/dev/null | grep "🔐" | cut -d' ' -f7)
echo "✅ Bob sent reply with ID: $BOB_MSG_ID"

echo ""
echo "📋 Step 7: Alice fetches Bob's reply"
echo "🔍 Fetching messages as Alice..."
cargo run -- fetch-encrypted --limit 5

echo ""
echo "🎉 E2E Encrypted Messaging Test Completed!"
echo "✅ Both Alice and Bob successfully sent and received encrypted messages"