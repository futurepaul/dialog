# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
Rust monorepo for building secure Nostr messaging applications with authenticated relays and Whitenoise DM protocol. **Basic functionality now working end-to-end** - notes can be published and retrieved through CLI with working relay. Currently implementing NIP-42 authentication, NIP-70 protected events, and MLS-based group messaging.

## Common Development Commands

### Building and Testing
```bash
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build -p dialog_client
cargo build -p dialog_relay  
cargo build -p dialog_cli

# Run tests
cargo test --workspace

# Run comprehensive E2E test
./e2e_test.sh

# Add dependencies (use cargo add for latest versions)
cd dialog_client && cargo add nostr-sdk tokio anyhow
```

### Running Components
```bash
# Start relay (now working! Listens on port 7979)
cargo run -p dialog_relay

# Test CLI commands (fully functional)
cargo run -p dialog_cli publish "Hello world" --relay ws://127.0.0.1:7979
cargo run -p dialog_cli fetch --limit 5 --relay ws://127.0.0.1:7979
cargo run -p dialog_cli test --message "Test message" --relay ws://127.0.0.1:7979
```

## Architecture Overview

This is a **Rust workspace** with four main crates plus iOS integration:

### dialog_client (Core Library)
- **Purpose**: Platform-agnostic Nostr client library
- **Current State**: ✅ **WORKING** - Basic kind-1 note publish/fetch using nostr-sdk
- **Key API**: `DialogClient::new()`, `connect_to_relay()`, `publish_note()`, `get_notes()`
- **Dependencies**: nostr-sdk, tokio, anyhow, tracing
- **Next Steps**: LMDB storage, Negentropy sync, MLS encryption, UniFFI bindings

### dialog_relay (Relay Server)
- **Purpose**: Authenticated Nostr relay with custom policies
- **Current State**: ✅ **WORKING** - Properly configured RelayBuilder listening on port 7979
- **Features**: Debug logging, signal handling, proper startup/shutdown
- **Dependencies**: nostr-relay-builder, tokio, anyhow, tracing
- **Next Steps**: NIP-42 auth, NIP-70 protected events, custom policies

### dialog_cli (Testing Tool)
- **Purpose**: CLI for testing client-relay interactions
- **Current State**: ✅ **FULLY FUNCTIONAL** - All commands working with relay
- **Commands**: `publish`, `fetch`, `test` with --relay flag support
- **Default Relay**: ws://127.0.0.1:7979
- **Dependencies**: clap with derive feature, dialog_client
- **Testing**: ✅ E2E test script validates all functionality

### dialog_deploy (Infrastructure)
- **Purpose**: Future AI agent deployment management
- **Current State**: Empty template crate

## Key Implementation Patterns

### Error Handling
All crates use `anyhow::Result<T>` for error propagation with `?` operator.

### Client Architecture
`DialogClient` wraps nostr-sdk `Client` with generated `Keys`. Each instance creates new keys (no persistence yet).

### Relay Configuration ✅ FIXED
Uses `RelayBuilder::default().addr(addr).port(7979)` with proper socket binding and shutdown handling.

### CLI Design
Clap derive macros with subcommands. Each command handles its own relay connection and operations.

## Development Context

### Reference Implementation
- **Whitenoise**: https://github.com/parres-hq/whitenoise - Production patterns for rust-nostr + MLS
- **Dialog Lib**: ../dialog_lib/SUMMARY.md - Reference client architecture

### Protocol Specifications
- **NIP-42**: Authentication via AUTH command and ephemeral events
- **NIP-70**: Protected events with `["-"]` tag requiring authenticated publishers
- **Negentropy**: Set reconciliation for efficient sync
- **MLS/Whitenoise**: Group messaging protocol (not NIP-04/NIP-17)

### Current Status ✅ MAJOR PROGRESS
✅ **Basic note publishing/fetching working end-to-end**
✅ **Relay properly configured and running on port 7979**
✅ **CLI fully functional with all commands working**
✅ **E2E test suite passing all tests**
✅ **Debug logging implemented (no trace spam)**
🔄 **Next**: NIP-42 auth, NIP-70 protected events, MLS protocol

### Critical Files
- `dialog_client/src/lib.rs`: Core client implementation
- `dialog_relay/src/lib.rs`: ✅ Working relay setup with proper configuration
- `dialog_cli/src/main.rs`: ✅ Fully functional CLI interface
- `e2e_test.sh`: ✅ Comprehensive end-to-end testing
- Each crate's `README.md`: Updated TODO lists for next implementation phases

### Dependencies Strategy
- Always use `cargo add` for latest versions
- Follow whitenoise Cargo.toml for feature flags and versions
- Prefer workspace dependencies for shared crates