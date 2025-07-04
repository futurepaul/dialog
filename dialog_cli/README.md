# Dialog CLI

This CLI provides tools for interacting with the Nostr Messaging Layer Security (MLS) protocol.

## Quick Setup

### 1. Start a Local Relay

```bash
# In a separate terminal, start a local relay
nostr-relay
```

### 2. Setup Environment

Use the justfile for easy setup:

```bash
# One command setup
just setup
```

Or manually create `.env.local`:

```bash
# Generate the .env.local file with test keys
cat > .env.local << 'EOF'
ALICE_SK_HEX=7c2e3f5a8b9c1d4e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e
BOB_SK_HEX=1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b
EOF
BOB_PK=$(cargo run -p dialog_cli -- get-pubkey --key bob)
echo "BOB_PK_HEX=$BOB_PK" >> .env.local
```

### 3. Add Shell Aliases (Recommended)

Add these to your `~/.bashrc` or `~/.zshrc` for the best experience:

```bash
alias alice='f() { cargo run -p dialog_cli -- "$@" --key alice; }; f'
alias bob='f() { cargo run -p dialog_cli -- "$@" --key bob; }; f'
```

Then reload your shell:
```bash
source ~/.bashrc  # or ~/.zshrc
```

### 4. Test the Setup

```bash
# With aliases (easiest)
alice get-pubkey
bob get-pubkey

# Or with full commands
cargo run -p dialog_cli -- get-pubkey --key alice
cargo run -p dialog_cli -- get-pubkey --key bob
```

## Basic Testing Workflow

### Step 1: Publish Key Packages

```bash
# With aliases (recommended)
alice publish-key
bob publish-key

# Verify they're published
alice list  # or: cargo run -p dialog_cli -- list
```

### Step 2: Alice Creates a Group

```bash
# Get Bob's public key
BOB_PK=$(grep BOB_PK_HEX .env.local | cut -d '=' -f2)

# Alice creates a group and invites Bob
alice create-group --name "alice-and-bob" --counterparty $BOB_PK

# Note the GROUP_ID from the output (32 hex characters)
```

### Step 3: Complete the Conversation

```bash
# Replace <GROUP_ID> with the ID from Step 2
export GROUP_ID="<paste-group-id-here>"

# Alice sends first message
alice send-message --group-id $GROUP_ID --message "Hello Bob!"

# Bob checks for invites and accepts
bob list-invites
bob accept-invite --group-id $GROUP_ID

# Bob reads Alice's message
bob get-messages --group-id $GROUP_ID

# Bob replies
bob send-message --group-id $GROUP_ID --message "Hi Alice!"

# Alice reads Bob's reply
alice get-messages --group-id $GROUP_ID
```

## Complete Example (Copy-Paste Ready)

Here's a complete example using the recommended aliases:

```bash
# 1. Setup (run once)
just setup

# 2. Add aliases to your shell (copy from 'just aliases' output)
alias alice='f() { cargo run -p dialog_cli -- "$@" --key alice; }; f'
alias bob='f() { cargo run -p dialog_cli -- "$@" --key bob; }; f'

# 3. Publish keys
alice publish-key
bob publish-key

# 4. Create group (save the GROUP_ID from output)
BOB_PK=$(grep BOB_PK_HEX .env.local | cut -d '=' -f2)
alice create-group --name "test-chat" --counterparty $BOB_PK

# 5. Set GROUP_ID (replace with actual ID from step 4)
export GROUP_ID="your-group-id-here"

# 6. Alice sends message
alice send-message --group-id $GROUP_ID --message "Hello Bob! How are you?"

# 7. Bob accepts invite
bob accept-invite --group-id $GROUP_ID

# 8. Bob reads and replies
bob get-messages --group-id $GROUP_ID
bob send-message --group-id $GROUP_ID --message "Hi Alice! I'm doing great!"

# 9. Alice reads reply
alice get-messages --group-id $GROUP_ID
```

## Why Use Shell Aliases?

**Shell aliases are the recommended approach** because they:
- ✅ **Handle quotes and spaces perfectly** - No justfile limitations
- ✅ **Shorter commands** - `alice publish-key` vs `cargo run -p dialog_cli -- publish-key --key alice`
- ✅ **More readable** - Clear who is doing what
- ✅ **Work with any message** - Even complex messages with special characters

```bash
# This just works with aliases:
alice send-message --group-id $GROUP_ID --message "I love you sooooo much! 🥰"

# Would be painful with full commands:
cargo run -p dialog_cli -- send-message --key alice --group-id $GROUP_ID --message "I love you sooooo much! 🥰"
```

## Alternative Testing Methods

### Automated Tests (Fastest)

If you just want to verify everything works:

```bash
# Run all tests
just test

# Or manually:
cargo test -p dialog_cli
cargo test -p dialog_cli --test memory_storage_full_test
cargo test -p dialog_cli --test mls_simple
```

### Memory Storage (Development)

For development/debugging, use memory storage to avoid database state issues:

```bash
# With aliases
alice publish-key --memory-storage
alice create-group --name "test" --counterparty $BOB_PK --memory-storage

# Note: State is lost between commands with memory storage
```

## Key Management

### Using Your Own Keys

Replace the test keys in `.env.local` with your own:

```bash
# Generate new keys (use any Nostr tool)
# Or use hex keys directly:
cargo run -p dialog_cli -- get-pubkey --key <64-char-hex-key>
```

### Environment Variables

The CLI looks for these variables (in order):
1. `.env.local` (recommended)
2. `.env` 
3. Environment variables

```bash
# .env.local format
ALICE_SK_HEX=<64-char-hex-secret-key>
BOB_SK_HEX=<64-char-hex-secret-key>
BOB_PK_HEX=<64-char-hex-public-key>
```

## Convenience Commands

```bash
# With aliases (recommended)
alice list                    # List all key packages on relay
alice get-pubkey             # Get Alice's public key
bob get-pubkey               # Get Bob's public key

# Create group and send message in one step
alice create-group-and-send --name "test" --counterparty $BOB_PK --message "Hello!"

# Or with full commands
cargo run -p dialog_cli -- list
cargo run -p dialog_cli -- get-pubkey --key alice
cargo run -p dialog_cli -- create-group-and-send --key alice --name "test" --counterparty $BOB_PK --message "Hello!"
```

## Storage Options

### SQLite Storage (Default)
- **Pros**: Persistent state across CLI commands
- **Cons**: Can have state synchronization issues
- **Use**: Default behavior (no flag needed)
- **Best for**: Production use, persistent conversations

### Memory Storage
- **Pros**: Fast, no database files, no persistence issues
- **Cons**: State lost between CLI commands
- **Use**: Add `--memory-storage` flag to any command
- **Best for**: Development, testing individual commands, debugging

## Troubleshooting

**"Environment variable not found"**: Make sure `.env.local` exists in the project root with the correct variable names (`ALICE_SK_HEX`, `BOB_SK_HEX`, `BOB_PK_HEX`)

**"Failed to decrypt message"**: This often indicates MLS epoch synchronization issues. Try:
1. Use memory storage for testing: `--memory-storage`
2. Run automated tests to verify core functionality works
3. Clear data and restart: `rm -rf .dialog_cli_data`

**"Group not found"**: Make sure you're using the correct GROUP_ID format (32 hex chars for MLS Group ID, 64 hex chars for Nostr Group ID)

**"No key packages found"**: Ensure both parties have published key packages before creating groups: `alice list` or `cargo run -p dialog_cli -- list` 