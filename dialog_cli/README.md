# dialog_cli

Test client using dialog_client library. Enables encrypted DMs between two dialog_cli clients using the MLS-based protocol.

## 🎉 Current Status: Basic CLI Fully Working!

✅ **WORKING**: All basic commands functional with dialog_relay  
✅ **TESTED**: E2E tests pass - publish, fetch, and test commands working  
✅ **INTEGRATION**: Seamless integration with dialog_relay on port 7979  
✅ **COMMANDS**: `publish`, `fetch`, and `test` commands all operational  

## Implementation Goals

Create a comprehensive testing utility for validating NIP-42 authentication, NIP-70 protected events, Negentropy sync, and MLS-based encrypted messaging end-to-end.

## ✅ COMPLETED: Basic CLI Functionality

### Working Commands
- [x] ✅ **publish** - `cargo run -p dialog_cli publish "message" --relay ws://127.0.0.1:7979`
- [x] ✅ **fetch** - `cargo run -p dialog_cli fetch --limit 5 --relay ws://127.0.0.1:7979`  
- [x] ✅ **test** - `cargo run -p dialog_cli test --message "test" --relay ws://127.0.0.1:7979`
- [x] ✅ **Relay connection handling** - proper WebSocket connection management
- [x] ✅ **Error handling** - graceful error reporting and recovery
- [x] ✅ **E2E testing** - validated through comprehensive test script

### Basic Client Setup  
- [x] ✅ **dialog_client integration** - using core library for Nostr operations
- [x] ✅ **Command-line arguments** - clap-based argument parsing
- [x] ✅ **Relay configuration** - --relay flag support with default ws://127.0.0.1:7979
- [x] ✅ **Key generation** - automatic key generation per CLI instance

## 🔄 TODO: Advanced Testing Features

### NIP-42 Authentication Testing
- [ ] Test AUTH challenge/response flow with dialog_relay
- [ ] Verify authentication persistence across reconnections
- [ ] Handle authentication failures and retry logic
- [ ] Test multiple clients authenticating simultaneously

### NIP-70 Protected Events Testing
- [ ] Publish protected events with `["-"]` tag
- [ ] Verify only authenticated clients can publish protected events
- [ ] Test rejection of protected events from wrong pubkey
- [ ] Validate protected event access control on relay

### Negentropy Sync Validation
- [ ] Test efficient sync between multiple clients
- [ ] Verify event deduplication across syncs
- [ ] Test sync with different relay configurations
- [ ] Measure sync performance and bandwidth usage
- [ ] Test partial sync scenarios and error recovery

### MLS-based Group Messaging
- [ ] Create encrypted groups between CLI instances
- [ ] Test group member addition/removal
- [ ] Verify end-to-end encryption of group messages
- [ ] Test group message ordering and delivery
- [ ] Handle group evolution and key rotation scenarios

### Interactive CLI Interface
- [ ] Implement enhanced command-line interface for testing:
  ```
  dialog_cli alice
  > connect wss://localhost:7979
  > auth
  > create-group bob charlie
  > send-message "Hello encrypted group!"
  > list-groups
  > list-messages group_id
  > leave-group group_id
  ```
- [ ] Add real-time message display
- [ ] Support multiple concurrent operations
- [ ] Provide status indicators for sync and auth

### Advanced Test Scenarios
- [ ] **Two-Client Communication**:
  - Start alice and bob clients
  - Authenticate both to relay
  - Create group between them
  - Exchange encrypted messages
- [ ] **Group Messaging**:
  - Start 3+ clients
  - Create group with all members
  - Test message delivery to all participants
  - Test member addition/removal
- [ ] **Sync Testing**:
  - Client goes offline
  - Other clients send messages
  - Original client comes back online
  - Verify all messages synced correctly
- [ ] **Protected Events**:
  - Test publishing protected events
  - Verify relay access control
  - Test cross-client visibility

### Performance and Reliability Testing
- [ ] Add metrics collection for:
  - Message latency
  - Sync performance
  - Authentication time
  - Group operation duration
- [ ] Implement stress testing modes:
  - High message volume
  - Rapid group membership changes
  - Network disconnection/reconnection
- [ ] Add automated test suites for CI/CD

### Error Handling and Edge Cases
- [x] ✅ **Basic error handling** - network errors and invalid responses
- [ ] Test network disconnection scenarios
- [ ] Handle relay unavailability gracefully
- [ ] Test invalid message formats
- [ ] Verify error reporting and recovery
- [ ] Test concurrent operations on same group

### Integration with dialog_relay
- [x] ✅ **Local relay testing** - working with dialog_relay instance
- [ ] Verify all relay policy enforcement
- [ ] Test relay-specific features (rate limiting, etc.)
- [ ] Validate end-to-end security properties

## Command Line Interface

### Current Working Commands
```bash
# Publish a note
cargo run -p dialog_cli publish "Hello Nostr!" --relay ws://127.0.0.1:7979

# Fetch recent notes  
cargo run -p dialog_cli fetch --limit 10 --relay ws://127.0.0.1:7979

# Test connectivity
cargo run -p dialog_cli test --message "Connection test" --relay ws://127.0.0.1:7979
```

### Future Enhanced Interface
```bash
# Start different test users
dialog_cli --user alice --relay ws://localhost:7979
dialog_cli --user bob --relay ws://localhost:7979

# Automated test scenarios
dialog_cli --test two-client-messaging
dialog_cli --test group-messaging --participants 5
dialog_cli --test sync-stress --duration 60s
dialog_cli --test protected-events
```

## Test Architecture

The CLI serves as:
1. ✅ **Basic Testing Tool**: Manual validation of core functionality
2. 🔄 **Automated Test Suite**: Scripted validation of protocol implementations (next)
3. 🔄 **Benchmarking Tool**: Performance measurement and optimization (next) 
4. ✅ **Integration Validator**: End-to-end system verification

This comprehensive testing approach ensures all components work together correctly and provides confidence in the security and reliability of the messaging system.