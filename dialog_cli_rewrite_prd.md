# Dialog CLI Rewrite PRD

## Executive Summary

Rewrite `dialog_cli` to use `dialog_lib` as its core MLS implementation, replacing the current direct nostr-mls usage. This creates a unified architecture across all dialog clients and establishes the foundation for comprehensive interoperability testing between `dialog_tui` (via ht-mcp automation) and `dialog_cli` before attempting whitenoise integration.

**Key Goals**:
1. **Unified Architecture**: Both `dialog_tui` and `dialog_cli` use `dialog_lib` 
2. **Persistent Storage**: Add SQLite storage to `dialog_lib` for `dialog_cli` persistence
3. **Interop Testing Foundation**: Enable automated testing between `dialog_tui` and `dialog_cli`
4. **Whitenoise Preparation**: Establish proven interop patterns before whitenoise integration

## Current State Analysis

### Dialog CLI Current Architecture
- **Direct Dependencies**: Uses `nostr-mls` directly
- **Storage**: No persistence - ephemeral sessions only
- **MLS Operations**: Basic implementation scattered across CLI commands
- **Testing**: Limited unit tests, no integration testing
- **Interface**: Simple CLI commands with manual input/output

### Dialog TUI Current Architecture  
- **Library Usage**: Uses `dialog_lib` for all MLS operations
- **Storage**: Ephemeral memory storage via `NostrMlsMemoryStorage`
- **MLS Operations**: Full feature set via `dialog_lib::MlsService`
- **Testing**: Interactive TUI with ht-mcp automation support
- **Interface**: Rich TUI with real-time updates

### Dialog Lib Current Architecture
- **Storage Abstraction**: `MlsService` trait with memory implementation
- **MLS Operations**: Comprehensive MLS feature set
- **Event Handling**: Real-time message processing
- **Types**: Shared types for contacts, conversations, messages
- **Configuration**: Flexible relay and key management

## Architecture Redesign

### Phase 1: Add SQLite Storage to Dialog Lib

#### New Components
```rust
// dialog_lib/src/sqlite_storage.rs
pub struct NostrMlsSqliteStorage {
    connection: SqliteConnection,
    // ... SQLite-specific fields
}

impl NostrMlsStorage for NostrMlsSqliteStorage {
    // Implement all storage methods with SQLite persistence
}

// dialog_lib/src/lib.rs
pub enum StorageBackend {
    Memory(NostrMlsMemoryStorage),
    Sqlite(NostrMlsSqliteStorage),
}

pub struct MlsServiceBuilder {
    storage_backend: StorageBackend,
    relay_urls: Vec<String>,
    // ... other config
}
```

#### Storage Migration Strategy
- **Backward Compatibility**: Keep memory storage as default for `dialog_tui`
- **New Default**: SQLite storage for `dialog_cli` 
- **Configuration**: Builder pattern for storage backend selection
- **Migration Tools**: Export/import between storage backends

### Phase 2: Rewrite Dialog CLI

#### New CLI Architecture
```rust
// dialog_cli/src/main.rs
use dialog_lib::{MlsService, RealMlsService, StorageBackend, NostrMlsSqliteStorage};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// SQLite database path
    #[arg(long, default_value = "./dialog_cli.db")]
    db_path: PathBuf,
    
    /// Relay URLs (comma-separated)
    #[arg(long, env = "DIALOG_RELAY_URLS")]
    relays: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Connect { relay_url: Option<String> },
    Disconnect,
    PublishKeyPackage,
    ListContacts,
    AddContact { pubkey: String, alias: Option<String> },
    CreateGroup { name: String, members: Vec<String> },
    ListGroups,
    JoinGroup { group_id: String },
    SendMessage { group_id: String, message: String },
    FetchMessages { group_id: Option<String> },
    ListInvites,
    AcceptInvite { invite_id: String },
    ShowPublicKey,
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize SQLite storage
    let storage = NostrMlsSqliteStorage::new(&cli.db_path).await?;
    let storage_backend = StorageBackend::Sqlite(storage);
    
    // Create MLS service
    let mls_service = RealMlsService::builder()
        .storage_backend(storage_backend)
        .relay_urls(parse_relay_urls(&cli.relays))
        .build()
        .await?;
    
    // Execute command
    match cli.command {
        Commands::Connect { relay_url } => {
            mls_service.connect(relay_url).await?;
            println!("Connected successfully");
        }
        Commands::CreateGroup { name, members } => {
            let group_id = mls_service.create_group(&name, members).await?;
            println!("Created group: {}", group_id);
        }
        // ... other commands
    }
    
    Ok(())
}
```

#### Command Implementation Strategy
- **State Persistence**: All operations persist to SQLite
- **Session Management**: Support multiple CLI invocations with shared state
- **Error Handling**: Comprehensive error reporting and recovery
- **Output Format**: Structured output for automation (JSON option)

### Phase 3: Interop Testing Infrastructure

#### Test Scenarios
1. **Dialog TUI → Dialog CLI**: TUI creates group, CLI joins and responds
2. **Dialog CLI → Dialog TUI**: CLI creates group, TUI (via ht-mcp) joins and responds  
3. **Bi-directional Messaging**: Extended message exchange between both clients
4. **Storage Verification**: Ensure CLI persists state correctly across sessions
5. **Error Recovery**: Network issues, reconnection, state synchronization

#### Automation Strategy
```rust
// tests/interop_tests.rs
use dialog_lib::test_helpers::*;

#[tokio::test]
async fn test_cli_tui_interop() -> Result<()> {
    // Setup: Start relays and initialize clients
    let relay_manager = TestRelayManager::new().await?;
    
    // Initialize CLI with SQLite storage
    let cli_service = create_cli_service("alice", &relay_manager).await?;
    
    // Initialize TUI via ht-mcp automation
    let tui_automation = HtMcpAutomation::new();
    let tui_session = tui_automation.create_session("bob", &relay_manager.urls()).await?;
    let tui_pubkey = tui_automation.setup_and_get_pubkey(&tui_session).await?;
    
    // Test 1: CLI creates group, TUI joins
    let group_id = cli_service.create_group("TestGroup", vec![tui_pubkey]).await?;
    tui_automation.accept_invite(&tui_session).await?;
    
    // Test 2: Bi-directional messaging
    cli_service.send_message(&group_id, "Hello from CLI!").await?;
    tui_automation.send_message(&tui_session, "Hello from TUI!").await?;
    
    // Test 3: Verify persistence - restart CLI
    drop(cli_service);
    let cli_service_2 = create_cli_service("alice", &relay_manager).await?;
    let groups = cli_service_2.list_groups().await?;
    assert!(groups.iter().any(|g| g.id == group_id));
    
    // Test 4: Verify message history
    let messages = cli_service_2.fetch_messages(&group_id).await?;
    assert!(messages.iter().any(|m| m.content.contains("Hello from TUI!")));
    
    Ok(())
}
```

## Implementation Plan

### Week 1: SQLite Storage Foundation
- [ ] Add SQLite dependencies to `dialog_lib`
- [ ] Implement `NostrMlsSqliteStorage` with full storage trait
- [ ] Add storage backend abstraction and builder pattern
- [ ] Create migration utilities between storage backends
- [ ] Test storage implementation with comprehensive unit tests

### Week 2: CLI Rewrite
- [ ] Rewrite `dialog_cli/src/main.rs` to use `dialog_lib`
- [ ] Implement all CLI commands using `MlsService` interface
- [ ] Add SQLite as default storage backend for CLI
- [ ] Implement session persistence across CLI invocations
- [ ] Add structured output options for automation

### Week 3: Interop Testing Infrastructure  
- [ ] Create ht-mcp automation helpers for `dialog_tui`
- [ ] Implement CLI automation and verification helpers
- [ ] Build comprehensive interop test suite
- [ ] Add test relay management and coordination
- [ ] Establish CI/CD pipeline for interop testing

### Week 4: Advanced Testing & Validation
- [ ] Stress testing with rapid message exchange
- [ ] Error recovery and network failure scenarios
- [ ] Long-running session persistence validation
- [ ] Performance benchmarking and optimization
- [ ] Documentation and usage examples

## Success Metrics

### Primary Success Criteria
1. **✅ Unified Architecture**: Both clients use `dialog_lib` exclusively
2. **✅ Storage Persistence**: CLI maintains state across sessions via SQLite
3. **✅ CLI-TUI Interop**: Complete message exchange in both directions
4. **✅ Automated Testing**: Comprehensive test suite with ht-mcp automation
5. **✅ Performance**: Message delivery within 1 second under normal conditions

### Quality Metrics
- **Code Reuse**: >90% of MLS logic shared via `dialog_lib`
- **Test Coverage**: >95% coverage of interop scenarios
- **Reliability**: 99% message delivery success rate in tests
- **Documentation**: Complete usage examples and troubleshooting guides

## Benefits for Whitenoise Interop

### Proven Architecture
- **Validated Patterns**: Interop testing patterns proven with CLI-TUI
- **Robust Storage**: SQLite storage proven for persistence needs
- **Automation Framework**: ht-mcp automation patterns established
- **Error Handling**: Comprehensive error recovery mechanisms tested

### Reduced Risk
- **Known Quantities**: Both dialog clients fully understood and tested
- **Debugging Tools**: Established patterns for cross-client debugging
- **Test Infrastructure**: Proven automation and verification systems
- **Incremental Approach**: Add whitenoise as third client to existing framework

### Technical Foundation
- **Storage Compatibility**: Both memory (TUI) and persistent (CLI) storage validated
- **Protocol Compliance**: Ensured MLS protocol compliance across implementations
- **Message Reliability**: Proven message delivery and ordering
- **State Management**: Validated state synchronization across clients

## Risk Assessment

### Medium-Risk Items
1. **SQLite Integration Complexity**: Storage abstraction may require significant refactoring
2. **CLI UX Changes**: Users may need to adapt to new command patterns
3. **Performance Impact**: SQLite overhead vs memory storage
4. **Migration Path**: Existing CLI users need clear upgrade path

### Mitigation Strategies
- **Incremental Development**: Implement storage abstraction incrementally
- **Backward Compatibility**: Maintain existing CLI command interface where possible
- **Performance Testing**: Benchmark SQLite vs memory storage thoroughly
- **Clear Documentation**: Provide comprehensive migration and upgrade guides

## Next Steps

1. **Start with Storage Layer**: Implement SQLite storage in `dialog_lib`
2. **Validate Storage**: Comprehensive testing of persistence and performance
3. **Rewrite CLI**: Migrate `dialog_cli` to use `dialog_lib` + SQLite
4. **Build Test Framework**: Create ht-mcp automation for TUI interactions
5. **Establish Interop**: Prove CLI-TUI interoperability works reliably
6. **Prepare for Whitenoise**: Use proven patterns for whitenoise integration

This approach provides a solid foundation for eventual whitenoise interoperability while delivering immediate value through improved architecture and comprehensive testing capabilities.