# Dialog TUI-CLI Interoperability Testing

This directory documents the manual testing process for verifying interoperability between dialog_tui and dialog_cli.

## Prerequisites

- Rust toolchain installed
- `nak` CLI tool installed (`cargo install nak`)
- Three terminal windows available
- No services running on port 10547

## Manual Test Process

### Terminal 1: Start Local Relay

```bash
nak serve --verbose
```

This starts a Nostr relay on `ws://localhost:10547`.

### Terminal 2: Start dialog_tui with Alice

```bash
cargo run --bin dialog_tui -- --key alice
```

Once the TUI starts:
1. Wait for "Connected to relay successfully!" message
2. Note that key packages are automatically published
3. Use `/pk` command to get Alice's public key (you'll need the hex format)

### Terminal 3: Use dialog_cli with Bob

```bash
# 1. Publish Bob's key packages
cargo run --bin dialog_cli -- publish-key --key bob

# 2. Create a group and invite Alice (replace with Alice's actual pubkey)
cargo run --bin dialog_cli -- create-group --key bob --name "Test Group" --counterparty <alice_pubkey_hex>

# Note the Group ID from the output

# 3. Send a message
cargo run --bin dialog_cli -- send-message --key bob --group-id <group_id> --message "Hello from CLI!"

# 4. Fetch messages to see the conversation
cargo run --bin dialog_cli -- get-messages --key bob --group-id <group_id>
```

### In dialog_tui (Terminal 2)

1. You should see "New group invitation received!"
2. Type `/invites` to view pending invites
3. Press Enter to accept the invite
4. You should automatically switch to the group and see Bob's message
5. Type a message and press Enter to send a response
6. Use `/fetch` if needed to ensure messages are synced

### Verify in dialog_cli (Terminal 3)

```bash
# Fetch messages again to see Alice's response
cargo run --bin dialog_cli -- get-messages --key bob --group-id <group_id>
```

## What This Tests

✅ **Group Creation**: CLI can create groups  
✅ **Invite Flow**: TUI receives and can accept invites  
✅ **Message Delivery**: Messages flow from CLI to TUI  
✅ **Bidirectional Communication**: TUI can send messages back to CLI  
✅ **Real-time Updates**: TUI shows messages as they arrive  

## Expected Output

A successful test shows:
- Bob's message appears in Alice's TUI
- Alice's response appears when Bob fetches messages
- Both clients remain stable and connected

## Notes

- The TUI uses ephemeral (memory) storage, so state is lost on restart
- The CLI uses SQLite storage in `.dialog_cli_data/`
- Both clients connect to the same relay (ws://localhost:10547)
- Both use the same default relay configuration from dialog_lib

## Troubleshooting

If messages aren't appearing:
- Ensure both clients show as connected to the relay
- Try `/fetch` in the TUI to manually sync messages
- Check that key packages were published successfully
- Verify you're using the correct group ID

## Future Work

Automated testing with ht-mcp would allow:
- Programmatic control of dialog_tui
- Repeatable test execution
- CI/CD integration

For now, this manual process verifies that interoperability works correctly.