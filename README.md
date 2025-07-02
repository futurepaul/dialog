# Nostr Dialog - Secure Messaging Infrastructure

A Rust monorepo for building secure Nostr messaging applications with authenticated relays and the Whitenoise MLS protocol.

## 🎉 Current Status: MLS Group Messaging Ready!

**✅ Production-Ready MLS**: Complete MLS group messaging implementation using whitenoise library with iOS bindings.

- **Relay**: Properly configured and running on port 7979 with debug logging
- **CLI**: Fully functional publish/fetch/test commands
- **Client**: **✅ NEW**: Full MLS group messaging via whitenoise integration
- **iOS**: **✅ NEW**: UniFFI Swift bindings for native iOS app integration
- **Testing**: Comprehensive E2E test suite passing all scenarios

## Overview

This project implements a complete Nostr messaging ecosystem with:
- **✅ Whitenoise MLS**: Production-ready group messaging with end-to-end encryption
- **✅ UniFFI iOS Integration**: Native Swift bindings for cross-platform development
- **🔄 NIP-42 Authentication**: Relay-level authentication for secure connections (planned)
- **🔄 NIP-70 Protected Events**: Private event support for secure messaging (planned)
- **🔄 Negentropy Sync**: Efficient set reconciliation protocol (planned)

## Project Structure

```
.
├── dialog_client/      # ✅ Core Rust library with MLS group messaging
├── dialog_relay/       # ✅ Working relay listening on port 7979
├── dialog_cli/         # ✅ Fully functional CLI test client
├── dialog_deploy/      # 🔄 Infrastructure management (future)
├── dialog_ios/         # ✅ NEW: Swift Package with UniFFI bindings
├── ios/                # 🔄 iOS app consuming dialog_ios package
├── e2e_test.sh         # ✅ Comprehensive E2E testing script
├── generate_swift_bindings.sh  # ✅ NEW: Swift bindings generator
├── build_xcframework.sh        # ✅ NEW: iOS framework builder
├── Cargo.toml          # Workspace configuration
├── CLAUDE.md           # AI assistant context
└── .cursorrules        # Cursor IDE configuration
```

## Components

### dialog_client ✅ Production Ready
Core library providing:
- ✅ Basic Nostr client operations (publish/fetch notes)
- ✅ **NEW**: Complete MLS group messaging via whitenoise integration
- ✅ **NEW**: UniFFI bindings for iOS Swift integration
- ✅ **NEW**: Group creation, member management, encrypted messaging
- 🔄 NIP-42 authentication client (planned)
- 🔄 Negentropy sync implementation (planned)

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
- ✅ **NEW**: MLS group operations via whitenoise
- 🔄 Enhanced group management commands (planned)

### dialog_deploy 🔄 Future
Infrastructure for on-demand AI agent deployment:
- Agent lifecycle management
- User-triggered deployments
- Resource orchestration

### dialog_ios ✅ Ready for Integration
Swift Package with UniFFI bindings:
- ✅ **NEW**: Complete Swift API generated from Rust
- ✅ **NEW**: Native iOS data types (DialogClient, NoteData, EncryptedMessage)
- ✅ **NEW**: MLS group messaging functions exposed to Swift
- ✅ **NEW**: XCFramework build system for distribution

### iOS App 🔄 Integration Ready
Native iOS application framework:
- ✅ Basic SwiftUI interface structure
- ✅ **NEW**: Ready to consume dialog_ios Swift Package
- 🔄 MLS group messaging UI (next)
- 🔄 Complete protocol integration (next)

## Getting Started

### Prerequisites
- Rust 1.70+ with cargo
- iOS development: Xcode 14+ (for iOS app)
- ✅ **NEW**: UniFFI tools integrated (automated via scripts)

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

# Generate iOS Swift bindings
./generate_swift_bindings.sh

# Build XCFramework for iOS distribution
./build_xcframework.sh
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

### ✅ Phase 2: MLS Group Messaging (COMPLETE)
- [x] **NEW**: Whitenoise library integration
- [x] **NEW**: Complete MLS group messaging implementation
- [x] **NEW**: UniFFI bindings for iOS
- [x] **NEW**: Swift Package generation and build system
- [x] **NEW**: Cross-platform group operations

### 🔄 Phase 3: Authentication & Security (PLANNED)
- [ ] NIP-42 authentication on relay
- [ ] NIP-70 protected events
- [ ] Authenticated client operations
- [ ] Enhanced security testing

### 🔄 Phase 4: Advanced Features (PLANNED)
- [ ] Negentropy sync protocol
- [ ] iOS app complete integration with dialog_ios
- [ ] Enhanced group management UI
- [ ] Message persistence and offline sync

### 🔄 Phase 5: Production Features (FUTURE)
- [ ] AI agent deployment infrastructure
- [ ] Performance optimization
- [ ] Production monitoring
- [ ] Advanced security features

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