# Dialog TUI ↔ CLI Interop Testing Guide

## Overview
This guide provides step-by-step instructions for testing end-to-end messaging between dialog_tui and dialog_cli using real MLS operations.

## Prerequisites

1. **Environment Setup**: Ensure you have the `.env.local` file with Alice and Bob keys:
   ```bash
   cd /Users/futurepaul/dev/heavy/denoise
   just setup
   ```

2. **Build Both Applications**:
   ```bash
   cargo build -p dialog_tui
   cargo build -p dialog_cli
   ```

3. **Start Local Relay** (if needed):
   ```bash
   # Start your local relay at ws://localhost:8080
   # (Implementation depends on your relay setup)
   ```

## Test Scenario: Alice (CLI) → Bob (TUI)

### Step 1: Start Bob's TUI
In **Terminal 1**, start Bob's TUI:
```bash
cd dialog_tui
cargo run -- --key bob
```

Expected output:
- TUI opens with empty conversations
- Shows "No conversations yet. Use CLI to create groups and invite this TUI."
- Status bar shows "0 contacts • Disconnected"

### Step 2: Get Bob's Public Key
In **Terminal 2**, get Bob's public key for Alice to use:
```bash
cd dialog_cli
cargo run -- get-pubkey --key bob
```

Copy the output public key (should be: `75427ab8309aad26beea8142edf427674e4544604ae4dc5045108ad21fc8a0db`)

### Step 3: Alice Publishes Her Key Package
In **Terminal 2**, Alice publishes her key package:
```bash
cargo run -- publish-key --key alice
```

Expected output:
- Key package event created and published
- Alice's MLS key package is now available on the relay

### Step 4: Bob Publishes His Key Package
In **Terminal 2**, Bob publishes his key package:
```bash
cargo run -- publish-key --key bob
```

Expected output:
- Bob's key package event created and published
- Bob's MLS key package is now available on the relay

### Step 5: Alice Creates Group and Invites Bob
In **Terminal 2**, Alice creates a group and invites Bob:
```bash
cargo run -- create-group --key alice --name "Test Group" --counterparty 75427ab8309aad26beea8142edf427674e4544604ae4dc5045108ad21fc8a0db
```

Expected output:
- Group created successfully
- Bob receives invitation
- Group ID displayed for future use

### Step 6: Check Bob's TUI for Invitation
In **Terminal 1** (Bob's TUI):
- Check if invitation appears in TUI
- Look for pending invites counter or new conversation
- Use `/invites` command to view pending invitations

### Step 7: Bob Accepts Invitation (via TUI)
In **Terminal 1** (Bob's TUI):
- Use TUI commands to accept the invitation
- Or use CLI in Terminal 2: `cargo run -- accept-invite --key bob --group-id <GROUP_ID>`

### Step 8: Alice Sends Message to Group
In **Terminal 2**, Alice sends a message:
```bash
cargo run -- send-message --key alice --group-id <GROUP_ID> --message "Hello Bob from Alice via CLI!"
```

Expected output:
- Message sent successfully
- Bob should receive the message in his TUI

### Step 9: Verify Message in Bob's TUI
In **Terminal 1** (Bob's TUI):
- Check if message appears in the conversation
- Message should show: "Hello Bob from Alice via CLI!"

### Step 10: Bob Sends Reply via TUI
In **Terminal 1** (Bob's TUI):
- Type a reply message in the TUI
- Send the message using the TUI interface

### Step 11: Alice Gets Messages via CLI
In **Terminal 2**, Alice retrieves messages:
```bash
cargo run -- get-messages --key alice --group-id <GROUP_ID>
```

Expected output:
- Shows conversation history
- Should include Bob's reply from the TUI

## Test Scenario: Bob (CLI) → Alice (TUI)

### Step 1: Start Alice's TUI
In **Terminal 1**, start Alice's TUI:
```bash
cd dialog_tui
cargo run -- --key alice
```

### Step 2: Bob Creates Group and Invites Alice
In **Terminal 2**, Bob creates a group:
```bash
cargo run -- get-pubkey --key alice  # Get Alice's public key first
cargo run -- create-group --key bob --name "Bob's Group" --counterparty <ALICE_PUBLIC_KEY>
```

### Step 3: Alice Accepts via TUI
In **Terminal 1** (Alice's TUI):
- Check for invitation in TUI
- Accept the invitation using TUI commands

### Step 4: Bob Sends Message
In **Terminal 2**, Bob sends a message:
```bash
cargo run -- send-message --key bob --group-id <GROUP_ID> --message "Hello Alice from Bob via CLI!"
```

### Step 5: Alice Replies via TUI
In **Terminal 1** (Alice's TUI):
- Read Bob's message
- Reply using the TUI interface

### Step 6: Bob Gets Reply
In **Terminal 2**, Bob retrieves messages:
```bash
cargo run -- get-messages --key bob --group-id <GROUP_ID>
```

## Advanced Test: Create-Group-and-Send
Test the combined command:
```bash
cargo run -- create-group-and-send --key alice --name "Quick Test" --counterparty <BOB_PUBLIC_KEY> --message "This is a test message!"
```

This should:
1. Create a group
2. Invite the counterparty
3. Send the message immediately

## Troubleshooting

### Common Issues:

1. **"Key not found" errors**: Make sure `.env.local` exists and has the correct keys
2. **"Connection failed" errors**: Ensure the relay is running at `ws://localhost:8080`
3. **"Group not found" errors**: Make sure to use the correct group ID from the create-group output
4. **TUI not showing messages**: Check that the TUI is properly connected (status bar shows "Connected")

### Debug Commands:

1. **List all key packages**:
   ```bash
   cargo run -- list
   ```

2. **Check pending invites**:
   ```bash
   cargo run -- list-invites --key alice
   cargo run -- list-invites --key bob
   ```

3. **Get public keys**:
   ```bash
   cargo run -- get-pubkey --key alice
   cargo run -- get-pubkey --key bob
   ```

### Expected Public Keys:
- Alice: `7c2e3f5a8b9c1d4e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e`
- Bob: `75427ab8309aad26beea8142edf427674e4544604ae4dc5045108ad21fc8a0db`

## Success Criteria

✅ **Complete Success** when:
1. Both TUI and CLI can send/receive messages
2. Group creation works from CLI
3. Invitations are properly delivered
4. Messages are encrypted/decrypted correctly
5. Both interfaces show consistent conversation state

## Next Steps
After successful interop testing, proceed to:
- Dialog CLI integration with dialog_lib
- Performance testing with multiple users
- UI/UX improvements based on testing feedback