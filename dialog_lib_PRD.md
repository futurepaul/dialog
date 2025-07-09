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

### ✅ Step 3: Implement Real MLS Service **COMPLETED**
**Goal**: Create `RealMlsService` using actual Nostr-MLS operations

**Completed Tasks**:
- ✅ Extracted MLS logic from `dialog_cli/src/main.rs` into `dialog_lib/src/real_service.rs`
- ✅ Implemented `RealMlsService` struct with memory storage (simplified from SQLite)
- ✅ Added real key management, relay connection management
- ✅ Implemented core `MlsService` trait methods with actual Nostr-MLS calls
- ✅ Added configuration system with environment variable support
- ✅ Implemented proper error handling for MLS operations
- ✅ **ARCHITECTURE DECISION**: Simplified to memory-only storage for cleaner implementation

**Implementation Details**:
```rust
// dialog_lib/src/real_service.rs
pub struct RealMlsService {
    nostr_mls: Arc<RwLock<NostrMls<NostrMlsMemoryStorage>>>,
    client: Arc<RwLock<Client>>,
    keys: Keys,
    relay_url: String,
    connection_status: Arc<RwLock<ConnectionStatus>>,
}

impl RealMlsService {
    pub async fn new(keys: Keys, relay_url: String) -> Result<Self>
}

#[async_trait::async_trait]  
impl MlsService for RealMlsService {
    async fn get_conversations(&self) -> Result<Vec<Conversation>> {
        // Real group querying from storage
    }
    
    async fn send_message(&self, group_id: &GroupId, content: &str) -> Result<()> {
        // Real MLS message encryption + Nostr relay publishing + local processing
    }
    
    async fn get_pending_invites_count(&self) -> Result<usize> {
        // Real pending welcomes counting
    }
    // NOTE: create_conversation() and add_contact() marked as TODO for next phase
}
```

**Files Created**:
- ✅ `dialog_lib/src/real_service.rs` - Real MLS implementation with memory storage
- ✅ `dialog_lib/src/config.rs` - Simplified configuration (memory-only)

**Success Criteria Met**:
- ✅ Real MLS operations working through same `MlsService` interface
- ✅ Memory storage backend implemented and functional
- ✅ Message encryption/decryption functional via existing dialog_cli patterns
- ✅ **DEPENDENCY ISOLATION**: dialog_tui now has ZERO direct nostr-mls dependencies
- ✅ **TYPE REEXPORTS**: All nostr-mls types available through dialog_lib

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

### ✅ Step 5: Implement Configuration and Mode Selection **COMPLETED**
**Goal**: Add configuration system for mock vs real mode

**Completed Tasks**:
- ✅ Created `dialog_lib/src/config.rs` with deployment configuration and builder pattern
- ✅ Added environment variable support for storage/relay selection (DIALOG_MODE, DIALOG_STORAGE, DIALOG_DB_PATH, DIALOG_RELAY_URL)
- ✅ Implemented `DialogLib::new_real()` constructor for production mode
- ✅ Added feature flags for different storage backends (memory-storage, sqlite-storage, all-storage)
- ✅ Added conditional compilation support for storage backends
- ✅ Created example demonstrating configuration usage

**Features Implemented**:
```rust
// Environment variable configuration
DialogLib::from_env()

// Builder pattern configuration
DialogConfig::builder()
    .mode(ServiceMode::Real)
    .storage(StorageConfig::Memory)
    .relay_url("wss://custom.relay")
    .build()

// Simple constructors
DialogLib::new_real()  // Production mode
DialogLib::new_mock()  // Mock mode
```

**Feature Flags**:
- `memory-storage` (default): Enables in-memory storage
- `sqlite-storage`: Enables SQLite storage backend
- `all-storage`: Enables all storage backends

**Success Criteria Met**:
- ✅ Complete configuration system with environment variables
- ✅ Feature flags for optional storage backends  
- ✅ Ready for RealMlsService integration in Step 3
- ✅ Clean separation of storage concerns

### Step 6: Migrate dialog_tui to RealMlsService ⭐ **NEXT PRIORITY**
**Goal**: Switch dialog_tui from MockMlsService to RealMlsService for real messaging

**Rationale**: Test RealMlsService integration with TUI first, then CLI integration, to validate both compatibilities step-by-step.

**Tasks**:
- Update dialog_tui to use `DialogLib::new_real()` instead of mock
- Test real MLS operations through TUI interface
- Verify message persistence and conversation management
- Ensure UI remains responsive with real async MLS operations
- Add configuration to toggle between Mock/Real modes for development

### Step 7: Enable dialog_tui ↔ dialog_cli Messaging
**Goal**: Achieve end-to-end messaging between TUI and CLI users

**Tasks**:
- Test group creation from dialog_cli  
- Test message sending from dialog_cli to dialog_tui user
- Test message receiving in dialog_tui from dialog_cli user
- Verify MLS state synchronization between both clients
- Document messaging workflow

### Step 8: Update dialog_cli to Use dialog_lib  
**Goal**: Refactor dialog_cli to use shared library (final integration)

**Tasks**:
- Replace dialog_cli direct MLS calls with `dialog_lib::RealMlsService`
- Remove duplicated logic from dialog_cli  
- Update CLI to use shared `MlsService` interface
- Ensure feature parity between old and new implementations
- Create thin CLI wrapper that maps commands to dialog_lib operations

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

**Step 6: Migrate dialog_tui to RealMlsService** is the next major milestone. With RealMlsService implementation complete, this involves:

1. **Switch dialog_tui from mock to real mode** using `DialogLib::new_real()`
2. **Test real MLS operations** through TUI interface (conversations, messaging)
3. **Verify async MLS operations** don't block UI responsiveness  
4. **Add mode configuration** to toggle Mock/Real for development
5. **Test conversation persistence** and state management

**Then Step 7: Enable dialog_tui ↔ dialog_cli Messaging** for end-to-end testing:

1. **Test cross-client messaging** between TUI and CLI users
2. **Verify MLS state synchronization** between different dialog_lib instances
3. **Document messaging workflow** for future development

**Finally Step 8: Integrate dialog_cli with dialog_lib** to complete the architecture:

1. **Create thin CLI wrapper** that uses dialog_lib for all MLS operations
2. **Remove duplicated MLS logic** from dialog_cli
3. **Achieve full code sharing** between TUI and CLI

This staged approach ensures we test each integration point thoroughly before proceeding to the next.

## File Structure

```
dialog_lib/
├── Cargo.toml              # Feature flags for storage backends
├── src/
│   ├── lib.rs              # Public API with configuration constructors
│   ├── types.rs            # Contact, Conversation, Message types
│   ├── service.rs          # MlsService trait definition
│   ├── mock_service.rs     # MockMlsService implementation
│   ├── real_service.rs     # RealMlsService implementation (TODO: Step 3)
│   ├── commands.rs         # Command system (TODO)
│   ├── config.rs           # ✅ Configuration management
│   └── errors.rs           # Error types
├── examples/
│   └── config_demo.rs      # ✅ Configuration usage examples
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