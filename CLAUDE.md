# CLAUDE.md - Project Context for AI Assistants

## Project Overview
This is a Rust monorepo for building Nostr protocol applications with a focus on:
- NIP-42 authentication (AUTH command for relay authentication)
- Negentropy protocol for efficient set reconciliation
- Whitenoise DM protocol (new encrypted messaging, NOT NIP-04)

## Architecture

### Crates
- **dialog_client**: Core library for Nostr client functionality
  - NIP-42 authentication implementation
  - Negentropy sync protocol
  - Whitenoise DM protocol support
  - Will be exposed to iOS via UniFFI

- **dialog_relay**: Nostr relay implementation
  - Built with rust-nostr's relay builder
  - Supports only NIP-42 authenticated connections
  - Implements Negentropy sync

- **dialog_cli**: Command-line test client
  - Uses dialog_client library
  - For testing encrypted DM functionality between instances

- **dialog_deploy**: Infrastructure management
  - Future: AI agent deployment on demand
  - Manages deployment lifecycle triggered by user actions

### iOS Integration
- iOS app in `/ios` directory
- Uses UniFFI to bind to dialog_client Rust library
- Swift/SwiftUI application

## Key Technologies
- **rust-nostr**: Core Nostr protocol implementation
- **UniFFI**: Rust-to-Swift bindings for iOS
- **Negentropy**: Efficient set reconciliation protocol
- **Whitenoise**: New encrypted DM protocol (not NIP-04/NIP-17)

## Development Guidelines
1. Follow Rust best practices and idioms
2. Keep crates focused on single responsibilities
3. Share common dependencies via workspace
4. Test protocol implementations thoroughly
5. Document public APIs clearly

## Testing
- Use dialog_cli to test client-relay interactions
- Test Whitenoise DM encryption between cli instances
- Verify NIP-42 auth flows
- Test Negentropy sync efficiency

## Security Considerations
- NIP-42 auth is required for all relay connections
- Use Whitenoise protocol for DMs, NOT old protocols
- Validate all cryptographic operations
- Never log private keys or sensitive data