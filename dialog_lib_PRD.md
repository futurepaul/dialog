# Dialog Library - Product Requirements Document

## Executive Summary

Dialog Library (`dialog_lib`) is a self-contained, testable library that provides core application logic for both `dialog_tui` and `dialog_cli`. It serves as the abstraction layer between UI implementations and the underlying Nostr-MLS functionality, ensuring consistent behavior across different interfaces while maintaining clean separation of concerns.

## Vision

Create a robust, well-tested library that handles all dialog application logic, allowing UI implementations to focus purely on user interaction while the library manages contacts, conversations, messaging, and MLS operations.

## Architecture Goals

1. **Clean Separation**: UI concerns separated from business logic
2. **Testable**: Library can be unit tested independently
3. **Gradual Migration**: Start with mock implementation, evolve to real MLS
4. **Shared Logic**: Both TUI and CLI use the same core functionality
5. **Type Safety**: Use real Nostr-MLS types throughout

## Current State Analysis

Based on examination of the existing codebase:

### dialog_tui Mock Implementation
- Located in `dialog_tui/src/app.rs`
- Contains fake data generation for contacts, conversations, messages
- Implements command handling, message processing, UI state management
- Uses fake keys and group IDs (strings instead of real crypto types)
- All in the UI layer - needs extraction

### dialog_cli Real Implementation  
- Located in `dialog_cli/src/main.rs`
- Full MLS functionality with real storage (SQLite/Memory)
- Real key generation, group creation, message encryption
- Direct command-line interface to MLS operations
- No abstraction layer - tightly coupled to CLI

### Target Architecture
```
┌─────────────────┐    ┌─────────────────┐
│   dialog_tui    │    │   dialog_cli    │
│   (UI Layer)    │    │   (UI Layer)    │
└─────────────────┘    └─────────────────┘
         │                       │
         └───────────────────────┘
                     │
         ┌─────────────────────────┐
         │     dialog_lib         │
         │  (Business Logic)      │
         │                        │
         │  ┌─────────────────┐   │
         │  │  MockMlsService │   │
         │  │  (Phase 1)      │   │
         │  └─────────────────┘   │
         │                        │
         │  ┌─────────────────┐   │
         │  │  RealMlsService │   │
         │  │  (Phase 2+)     │   │
         │  └─────────────────┘   │
         └─────────────────────────┘
```

## Implementation Plan: 10 Steps

### ✅ Step 1: Extract Mock Code to dialog_lib **COMPLETED**
**Goal**: Move dialog_tui mock implementation into dialog_lib

**Completed Tasks**:
- ✅ Created `dialog_lib` crate with workspace dependencies (thiserror, tokio, nostr-mls, etc.)
- ✅ Moved `Contact`, `Conversation`, `AppMode`, `ConnectionStatus`, `AppResult` to `dialog_lib/src/types.rs`
- ✅ Extracted mock data generation (`setup_fake_data()`) to `dialog_lib/src/mock_service.rs`
- ✅ Created `MockMlsService` implementing `MlsService` trait with async methods
- ✅ Updated `dialog_tui` to use `dialog_lib::DialogLib` and `MockMlsService`
- ✅ Verified dialog_tui works exactly as before with identical behavior

**Files Created**:
- ✅ `dialog_lib/Cargo.toml` - Workspace member with proper dependencies
- ✅ `dialog_lib/src/lib.rs` - Public API with `DialogLib` facade
- ✅ `dialog_lib/src/service.rs` - `MlsService` trait definition
- ✅ `dialog_lib/src/mock_service.rs` - `MockMlsService` implementation with async/await
- ✅ `dialog_lib/src/types.rs` - Core types already using real `PublicKey` and `GroupId`
- ✅ `dialog_lib/src/errors.rs` - Custom error types using `thiserror`

**Success Criteria Met**:
- ✅ dialog_tui compiles and runs identically to previous behavior
- ✅ All mock data generation moved to dialog_lib with real cryptographic keys
- ✅ Clean separation achieved - no UI logic in dialog_lib
- ✅ Already using real `nostr-mls::PublicKey` and `GroupId` types throughout

### ✅ Step 2: Adopt Real Nostr-MLS Types in Mock **COMPLETED** 
**Goal**: Replace string-based IDs with real cryptographic types

**Completed Tasks**:
- ✅ `nostr-mls` dependencies already added to dialog_lib
- ✅ `Contact.pubkey` already uses real `PublicKey` (not String)
- ✅ `Conversation.group_id` already uses real `GroupId` (not String)
- ✅ Mock data generation already uses `Keys::generate()` for real keys
- ✅ Display logic already handles real key encoding (bech32/hex)

**Success Criteria Met**:
- ✅ Mock data uses real cryptographic identifiers throughout
- ✅ Type safety enforced at compile time with real types
- ✅ UI displays real bech32/hex encodings properly

### Step 3: Implement Real MLS Service ⭐ **READY TO START**
**Goal**: Create `RealMlsService` using actual Nostr-MLS operations

**Current Status**: Mock implementation complete with proper abstractions. Ready to implement real MLS functionality by extracting logic from `dialog_cli` and adapting it to the `MlsService` trait.

**Tasks**:
- Extract MLS logic from `dialog_cli/src/main.rs` into `dialog_lib/src/real_service.rs`
- Implement `RealMlsService` struct with real storage (Memory/SQLite)
- Add real key management, group creation, message encryption/decryption
- Implement all `MlsService` trait methods with actual Nostr-MLS calls
- Add configuration for storage backend selection (Memory vs SQLite)  
- Add relay connection management for Nostr events
- Implement proper error handling for MLS operations

**Key Implementation Details**:
```rust
// dialog_lib/src/real_service.rs
pub struct RealMlsService {
    storage: Box<dyn MlsStorage + Send + Sync>,
    client: Client,
    keys: Keys,
    relay_url: String,
}

impl RealMlsService {
    pub async fn new_memory() -> Result<Self> { /* Memory storage */ }
    pub async fn new_sqlite(db_path: &str) -> Result<Self> { /* SQLite storage */ }
}

#[async_trait::async_trait]  
impl MlsService for RealMlsService {
    async fn get_contacts(&self) -> Result<Vec<Contact>> {
        // Query real storage for contacts
    }
    
    async fn send_message(&self, group_id: &GroupId, content: &str) -> Result<()> {
        // Real MLS message encryption + Nostr relay publishing  
    }
    
    async fn create_conversation(&self, name: &str, participants: Vec<PublicKey>) -> Result<String> {
        // Real MLS group creation + key package exchange
    }
}
```

**Files to Create**:
- `dialog_lib/src/real_service.rs` - Real MLS implementation
- `dialog_lib/src/config.rs` - Configuration for storage/relay selection

**Success Criteria**:
- Real MLS operations working through same `MlsService` interface
- Both Memory and SQLite storage backends supported
- Message encryption/decryption functional
- Group creation and management operational

### ✅ Step 4: Create Service Abstraction **COMPLETED**
**Goal**: Define trait for MLS operations

**Completed Tasks**:
- ✅ Created `MlsService` trait with async operations in `dialog_lib/src/service.rs`
- ✅ Implemented trait for `MockMlsService` with full async support
- ✅ Added proper error types and result handling using `thiserror`
- ✅ Defined comprehensive interface covering all needed operations

**Operations Implemented**:
```rust
#[async_trait::async_trait]
pub trait MlsService: Send + Sync + std::fmt::Debug {
    fn as_any(&self) -> &dyn Any;
    async fn get_contacts(&self) -> Result<Vec<Contact>>;
    async fn get_conversations(&self) -> Result<Vec<Conversation>>;
    async fn send_message(&self, group_id: &GroupId, content: &str) -> Result<()>;
    async fn create_conversation(&self, name: &str, participants: Vec<PublicKey>) -> Result<String>;
    async fn add_contact(&self, pubkey: &str) -> Result<()>;
    async fn switch_conversation(&self, conversation_id: &str) -> Result<()>;
    async fn get_active_conversation(&self) -> Result<Option<String>>;
    async fn get_pending_invites_count(&self) -> Result<usize>;
    async fn toggle_connection(&self) -> Result<ConnectionStatus>;
}
```

### Step 5: Implement Configuration and Mode Selection
**Goal**: Add configuration system for mock vs real mode

**Tasks**:
- Create `dialog_lib/src/config.rs` with deployment configuration
- Add environment variable support for storage/relay selection
- Implement `DialogLib::new_real()` constructor for production mode
- Add feature flags for different storage backends

### Step 6: Update dialog_cli to Use dialog_lib  
**Goal**: Refactor dialog_cli to use shared library

**Tasks**:
- Replace dialog_cli direct MLS calls with `dialog_lib::RealMlsService`
- Remove duplicated logic from dialog_cli  
- Update CLI to use shared `MlsService` interface
- Ensure feature parity between old and new implementations
- Add CLI configuration for storage backend selection

### Step 7: Enhanced Mock Features (Optional)
**Goal**: Add advanced mock capabilities for testing

**Tasks**:
- Add deterministic mock scenarios for testing
- Implement mock message history persistence 
- Add mock contact search and filtering
- Create configurable mock response patterns

### Step 8: Testing and Documentation
**Goal**: Comprehensive testing and documentation

**Tasks**:
- Add unit tests for all service implementations
- Add integration tests for both mock and real services
- Create API documentation with examples
- Add performance benchmarks
- Document migration guide from old architecture

## Next Immediate Priority

**Step 3: Implement Real MLS Service** is the next major milestone. This involves:

1. **Examine dialog_cli implementation** to understand current MLS patterns
2. **Extract reusable logic** into `RealMlsService` 
3. **Implement storage abstraction** for Memory/SQLite backends
4. **Add relay management** for Nostr event publishing/subscribing
5. **Test real MLS operations** through the existing `MlsService` interface

This will provide a complete working MLS implementation that can be used by both dialog_tui and dialog_cli.

## File Structure

```
dialog_lib/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API
│   ├── types.rs            # Contact, Conversation, Message types
│   ├── service.rs          # MlsService trait definition
│   ├── mock_service.rs     # MockMlsService implementation
│   ├── real_service.rs     # RealMlsService implementation  
│   ├── commands.rs         # Command system
│   ├── config.rs           # Configuration management
│   └── errors.rs           # Error types
└── tests/
    ├── mock_service_test.rs
    └── integration_test.rs
```

## Step 1 Implementation Details

### Current dialog_tui/src/app.rs Code to Extract

**Core Types** (move to `dialog_lib/src/types.rs`):
```rust
pub struct Contact {
    pub name: String,
    pub pubkey: PublicKey,  // Will be real PublicKey
    pub online: bool,
}

pub struct Conversation {
    pub id: String,
    pub group_id: Option<GroupId>,  // Will be real GroupId
    pub name: String,
    pub participants: Vec<PublicKey>,
    pub last_message: Option<String>,
    pub unread_count: usize,
    pub is_group: bool,
}
```

**Mock Data Generation** (move to `dialog_lib/src/mock_service.rs`):
```rust
impl MockMlsService {
    pub fn setup_fake_data(&self) -> (Vec<Contact>, Vec<Conversation>) {
        // Move setup_fake_data() logic here
        // Generate real keys for mock contacts
        let alice_keys = Keys::generate();
        let bob_keys = Keys::generate();
        // ...
    }
}
```

**Command Processing** (move to `dialog_lib/src/commands.rs`):
```rust
pub async fn process_command(
    service: &dyn MlsService,
    command: &str,
    args: Vec<&str>
) -> Result<CommandResult> {
    // Move command logic from process_command()
}
```

### dialog_lib Public API

```rust
// dialog_lib/src/lib.rs
pub use types::*;
pub use service::MlsService;
pub use mock_service::MockMlsService;
pub use commands::*;
pub use errors::*;

// Public interface for UI applications
pub struct DialogLib {
    service: Box<dyn MlsService>,
}

impl DialogLib {
    pub fn new_mock() -> Self {
        Self {
            service: Box::new(MockMlsService::new()),
        }
    }
    
    pub async fn get_contacts(&self) -> Result<Vec<Contact>> {
        self.service.get_contacts().await
    }
    
    // ... other public methods
}
```

### Updated dialog_tui Integration

```rust
// dialog_tui/src/app.rs - simplified after extraction
use dialog_lib::{DialogLib, Contact, Conversation};

pub struct App {
    pub mode: AppMode,
    pub text_area: TextArea<'static>,
    pub connection_status: ConnectionStatus,
    pub messages: Vec<String>,
    pub scroll_offset: usize,
    
    // Replace direct data with library
    dialog_lib: DialogLib,
}

impl App {
    pub fn new() -> Self {
        let dialog_lib = DialogLib::new_mock();
        // ... rest of initialization
    }
    
    pub async fn get_contacts(&self) -> Vec<Contact> {
        self.dialog_lib.get_contacts().await.unwrap_or_default()
    }
}
```

## Benefits of This Approach

1. **Gradual Migration**: Start with mock, evolve to real implementation
2. **Clean Separation**: UI logic separate from business logic
3. **Testability**: Library can be unit tested independently
4. **Reusability**: Same logic used by both TUI and CLI
5. **Type Safety**: Real crypto types from the start
6. **Future-Proof**: Easy to add new UI implementations

## Success Metrics

- **Step 1**: dialog_tui behavior identical before/after extraction
- **Step 2**: All mock data uses real cryptographic types
- **Step 3**: Clean service abstraction with async support
- **Step 7**: Real MLS operations working through same interface
- **Step 9**: dialog_cli refactored to use shared library
- **Step 10**: >90% test coverage, comprehensive documentation

## Timeline

**Phase 1 (Steps 1-3)**: Foundation - 1-2 weeks
- Extract mock code and create service abstraction

**Phase 2 (Steps 4-6)**: Enhanced Mock - 1-2 weeks  
- Add full feature set with mock implementation

**Phase 3 (Steps 7-9)**: Real Implementation - 2-3 weeks
- Integrate actual MLS functionality

**Phase 4 (Step 10)**: Polish - 1 week
- Testing, documentation, cleanup

## Next Steps

**Ready to start Step 1** - Extract mock code from dialog_tui into dialog_lib:

1. Create `dialog_lib` crate in workspace
2. Move core types and mock data generation
3. Create `MockMlsService` implementation
4. Update `dialog_tui` to use library
5. Verify identical behavior

This approach ensures we deliver working functionality at each step while building toward a fully integrated solution.