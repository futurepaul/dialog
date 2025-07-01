# dialog_relay

Nostr relay supporting NIP-42 authenticated clients and Negentropy sync, built with rust-nostr's relay builder.

## 🎉 Current Status: Basic Relay Working!

✅ **WORKING**: Relay properly configured and running on port 7979  
✅ **TESTED**: E2E tests pass - can publish and retrieve notes via CLI  
✅ **LOGGING**: Debug-level logging with clean connection/event tracking  
✅ **SHUTDOWN**: Proper signal handling and graceful shutdown  

## Implementation Goals

The current goal is to build a secure relay that enforces authentication and supports protected events for private messaging.

## ✅ COMPLETED: Basic Infrastructure

### Basic Relay Configuration
- [x] ✅ **Fixed RelayBuilder configuration** - properly listens on port 7979
- [x] ✅ **Address and port binding** - RelayBuilder::default().addr(addr).port(7979)
- [x] ✅ **Signal handling** - graceful shutdown with Ctrl+C
- [x] ✅ **Debug logging** - comprehensive logging without trace spam
- [x] ✅ **E2E testing** - validated with publish/fetch operations via CLI

### Working Features
- [x] ✅ **Note storage and retrieval** - basic kind-1 events working
- [x] ✅ **WebSocket connections** - proper connection lifecycle management
- [x] ✅ **CLI integration** - dialog_cli can publish/fetch notes successfully
- [x] ✅ **Relay URL reporting** - shows ws://127.0.0.1:7979 on startup

## 🔄 TODO: Advanced Features

### NIP-42 Authentication
- [ ] Configure RelayBuilder with `nip42(Nip42Mode::Both)` to require auth for all operations
- [ ] Implement AUTH challenge/response flow
- [ ] Validate client authentication before accepting events
- [ ] Maintain authenticated client state per WebSocket connection

### NIP-70 Protected Events
- [ ] Detect events with `["-"]` tag in event validation pipeline
- [ ] Implement protected event policy:
  - Reject protected events from unauthenticated clients
  - Verify authenticated client pubkey matches event author pubkey
  - Only accept protected events after successful NIP-42 auth
- [ ] Add configuration option to enable/disable protected event support
- [ ] Return appropriate error responses for rejected protected events

### Enhanced Relay Builder Integration
- [ ] Upgrade to authenticated RelayBuilder configuration:
  ```rust
  RelayBuilder::default()
      .addr(addr)
      .port(7979)
      .nip42(Nip42Mode::Both)
      .write_policy(DialogWritePolicy::new())
      .query_policy(DialogQueryPolicy::new())
  ```
- [ ] Implement custom WritePolicy for dialog-specific validation
- [ ] Implement custom QueryPolicy for access control
- [ ] Configure rate limiting and connection limits

### Dialog-Specific Policies
- [ ] Create DialogWritePolicy struct implementing WritePolicy trait
- [ ] Add validation for dialog message event kinds
- [ ] Ensure all private dialog events include `["-"]` tag
- [ ] Implement DialogQueryPolicy for restricting queries to authenticated users

### Storage and Performance
- [ ] Configure appropriate database backend (SQLite or LMDB)
- [ ] Set connection limits and timeouts
- [ ] Implement event cleanup policies for old messages
- [ ] Add monitoring and logging for relay operations

### Testing and Integration
- [x] ✅ **Basic E2E testing** - comprehensive e2e_test.sh working
- [ ] Create integration tests with MockRelay
- [ ] Test NIP-42 auth flow with real clients
- [ ] Verify NIP-70 protected event handling
- [ ] Test with dialog_cli clients for authenticated operations

## Architecture Notes

Based on rust-nostr relay builder patterns:
- ✅ **Basic relay functionality** - publish/subscribe working
- Use policy system for fine-grained access control
- Leverage built-in NIP-42 and NIP-70 support  
- Implement custom business logic through policy traits
- Maintain stateless design where possible

## Running the Relay

```bash
# Start relay (will run until Ctrl+C)
cargo run -p dialog_relay

# Test basic functionality
cargo run -p dialog_cli publish "Test message" --relay ws://127.0.0.1:7979
cargo run -p dialog_cli fetch --limit 5 --relay ws://127.0.0.1:7979

# Run comprehensive E2E tests
./e2e_test.sh
```