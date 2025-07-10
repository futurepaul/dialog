# Whitenoise-Dialog Interoperability Integration

This directory contains comprehensive integration tests and automation for achieving true interoperability between **whitenoise** and **dialog_tui** nostr-mls clients.

## Overview

The integration test suite implements the scenarios outlined in `../whitenoise_interop_prd.md` using ht-mcp automation to control dialog_tui interactions and coordinate with whitenoise integration tests.

## Architecture

```
integration/
├── src/
│   ├── ht_mcp_automation.rs      # ht-mcp client for dialog_tui control
│   ├── whitenoise_interop.rs     # Core interop test scenarios
│   ├── test_scenarios.rs         # Complete test scenario orchestration
│   ├── whitenoise_coordination.rs # Functions for whitenoise integration_test.rs
│   ├── welcome_compatibility.rs   # Enhanced welcome message processing
│   └── lib.rs                    # Main library interface
├── tests/
│   └── whitenoise_integration_test.rs # Integration tests for whitenoise
├── Cargo.toml                    # Dependencies (dialog_lib + whitenoise)
└── README.md                     # This file
```

## Key Components

### 1. ht-mcp Automation (`ht_mcp_automation.rs`)

Provides automated control of dialog_tui via ht-mcp sessions:

- Create dialog_tui sessions with specific keys and relay configurations
- Automate setup (connect, publish key packages, get pubkey)
- Simulate user interactions (accept invites, send messages, create groups)
- Take snapshots and wait for specific text patterns
- Clean session management

### 2. Test Scenarios (`test_scenarios.rs`)

Comprehensive test scenario orchestration:

- **Complete Interop Test**: Full round-trip testing both directions
- **Stress Testing**: Rapid message exchange under load
- **Error Recovery**: Network issues and reconnection scenarios
- **Coordination Helpers**: Bridge between whitenoise and dialog_tui automation

### 3. Whitenoise Coordination (`whitenoise_coordination.rs`)

Functions designed to be integrated into whitenoise's `integration_test.rs`:

```rust
// Get dialog_tui ready for whitenoise invitation
let dialog_pubkey = get_dialog_tui_pubkey_for_whitenoise("alice").await?;

// Create whitenoise group with dialog_tui member
let group_id = whitenoise.create_group(&account, vec![dialog_pubkey], admins, config).await?;

// Wait for dialog_tui to accept and join
wait_for_dialog_tui_to_join_group(&group_id, &dialog_pubkey).await?;

// Verify message delivery both ways
let responses = wait_for_dialog_tui_response(&group_id).await?;
```

### 4. Welcome Compatibility (`welcome_compatibility.rs`)

Enhanced welcome message processing for improved compatibility:

- Process both gift-wrapped (whitenoise) and direct (dialog_tui) welcome formats
- Dual-format welcome sending for maximum compatibility
- Validation and error handling for different welcome types
- Integration helpers for dialog_lib enhancement

## Test Scenarios

### Scenario 1: Whitenoise Creates → Dialog Joins

1. Setup dialog_tui via ht-mcp automation
2. Extract dialog_tui pubkey for whitenoise
3. Whitenoise creates group and invites dialog_tui
4. Dialog_tui accepts invitation automatically
5. Bi-directional message exchange verification

### Scenario 2: Dialog Creates → Whitenoise Joins

1. Setup dialog_tui via ht-mcp automation  
2. Dialog_tui creates group and invites whitenoise
3. Whitenoise accepts invitation (via integration test)
4. Message exchange verification

### Scenario 3: Stress Testing

- Rapid message bursts from both clients
- Concurrent group operations
- Network recovery scenarios
- Long-running stability testing

## Usage

### Running Integration Tests

```bash
# Run complete test suite
cd integration
cargo run

# Run specific test scenarios
cargo test --test whitenoise_integration_test
```

### Integrating with Whitenoise

Copy functions from `whitenoise_coordination.rs` into whitenoise's `integration_test.rs`:

```rust
// In whitenoise/tests/integration_test.rs
use whitenoise_dialog_integration::whitenoise_coordination::*;

#[tokio::test]
async fn test_dialog_tui_interop() -> Result<()> {
    // Use coordination functions
    let dialog_pubkey = get_dialog_tui_pubkey_for_whitenoise("alice").await?;
    // ... rest of test
}
```

### Environment Setup

Required infrastructure:

```bash
# Terminal 1: Start whitenoise relays
cd ~/dev/heavy/whitenoise
docker compose up  # Ports 7777, 8080

# Terminal 2: Start backup relay
cd ~/dev/heavy/denoise
nak serve --verbose  # Port 10547

# Terminal 3: Run integration tests
cd ~/dev/heavy/denoise/integration
cargo run
```

## Key Features

### Automated Dialog_TUI Control

- ht-mcp session management with proper cleanup
- Interactive command simulation (navigation, selection)
- Real-time text pattern waiting and verification
- Snapshot capture for debugging

### Whitenoise Integration

- Direct integration with existing whitenoise test infrastructure
- Minimal changes required to whitenoise codebase
- Coordination functions designed for easy integration
- Proper error handling and timeouts

### Welcome Message Enhancement

- Dual-format welcome processing (gift-wrapped + direct)
- Compatibility validation and error recovery
- Enhanced subscription filters for multiple formats
- Integration hooks for dialog_lib enhancement

### Comprehensive Testing

- End-to-end scenario coverage
- Performance and stress testing
- Error recovery and network issue simulation
- Cross-client verification and validation

## Configuration

### Relay Configuration

Tests use multiple relay configurations for robustness:

- Primary: `ws://localhost:8080,ws://localhost:7777` (whitenoise relays)
- Backup: `ws://localhost:10547` (nak serve)

### Test Keys

- `alice`: Primary test user for whitenoise→dialog scenarios
- `bob`: Secondary test user for dialog→whitenoise scenarios
- `charlie`: Stress testing user
- `*_dialog`: Dialog_tui specific test users

## Debugging

### Logging

All modules use structured logging:

```bash
RUST_LOG=debug cargo run  # Detailed logging
RUST_LOG=whitenoise_dialog_integration=trace cargo test  # Module-specific
```

### ht-mcp Sessions

List active sessions:
```bash
ht list-sessions
```

Take manual snapshots:
```bash
ht take-snapshot <session-id>
```

### Message Verification

The test suite captures and validates:

- Message delivery timestamps
- MLS state synchronization
- Group membership consistency
- Welcome message processing

## Contributing

When adding new test scenarios:

1. Add scenario functions to `test_scenarios.rs`
2. Add corresponding whitenoise coordination to `whitenoise_coordination.rs`
3. Update the main test runner in `lib.rs`
4. Add integration tests to `tests/whitenoise_integration_test.rs`

## Future Enhancements

- [ ] Docker-based test environment for CI
- [ ] Performance metrics collection
- [ ] Automated regression testing
- [ ] Multi-relay synchronization testing
- [ ] Member management (add/remove) testing
- [ ] Admin permission compatibility testing