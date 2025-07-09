# Dialog Integration PRD: Unified Architecture Implementation

## Executive Summary

This PRD outlines the step-by-step integration strategy to achieve unified architecture where dialog_tui and dialog_cli share all business logic through dialog_lib, enabling seamless messaging between CLI and TUI users.

## Vision

Create a production-ready messaging system where:
- **dialog_lib** = All MLS/Nostr business logic (real and mock modes)
- **dialog_tui** = Pure UI focused on user interaction 
- **dialog_cli** = Thin command-line wrapper
- **End-to-End Messaging** = CLI users ↔ TUI users communicate seamlessly

## Current Status: Phase 2 COMPLETED ✅

### ✅ **Architecture Foundation - COMPLETED**
- **Dependency Isolation**: dialog_tui has ZERO direct nostr-mls dependencies
- **Type Reexports**: All needed types available through dialog_lib
- **RealMlsService**: Extracted from dialog_cli, implements full MLS operations
- **Clean Integration**: DialogLib can switch between Mock/Real modes

### ✅ **Technical Achievements**
- RealMlsService extracted from dialog_cli with memory storage
- Complete MlsService trait implementation with real MLS operations
- Configuration system for Mock/Real mode switching
- dialog_tui builds and runs with zero nostr-mls dependencies

## Phase 3: Production Readiness - NEXT STEPS

### **Step 3A: Migrate dialog_tui to RealMlsService** ⭐ **IMMEDIATE NEXT**

**Goal**: Switch dialog_tui from mock data to real MLS operations

**Why This First**: Test RealMlsService with TUI interface before CLI integration to validate both compatibilities independently.

**Implementation Plan**:
```rust
// dialog_tui/src/main.rs - Add mode selection
let dialog_lib = match args.mode {
    Some("real") => DialogLib::new_real().await?,
    _ => DialogLib::new_mock_with_data().await,
};

// dialog_tui/src/app.rs - Update initialization
impl App {
    pub async fn new_real() -> Result<Self> {
        let dialog_lib = DialogLib::new_real().await?;
        // ... rest of initialization
    }
}
```

**Tasks**:
1. Add command-line argument for `--mode real|mock` 
2. Update App::new() to accept DialogLib instance
3. Replace mock data usage with real dialog_lib calls
4. Test conversation persistence across TUI sessions
5. Verify async MLS operations don't block UI rendering
6. Add development toggle for easy mock/real switching

**Success Criteria**:
- dialog_tui works with real MLS operations
- No UI blocking during MLS operations
- Conversations persist across TUI restarts
- Real key generation and storage functional

**Files to Modify**:
- `dialog_tui/src/main.rs` - Add mode argument parsing
- `dialog_tui/src/app.rs` - Update initialization pattern

---

### **Step 3B: Enable Cross-Client Messaging** 🎯 **CORE GOAL**

**Goal**: Achieve end-to-end messaging between dialog_tui and dialog_cli

**Implementation Plan**:
1. **Test Group Creation**: Use dialog_cli to create group, invite TUI user
2. **Test Message Flow**: CLI user sends message → TUI user receives message  
3. **Test Bi-directional**: TUI user sends message → CLI user receives message
4. **Verify State Sync**: Both clients maintain consistent MLS group state

**Testing Workflow**:
```bash
# Terminal 1: Start TUI with real mode
cd dialog_tui && cargo run -- --mode real

# Terminal 2: CLI creates group and sends message
cd dialog_cli
cargo run -- publish-key --key alice
cargo run -- create-group-and-send --key alice --name "Test Group" --counterparty <tui_pubkey> --message "Hello from CLI!"

# Verify: TUI receives invitation and message
# Then: TUI sends response, CLI receives it
```

**Success Criteria**:
- Group creation from CLI appears in TUI
- Messages sent from CLI appear in TUI
- Messages sent from TUI appear in CLI via get-messages
- MLS state remains synchronized between clients

---

### **Step 3C: Integrate dialog_cli with dialog_lib** 🏁 **FINAL STEP**

**Goal**: Complete the unified architecture by making dialog_cli use dialog_lib

**Implementation Plan**:
```rust
// dialog_cli/src/main.rs - Simplified new architecture
use dialog_lib::{DialogLib, ServiceMode};

#[tokio::main]
async fn main() -> Result<()> {
    let keys = get_keys_from_args()?;
    let dialog = DialogLib::new_real_with_keys(keys).await?;
    
    match matches.subcommand() {
        Some(("send-message", args)) => {
            let group_id = parse_group_id(args)?;
            let content = args.get_one::<String>("message").unwrap();
            dialog.send_message(&group_id, content).await?;
        }
        Some(("get-messages", args)) => {
            let conversations = dialog.get_conversations().await?;
            // Display conversations and messages
        }
        // ... other commands map to dialog_lib methods
    }
}
```

**Tasks**:
1. Replace direct MLS calls with dialog_lib operations
2. Map CLI commands to dialog_lib methods
3. Remove duplicated MLS logic from dialog_cli
4. Maintain identical CLI behavior and output
5. Add any missing methods to MlsService trait if needed

**Success Criteria**:
- dialog_cli behavior identical before/after integration
- Zero code duplication between CLI and TUI
- Full end-to-end messaging CLI ↔ TUI confirmed
- Clean, maintainable codebase

**Files to Modify**:
- `dialog_cli/src/main.rs` - Replace with thin wrapper
- `dialog_cli/Cargo.toml` - Remove direct nostr-mls dependencies 
- `dialog_lib/src/service.rs` - Add any missing CLI-specific methods

---

## Architecture After Completion

```
┌─────────────────┐    ┌─────────────────┐
│   dialog_tui    │    │   dialog_cli    │
│                 │    │                 │
│  ┌───────────┐  │    │  ┌───────────┐  │
│  │    UI     │  │    │  │  Args     │  │
│  │ Rendering │  │    │  │ Parsing   │  │
│  └───────────┘  │    │  └───────────┘  │
└─────────────────┘    └─────────────────┘
         │                       │
         └───────────────────────┘
                     │
         ┌─────────────────────────┐
         │     dialog_lib         │
         │                        │
         │  ┌─────────────────┐   │
         │  │ MockMlsService  │   │
         │  │ (Development)   │   │
         │  └─────────────────┘   │
         │                        │
         │  ┌─────────────────┐   │
         │  │ RealMlsService  │   │
         │  │ (Production)    │   │
         │  └─────────────────┘   │
         │                        │
         │  ┌─────────────────┐   │
         │  │ Configuration   │   │
         │  │ Mock/Real Mode  │   │
         │  └─────────────────┘   │
         └─────────────────────────┘
                     │
             ┌───────────────┐
             │  nostr-mls    │
             │   nostr-sdk   │
             │ (Dependencies │
             │  only here)   │
             └───────────────┘
```

## Benefits of This Approach

### 🔒 **Risk Mitigation**
- Test each integration independently
- Validate RealMlsService with TUI first
- Confirm cross-client messaging works
- Only then integrate CLI (lowest risk)

### 🏗️ **Architecture Quality**
- Zero code duplication
- Clean separation of concerns  
- Easy to test and maintain
- Consistent behavior across UIs

### 🚀 **Development Velocity**
- Mock mode for rapid UI development
- Real mode for production testing
- Shared business logic reduces bugs
- Clear integration points

## Timeline Estimate

- **Step 3A** (TUI → Real): 2-3 days
- **Step 3B** (Cross-Client): 2-3 days  
- **Step 3C** (CLI Integration): 2-3 days
- **Testing & Polish**: 1-2 days

**Total: 7-11 days to complete unified architecture**

## Success Metrics

### Technical Metrics
- ✅ Zero direct nostr-mls dependencies in UIs
- ✅ Identical behavior before/after integration  
- ✅ End-to-end messaging CLI ↔ TUI working
- ✅ >95% code reuse between CLI and TUI

### User Experience Metrics  
- ✅ TUI remains responsive during MLS operations
- ✅ CLI commands have identical output format
- ✅ Message delivery is reliable and fast
- ✅ Development workflow supports both mock and real modes

## Next Immediate Action

**Start Step 3A**: Migrate dialog_tui to use `DialogLib::new_real()` and test real MLS operations through the TUI interface. This validates our RealMlsService implementation before proceeding to cross-client testing.