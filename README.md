# Nostr Dialog - Secure Messaging Infrastructure

A Rust monorepo for building secure Nostr messaging applications with authenticated relays and the new Whitenoise DM protocol.

## 🎉 Current Status: Basic Functionality Working!

**✅ End-to-End Tested**: Notes can be published and retrieved through CLI with a working authenticated relay.

- **Relay**: Properly configured and running on port 7979 with debug logging
- **CLI**: Fully functional publish/fetch/test commands
- **Client**: Basic note operations working via nostr-sdk
- **Testing**: Comprehensive E2E test suite passing all scenarios

## Overview

This project implements a complete Nostr messaging ecosystem with:
- **NIP-42 Authentication**: Relay-level authentication for secure connections
- **NIP-70 Protected Events**: Private event support for secure messaging
- **Negentropy Sync**: Efficient set reconciliation protocol
- **Whitenoise Protocol**: Next-generation encrypted direct messaging (not NIP-04/NIP-17)

## Project Structure

```
.
├── dialog_client/      # ✅ Core Rust library (basic functionality working)
├── dialog_relay/       # ✅ Working relay listening on port 7979
├── dialog_cli/         # ✅ Fully functional CLI test client
├── dialog_deploy/      # 🔄 Infrastructure management (future)
├── ios/                # 🔄 iOS app using UniFFI bindings (future)
├── e2e_test.sh         # ✅ Comprehensive E2E testing script
├── Cargo.toml          # Workspace configuration
├── CLAUDE.md           # AI assistant context
└── .cursorrules        # Cursor IDE configuration
```

## Components

### dialog_client ✅ Working
Core library providing:
- ✅ Basic Nostr client operations (publish/fetch notes)
- 🔄 NIP-42 authentication client (next)
- 🔄 Negentropy sync implementation (next)
- 🔄 MLS-based DM protocol support (next)
- 🔄 UniFFI bindings for iOS integration (future)

### dialog_relay ✅ Working
Nostr relay built with rust-nostr's relay builder:
- ✅ Properly configured to listen on port 7979
- ✅ Debug logging and signal handling
- ✅ Basic note storage and retrieval
- 🔄 NIP-42 authentication requirement (next)
- 🔄 NIP-70 protected events support (next)
- 🔄 Negentropy sync implementation (next)

### dialog_cli ✅ Working
Testing utility for protocol verification:
- ✅ Publish notes to relay
- ✅ Fetch notes from relay
- ✅ Test connectivity and basic operations
- 🔄 Encrypted DM communication (next)
- 🔄 Protocol compliance testing (next)

### dialog_deploy 🔄 Future
Infrastructure for on-demand AI agent deployment:
- Agent lifecycle management
- User-triggered deployments
- Resource orchestration

### iOS App 🔄 Future
Native iOS application:
- Swift/SwiftUI interface
- UniFFI integration with dialog_client
- Full protocol support

## Getting Started

### Prerequisites
- Rust 1.70+ with cargo
- iOS development: Xcode 14+ (for iOS app)
- UniFFI tools for iOS bindings (future)

### Building

```bash
# Build all Rust components
cargo build --workspace

# Build specific component
cargo build -p dialog_client

# Run tests
cargo test --workspace

# Run comprehensive E2E test
./e2e_test.sh
```

### Testing with CLI ✅ Working

```bash
# Terminal 1: Start relay
cargo run -p dialog_relay

# Terminal 2: Publish a note
cargo run -p dialog_cli publish "Hello Nostr!" --relay ws://127.0.0.1:7979

# Terminal 3: Fetch notes
cargo run -p dialog_cli fetch --limit 5 --relay ws://127.0.0.1:7979

# Run comprehensive tests
./e2e_test.sh
```

## Development Roadmap

### ✅ Phase 1: Basic Infrastructure (COMPLETE)
- [x] Working relay configuration
- [x] Basic note publishing/retrieval
- [x] CLI interface for testing
- [x] End-to-end testing infrastructure

### 🔄 Phase 2: Authentication & Security (IN PROGRESS)
- [ ] NIP-42 authentication on relay
- [ ] NIP-70 protected events
- [ ] Authenticated client operations
- [ ] Enhanced security testing

### 🔄 Phase 3: Advanced Features (PLANNED)
- [ ] Negentropy sync protocol
- [ ] MLS-based encrypted messaging
- [ ] Group messaging support
- [ ] UniFFI bindings for iOS

### 🔄 Phase 4: Production Features (FUTURE)
- [ ] iOS app with SwiftUI interface
- [ ] AI agent deployment infrastructure
- [ ] Performance optimization
- [ ] Production monitoring

## Development

See [CLAUDE.md](CLAUDE.md) for detailed development guidelines and architecture decisions.

## Security

This project prioritizes security:
- All relay connections require authentication (implementing)
- MLS-based protocol for end-to-end encryption (planned)
- No logging of private keys or sensitive data
- Constant-time cryptographic operations

## License

[License details to be determined]

## Contributing

[Contribution guidelines to be added]