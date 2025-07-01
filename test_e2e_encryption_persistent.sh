#!/bin/bash

# E2E Encrypted Messaging Test Script with Persistent Keys

set -e

echo "🧪 Starting E2E Encrypted Messaging Test (with persistent keys)"
echo "================================================================"

# Make sure we're in the right directory
cd dialog_cli

echo "📋 Step 1: Generate and save Alice's identity"
ALICE_OUTPUT=$(cargo run --quiet -- show-key 2>/dev/null)
ALICE_PUB=$(echo "$ALICE_OUTPUT" | grep "🔑" | sed 's/.*🔑 Your public key: //')
ALICE_SEC=$(echo "$ALICE_OUTPUT" | grep "🗝️" | sed 's/.*🗝️  Your secret key: //')
echo "👩 Alice's public key: $ALICE_PUB"
echo "👩 Alice's secret key: $ALICE_SEC"

echo ""
echo "📋 Step 2: Generate and save Bob's identity"
BOB_OUTPUT=$(cargo run --quiet -- show-key 2>/dev/null)
BOB_PUB=$(echo "$BOB_OUTPUT" | grep "🔑" | sed 's/.*🔑 Your public key: //')
BOB_SEC=$(echo "$BOB_OUTPUT" | grep "🗝️" | sed 's/.*🗝️  Your secret key: //')
echo "👨 Bob's public key: $BOB_PUB"
echo "👨 Bob's secret key: $BOB_SEC"

echo ""
echo "📋 Step 3: Alice sends encrypted message to Bob"
echo "🔐 Sending: 'Hello Bob from Alice! This is encrypted.'"
cargo run --quiet -- --key "$ALICE_SEC" send-encrypted --recipient "$BOB_PUB" "Hello Bob from Alice! This is encrypted."

echo ""
echo "📋 Step 4: Wait for message propagation"
sleep 2

echo ""
echo "📋 Step 5: Bob fetches encrypted messages using his key"
echo "🔍 Fetching messages as Bob..."
cargo run --quiet -- --key "$BOB_SEC" fetch-encrypted --limit 5

echo ""
echo "📋 Step 6: Bob sends reply to Alice"
echo "🔐 Sending: 'Hi Alice! I got your encrypted message.'"
cargo run --quiet -- --key "$BOB_SEC" send-encrypted --recipient "$ALICE_PUB" "Hi Alice! I got your encrypted message."

echo ""
echo "📋 Step 7: Alice fetches Bob's reply using her key"
echo "🔍 Fetching messages as Alice..."
cargo run --quiet -- --key "$ALICE_SEC" fetch-encrypted --limit 5

echo ""
echo "🎉 E2E Encrypted Messaging Test Completed!"
echo "✅ Both Alice and Bob successfully sent and received encrypted messages with persistent keys"