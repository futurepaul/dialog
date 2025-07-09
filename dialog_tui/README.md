# Dialog TUI

A modern terminal user interface for secure, end-to-end encrypted messaging using the Nostr-MLS protocol. Inspired by Claude Code's clean, command-driven interface.

## Phase 1 Complete ✅

We've successfully implemented the foundation of Dialog TUI with:

- **Basic Ratatui Application Structure**: Clean separation of concerns with app state, UI rendering, and theme management
- **Command Input System**: Full command input with history navigation
- **Welcome Screen**: Claude Code-style welcome screen with project info
- **Status Bar**: Shows connection status, contact count, and current state
- **Claude Code Theme**: Dark theme matching Claude Code's aesthetic
- **Command Parsing**: Basic command recognition and mode switching
- **Multiple UI Modes**: Support for different screens (conversations, contacts, invites, etc.)
- **Sub-command UI Patterns**: 
  - Searchable lists (conversations, contacts)
  - Simple selections (invites, help)
  - Confirmation dialogs (keypackage, quit)

## Running the Application

```bash
cargo run
```

## Available Commands

- `/help` - Show available commands
- `/add <pubkey|nip05>` - Add a new contact
- `/new` - Start a new conversation
- `/keypackage` - Publish your key package
- `/invites` - View pending invitations
- `/contacts` - List all contacts
- `/conversations` - List active conversations
- `/clear` - Clear current conversation
- `/quit` - Exit the application

## Keyboard Shortcuts

- `/` - Open command input
- `Esc` - Cancel/go back
- `Ctrl+C` - Force quit
- `Up/Down` - Navigate command history
- `Enter` - Submit command

## Next Steps (Phase 2)

- Implement command palette with fuzzy search
- Add mock data generation for testing
- Enhance navigation in sub-command screens
- Add more keyboard shortcuts
- Implement proper scrolling in lists