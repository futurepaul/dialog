#!/bin/bash
set -e

echo "=== Automated Dialog TUI-CLI Interop Test ==="

# Cleanup function
cleanup() {
    echo "Cleaning up..."
    kill $RELAY_PID 2>/dev/null || true
    kill $TUI_PID 2>/dev/null || true
}
trap cleanup EXIT

# 1. Start relay
echo "Starting relay..."
nak serve --verbose &
RELAY_PID=$!
sleep 2

# 2. Start TUI with automation
echo "Starting dialog_tui with automation..."
(
    cat << 'EOF' | expect -f -
#!/usr/bin/expect -f
set timeout 30
spawn cargo run --bin dialog_tui -- --key alice
expect "Connected to relay successfully"
expect "Published 5 key packages"

# Handle invite
expect "New group invitation received" {
    send "/invites\r"
    expect "Test Interop Group"
    send "\r"
    expect "Successfully joined group"
}

# Send message when we see CLI message
expect "75427ab8...: Hello from CLI" {
    send "Hello from TUI!\r"
}

# Keep alive
expect "impossible_string" { exp_continue }
EOF
) &
TUI_PID=$!
sleep 5

# 3. Get Alice's pubkey
echo "Getting Alice's public key..."
ALICE_PUBKEY=$(cargo run --bin dialog_cli -- get-pubkey --key alice 2>/dev/null | grep -E '^[a-f0-9]{64}$' | head -1)
echo "Alice pubkey: $ALICE_PUBKEY"

# 4. Bob workflow
echo "Publishing Bob's key packages..."
cargo run --bin dialog_cli -- publish-key --key bob

echo "Creating group and inviting Alice..."
GROUP_ID=$(cargo run --bin dialog_cli -- create-group --key bob --name "Test Interop Group" --counterparty $ALICE_PUBKEY 2>&1 | grep "Group ID:" | awk '{print $NF}')
echo "Created group: $GROUP_ID"

# 5. Send message and wait
echo "Bob sending message..."
cargo run --bin dialog_cli -- send-message --key bob --group-id $GROUP_ID --message "Hello from CLI!"
sleep 3

# 6. Verify messages
echo "Fetching messages to verify..."
MESSAGES=$(cargo run --bin dialog_cli -- get-messages --key bob --group-id $GROUP_ID 2>/dev/null)

if echo "$MESSAGES" | grep -q "Hello from CLI!" && echo "$MESSAGES" | grep -q "Hello from TUI!"; then
    echo "✅ Test PASSED: Bidirectional messaging verified!"
    echo "$MESSAGES"
    exit 0
else
    echo "❌ Test FAILED: Messages not found"
    echo "$MESSAGES"
    exit 1
fi