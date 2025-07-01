# dialog_client

Core Nostr client logic supporting NIP-42 authentication, Negentropy sync, and the new Whitenoise DM protocol.

## 🎉 Current Status: Basic Client Working!

✅ **WORKING**: Basic Nostr operations (publish/fetch notes) functional  
✅ **TESTED**: E2E integration with dialog_relay and dialog_cli working  
✅ **ARCHITECTURE**: Clean DialogClient API with nostr-sdk integration  
✅ **READY**: Foundation ready for authentication and encryption features  

## Implementation Goals

Build a production-ready Nostr client library with efficient sync, secure storage, and encrypted group messaging capabilities. This library will be used by both dialog_cli and the iOS app (via UniFFI).

## ✅ COMPLETED: Basic Client Foundation

### Working Core Features
- [x] ✅ **DialogClient struct** - main client interface implemented
- [x] ✅ **Key generation** - automatic key creation for each client instance
- [x] ✅ **Relay connections** - connect_to_relay() working with WebSocket URLs
- [x] ✅ **Note publishing** - publish_note() creates and sends kind-1 events
- [x] ✅ **Note retrieval** - get_notes() fetches events with filters
- [x] ✅ **Error handling** - anyhow::Result throughout with proper error propagation
- [x] ✅ **CLI integration** - used successfully by dialog_cli commands
- [x] ✅ **E2E testing** - validated through comprehensive test scenarios

### Current API (Working)
```rust
pub struct DialogClient {
    client: Client,
    keys: Keys,
}

impl DialogClient {
    pub fn new() -> Result<Self>;                                    // ✅ Working
    pub async fn connect_to_relay(&self, url: &str) -> Result<()>;   // ✅ Working  
    pub async fn publish_note(&self, content: &str) -> Result<()>;   // ✅ Working
    pub async fn get_notes(&self, limit: usize) -> Result<Vec<Event>>; // ✅ Working
}
```

## 🔄 TODO: Advanced Features

### Client Architecture Enhancement (Based on Whitenoise patterns)
- [ ] Set up modular architecture:
  ```
  src/
  ├── lib.rs              # Main entry point with re-exports
  ├── types.rs            # Core data structures  
  ├── error.rs            # Centralized error handling
  ├── client/             # Core client logic
  │   ├── mod.rs
  │   ├── config.rs       # Client configuration
  │   └── manager.rs      # Client lifecycle management
  ├── storage/            # Local storage management
  │   ├── mod.rs
  │   ├── lmdb.rs         # LMDB integration
  │   └── migrations.rs   # Database migrations
  ├── sync/               # Negentropy sync implementation
  │   ├── mod.rs
  │   └── negentropy.rs   # Sync protocol
  ├── messaging/          # MLS encrypted messaging
  │   ├── mod.rs
  │   ├── groups.rs       # Group management
  │   └── encryption.rs   # MLS integration
  └── nostr/              # Nostr protocol handling
      ├── mod.rs
      ├── events.rs       # Event creation/parsing
      └── subscriptions.rs # Real-time event handling
  ```

### Storage Implementation (LMDB)
- [ ] Integrate LMDB with nostr-sdk patterns:
  ```rust
  let database = NostrLMDB::open("./db/dialog-client")?;
  let client = ClientBuilder::default()
      .signer(keys.clone())
      .database(database)
      .build();
  ```
- [ ] Implement database migrations for schema evolution
- [ ] Add event indexing for efficient queries
- [ ] Configure connection pooling and WAL mode
- [ ] Implement automatic old event cleanup

### Negentropy Sync Integration
- [ ] Set up efficient sync with progress tracking:
  ```rust
  let sync_opts = SyncOptions::default()
      .progress_sender(progress_tx);
  client.sync_with_opts(sync_opts).await?;
  ```
- [ ] Implement sync filters for different event types
- [ ] Add background sync with configurable intervals
- [ ] Handle sync errors with exponential backoff retry
- [ ] Support multi-relay sync with conflict resolution

### NIP-42 Authentication
- [ ] Implement automatic AUTH challenge handling
- [ ] Maintain authentication state per relay connection
- [ ] Handle authentication errors and re-auth flows
- [ ] Support ephemeral event signing for AUTH responses
- [ ] Add authentication status tracking and callbacks

### Whitenoise MLS Integration (Based on production patterns)
- [ ] Integrate nostr-mls for secure group messaging:
  ```rust
  let mls_storage = NostrMlsStorage::new(database.clone());
  let mls_client = NostrMls::new(keys.clone(), mls_storage).await?;
  ```
- [ ] Implement group lifecycle management:
  - Create groups with member key packages
  - Add/remove members with proper MLS proposals
  - Handle group evolution and key rotation
- [ ] Add secure message handling:
  - Encrypt group messages with current group key
  - Decrypt received messages and update group state
  - Handle welcome messages for new group members
- [ ] Support gift-wrapped events for private delivery (NIP-59)
- [ ] Implement key package management and discovery

### Enhanced Error Handling (Following Whitenoise patterns)
- [x] ✅ **Basic error handling** - anyhow::Result throughout
- [ ] Create comprehensive error types with thiserror:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum DialogClientError {
      #[error("Nostr client error: {0}")]
      NostrClient(#[from] nostr_sdk::client::Error),
      #[error("Database error: {0}")]
      Database(#[from] NostrLMDBError),
      #[error("MLS error: {0}")]
      Mls(#[from] NostrMlsError),
      #[error("Sync error: {0}")]
      Sync(String),
      #[error("Authentication error: {0}")]
      Auth(String),
  }
  ```
- [ ] Implement From traits for seamless error conversion
- [ ] Add Result type alias for convenience
- [ ] Include context in error messages for debugging

### Enhanced Public API Design
- [ ] Expand async-first public interface:
  ```rust
  pub struct DialogClient {
      // Internal fields
  }
  
  impl DialogClient {
      pub async fn new(config: ClientConfig) -> Result<Self>;        // Enhance existing
      // ✅ WORKING: pub async fn connect_to_relay(&self, url: &str) -> Result<()>;
      pub async fn sync(&self) -> Result<SyncProgress>;              // Add
      pub async fn create_group(&self, members: Vec<PublicKey>) -> Result<GroupId>; // Add
      pub async fn send_group_message(&self, group_id: &GroupId, content: &str) -> Result<EventId>; // Add
      pub async fn get_group_messages(&self, group_id: &GroupId) -> Result<Vec<Message>>; // Add
  }
  ```
- [ ] Add event callbacks for real-time message handling
- [ ] Support configuration options for different environments
- [ ] Implement graceful shutdown with resource cleanup

### UniFFI Integration Preparation
- [ ] Design FFI-friendly types and interfaces
- [ ] Add necessary derives and attributes for code generation
- [ ] Create wrapper types for complex Rust types
- [ ] Plan async callback patterns for iOS integration
- [ ] Document UniFFI usage patterns and examples

### Testing Infrastructure
- [x] ✅ **Basic E2E testing** - working with dialog_cli and relay
- [ ] Set up comprehensive test suite:
  - Unit tests for each module
  - Integration tests with MockRelay
  - End-to-end tests with real relays
- [ ] Add property-based testing for sync algorithms
- [ ] Create test utilities for MLS group scenarios
- [ ] Implement test fixtures for different client configurations

## Architecture Decisions

Following Whitenoise production patterns:
- ✅ **Basic Design**: Core client structure working with nostr-sdk
- **Modular Design**: Separate concerns into focused modules (next)
- **Error-First**: Comprehensive error handling with context (next)
- **Async Throughout**: Full async/await support with proper resource management (next)
- **Storage Strategy**: LMDB for performance with embedded migrations (next)
- **Security Focus**: MLS for group encryption, proper key management (next)
- **Cross-Platform**: Design for both native Rust and UniFFI usage (next)