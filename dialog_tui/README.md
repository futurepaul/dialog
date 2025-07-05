# Dialog TUI

A terminal user interface for encrypted Nostr MLS messaging.

## Usage

```bash
# Run as Alice (reads ALICE_SK_HEX from .env.local)
cargo run --bin dialog_tui -- alice

# Run as Bob (reads BOB_SK_HEX from .env.local)
cargo run --bin dialog_tui -- bob

# Run with a specific hex private key
cargo run --bin dialog_tui -- <64-char-hex-key>

# Test mode (initializes storage without UI)
cargo run --bin dialog_tui -- --test alice
cargo run --bin dialog_tui -- --test bob
```

## Environment Variables

The app looks for `.env.local` file in the current directory or parent directories. It should contain:

```
ALICE_SK_HEX=<alice-private-key-hex>
BOB_SK_HEX=<bob-private-key-hex>
```

## MLS Messaging Flow

1. **Start the app** with your secret key (alice or bob)
2. **Publish your keypackage** (Ctrl-P) - This allows others to invite you to groups
3. **Add a contact** - Navigate to Contacts pane, press Enter, input their public key and a petname
4. **Create a conversation** - Navigate to Conversations pane, press Enter, select a contact
   - This creates an MLS group and sends a welcome message to the contact
5. **Send messages** - Select a conversation, switch to Input pane (Tab), type and press Enter

## Debug Information

The app logs debug information to `/tmp/dialog_tui.log` for troubleshooting.

## Features

- **Three-pane layout**: Contacts (top left), Conversations (bottom left), Chat (right)
- **Contact management**: Add contacts with petnames
- **MLS group creation**: Create encrypted conversations with contacts
- **End-to-end encryption**: All messages are encrypted using MLS
- **Per-user storage**: Each user gets isolated SQLite storage under `~/.local/share/dialog_tui/<pubkey>/`

## Keyboard Shortcuts

- `Tab`: Switch between panes
- `j/k`: Navigate up/down in active pane
- `Enter`: Context-sensitive action (Add contact, Create conversation, Send message)
- `Ctrl-P`: Publish keypackage
- `F1`: Help
- `Ctrl-Q`: Quit
- `Esc`: Cancel dialogs