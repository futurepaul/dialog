# Nostr Dialog - Secure Messaging Infrastructure

A Rust monorepo for building secure Nostr messaging applications with authenticated relays and the new Whitenoise DM protocol.

## Overview

This project implements a complete Nostr messaging ecosystem with:
- **NIP-42 Authentication**: Relay-level authentication for secure connections
- **NIP-70 Protected Events**: Private event support for secure messaging
- **Negentropy Sync**: Efficient set reconciliation protocol
- **Whitenoise Protocol**: Next-generation encrypted direct messaging (not NIP-04/NIP-17)

## Project Structure

```
.
├── dialog_client/      # Core Rust library for Nostr client functionality
├── dialog_relay/       # Authenticated Nostr relay implementation
├── dialog_cli/         # Command-line test client
├── dialog_deploy/      # Infrastructure management for AI agents
├── ios/                # iOS app using UniFFI bindings
├── Cargo.toml          # Workspace configuration
├── CLAUDE.md           # AI assistant context
└── .cursorrules        # Cursor IDE configuration
```

## Components

### dialog_client
Core library providing:
- NIP-42 authentication client
- Negentropy sync implementation
- Whitenoise DM protocol support
- UniFFI bindings for iOS integration

### dialog_relay
Nostr relay built with rust-nostr's relay builder:
- Requires NIP-42 authentication for all connections
- Supports NIP-70 protected events
- Implements Negentropy sync
- Custom policies for dialog management

### dialog_cli
Testing utility for protocol verification:
- Encrypted DM communication between instances
- Protocol compliance testing
- Development and debugging tool

### dialog_deploy
Future infrastructure for on-demand AI agent deployment:
- Agent lifecycle management
- User-triggered deployments
- Resource orchestration

### iOS App
Native iOS application:
- Swift/SwiftUI interface
- UniFFI integration with dialog_client
- Full protocol support

## Getting Started

### Prerequisites
- Rust 1.70+ with cargo
- iOS development: Xcode 14+
- UniFFI tools for iOS bindings

### Building

```bash
# Build all Rust components
cargo build --workspace

# Build specific component
cargo build -p dialog_client

# Run tests
cargo test --workspace
```

### Testing with CLI

```bash
# Terminal 1: Start relay
cargo run -p dialog_relay

# Terminal 2: Start first client
cargo run -p dialog_cli -- --user alice

# Terminal 3: Start second client
cargo run -p dialog_cli -- --user bob
```

## Development

See [CLAUDE.md](CLAUDE.md) for detailed development guidelines and architecture decisions.

## Security

This project prioritizes security:
- All relay connections require authentication
- Whitenoise protocol for end-to-end encryption
- No logging of private keys or sensitive data
- Constant-time cryptographic operations

## License

[License details to be determined]

## Contributing

[Contribution guidelines to be added]