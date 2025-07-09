# Dialog Project: Unified Architecture PRD

## Executive Summary

Dialog is a Nostr-based messaging application with MLS (Message Layer Security) under the hood. The project consists of two user interfaces (CLI and TUI) unified by a shared library (`dialog_lib`) that encapsulates all business logic. This PRD consolidates the project's architecture, current state, and roadmap for achieving a unified, production-ready messaging system.

## 🎯 Current Status (Updated: Today)

**Phase 1 ✅ COMPLETED** - Core group functionality is working:
- Group creation with multiple participants
- Key package publishing and management  
- Invite sending and acceptance
- Message sending with MLS state synchronization
- All commands integrated into dialog_tui (`/create`, `/keypackage`, `/invites`, `/accept`)

**Phase 2 🚧 IN PROGRESS** - Real-time messaging:
- Need to implement message fetching/receiving
- Need WebSocket subscriptions for live updates
- Cross-client messaging testing pending

**What's Working:**
- Can create groups and send invites between users
- Can accept invites and join groups
- Can send messages (but can't see received messages yet)
- Full MLS encryption and state management

**Next Critical Step:**
- Implement message reception so users can see each other's messages
- Add real-time subscriptions for live chat experience

## 🚀 Quick Testing Guide

To test current functionality:

```bash
# Terminal 1: Start relay
nak serve --verbose

# Terminal 2: Start TUI for Alice
cd dialog_tui && cargo run

# Terminal 3: Start TUI for Bob (different data dir)
cd dialog_tui && cargo run -- --data-dir /tmp/bob_data

# In each TUI:
1. /connect                    # Connect to relay
2. /keypackage                 # Publish key package
3. /pk                        # Get your pubkey
4. /add <other_user_pubkey>   # Add contact
5. /create TestGroup Bob      # Create group (from Alice)
6. /invites                   # Check invites (from Bob)
7. /accept <group_id>         # Accept invite (from Bob)
8. /switch 1                  # Switch to conversation
9. Send messages              # Type without / to send

Note: Messages are sent but not yet visible to recipients!
```

## Project Vision

Build a secure, decentralized messaging platform where:
- Multiple UI clients (CLI, TUI, future mobile/web) share identical functionality
- All MLS and Nostr logic is centralized in `dialog_lib`
- Cross-client messaging works seamlessly
- The system is thoroughly tested and production-ready

## Architecture Overview

```
┌─────────────┐     ┌─────────────┐     ┌──────────────┐
│ dialog_cli  │     │ dialog_tui  │     │ Future UIs   │
└──────┬──────┘     └──────┬──────┘     └──────┬───────┘
       │                   │                    │
       └───────────────────┴────────────────────┘
                           │
                   ┌───────▼────────┐
                   │   dialog_lib    │
                   │                 │
                   │ - MlsService    │
                   │ - Types         │
                   │ - Config        │
                   │ - Service API   │
                   └───────┬────────┘
                           │
                ┌──────────┴───────────┐
                │                      │
        ┌───────▼────────┐    ┌───────▼────────┐
        │   nostr-mls    │    │   nostr-sdk    │
        └────────────────┘    └────────────────┘
```

## Current Implementation Status

### ✅ Completed Components

1. **dialog_lib Architecture**
   - Clean separation of concerns achieved
   - `RealMlsService` extracted from dialog_cli
   - In-memory storage for state management
   - Type definitions unified across codebase
   - Zero direct MLS dependencies in UI layers

2. **dialog_tui Integration**
   - Mock mode completely removed
   - Real MLS operations integrated
   - Async operations without UI blocking
   - Basic messaging to existing groups works
   - Contact management functional

3. **Core Features Working**
   - Connection management (relay connections)
   - Profile loading and publishing
   - Contact adding with pubkey validation
   - Group listing (existing groups only)
   - Message sending (to pre-existing groups)
   - Pending invite counting

### ❌ Missing Critical Features

1. **Group Management**
   - Group creation (`create_conversation` is stubbed)
   - Invite processing and acceptance
   - Group member management
   - Group metadata updates

2. **Message Synchronization**
   - Pre-send group state sync
   - Post-send local state update
   - Message history loading
   - Real-time event subscriptions

3. **Cross-Client Compatibility**
   - dialog_tui ↔ dialog_cli messaging untested
   - MLS state synchronization issues
   - Event routing inconsistencies

4. **dialog_cli Integration**
   - Still uses direct MLS calls
   - Duplicated business logic
   - Not using dialog_lib

## Implementation Roadmap

### Phase 1: Core Functionality (Week 1-2) ✅ COMPLETED
**Goal: Enable full group lifecycle and messaging**

#### 1.1 Group Creation Implementation ✅
```rust
// In RealMlsService
async fn create_group(&self, name: &str, participants: Vec<PublicKey>) -> Result<String, DialogError> {
    // 1. Validate participants exist and have key packages
    // 2. Fetch key packages from relay
    // 3. Create MLS group with config
    // 4. Generate and send encrypted welcomes
    // 5. Store group state locally
    // 6. Return group_id
}
```

Key tasks:
- [x] Port group creation logic from dialog_cli
- [x] Implement key package fetching
- [x] Add welcome message encryption
- [x] Handle group configuration (admins, relays)
- [x] Add proper error handling

#### 1.2 Invite System Implementation ✅
```rust
async fn list_pending_invites(&self) -> Result<Vec<PendingInvite>, DialogError> {
    // 1. Fetch gift-wrapped events from relay
    // 2. Decrypt and validate welcomes
    // 3. Extract group metadata
    // 4. Return structured invite list
}

async fn accept_invite(&self, invite_id: &str) -> Result<(), DialogError) {
    // 1. Process welcome message
    // 2. Join MLS group
    // 3. Update local state
    // 4. Publish join confirmation
}
```

Key tasks:
- [x] Implement gift-wrap event processing
- [x] Add welcome message handling
- [x] Create invite acceptance workflow
- [x] Add invite rejection capability
- [x] Handle invite expiration

#### 1.3 Message Synchronization ✅
```rust
async fn sync_before_send(&self, group_id: &GroupId) -> Result<(), DialogError> {
    // 1. Fetch all group evolution events since last sync
    // 2. Process each event to update MLS state
    // 3. Handle any pending proposals
    // 4. Ensure local state matches group state
}
```

Key tasks:
- [x] Port sync logic from dialog_cli
- [x] Add event deduplication
- [x] Handle out-of-order events
- [x] Implement state recovery
- [x] Add sync status tracking

### Phase 2: Real-time Messaging (Week 2) 🚧 IN PROGRESS
**Goal: Enable live message reception and display**

#### 2.1 Message Storage Layer
```rust
// In RealMlsService - add in-memory message cache
struct MessageCache {
    messages: HashMap<GroupId, Vec<DecryptedMessage>>,
    last_sync: HashMap<GroupId, Timestamp>,
}
```

Key tasks:
- [ ] Add message storage to RealMlsService
- [ ] Implement message deduplication
- [ ] Add message ordering by timestamp
- [ ] Cache decrypted messages for UI display

#### 2.2 Dual API Design
```rust
// One-shot fetch for CLI compatibility
async fn fetch_messages(&self, group_id: &GroupId) -> Result<Vec<Message>>;

// Streaming subscription for TUI
async fn subscribe_to_groups(&self) -> Result<MessageStream>;
```

Key tasks:
- [ ] Implement /fetch command for manual message retrieval
- [ ] Design MessageStream type for async message delivery
- [ ] Add subscription management to RealMlsService
- [ ] Handle multiple group subscriptions efficiently

#### 2.3 Background Event Processing
```rust
// Inspired by whitenoise architecture
struct EventProcessor {
    receiver: mpsc::Receiver<ProcessableEvent>,
    ui_sender: mpsc::Sender<UiUpdate>,
}

async fn start_event_processing(dialog_lib: &DialogLib) {
    // 1. Subscribe to all group messages
    // 2. Process incoming events
    // 3. Decrypt and store messages
    // 4. Send UI updates
}
```

Key tasks:
- [ ] Create background task for event processing
- [ ] Implement channel-based communication with UI
- [ ] Add subscription lifecycle management
- [ ] Handle reconnection and error recovery

#### 2.4 UI Integration
```rust
// In dialog_tui App
enum UiUpdate {
    NewMessage { group_id: GroupId, message: Message },
    GroupStateChange { group_id: GroupId, epoch: u64 },
    ConnectionStatus(ConnectionStatus),
}
```

Key tasks:
- [ ] Add message receiver to App struct
- [ ] Update UI on new message arrival
- [ ] Show typing indicators (future)
- [ ] Add notification system

### Phase 3: Cross-Client Messaging (Week 3)
**Goal: Prove architecture with real interoperability**

#### 3.1 Testing Infrastructure
- [ ] Set up ephemeral relay testing with `nak serve --verbose`
- [ ] Create multi-client test scenarios
- [ ] Add MLS state verification tools
- [ ] Implement message delivery verification

#### 3.2 Cross-Client Workflow
Test scenario:
1. Start dialog_tui with User A
2. Start dialog_cli with User B  
3. User B creates group and invites User A
4. User A accepts invite via TUI
5. Both users exchange messages
6. Verify MLS state consistency

Key validations:
- [ ] Messages delivered in both directions
- [ ] MLS epochs stay synchronized
- [ ] No message decryption failures
- [ ] Proper sender attribution
- [ ] Message ordering preserved

### Phase 4: dialog_cli Migration (Week 4)
**Goal: Complete architectural unification**

#### 4.1 Command Mapping
Map existing CLI commands to dialog_lib:
- `create-identity` → `dialog_lib.create_identity()`
- `publish-key-packages` → `dialog_lib.publish_key_packages()`
- `create-group` → `dialog_lib.create_group()`
- `send-message` → `dialog_lib.send_message()`
- etc.

#### 4.2 Remove Duplicated Code
- [ ] Replace direct MLS calls with dialog_lib
- [ ] Remove CLI-specific MLS logic
- [ ] Unify configuration handling
- [ ] Consolidate error types
- [ ] Share state management

### Phase 5: Production Readiness (Week 5)
**Goal: Robust, tested, production system**

#### 5.1 Comprehensive Testing
Test categories from whitenoise best practices:
- **Unit tests**: Service methods, type conversions, validation
- **Integration tests**: Multi-user workflows, relay interactions
- **Property tests**: Message ordering, state consistency
- **Stress tests**: High message volume, many participants
- **Failure tests**: Network failures, malformed events

#### 5.2 TUI Enhancements
- [ ] Implement @ command navigation system
- [ ] Add conversation search and filtering
- [ ] Improve message rendering
- [ ] Add notification system
- [ ] Fix text input edge cases

#### 5.3 Performance Optimization
- [ ] Add event caching layer
- [ ] Implement batch message processing
- [ ] Optimize relay subscriptions
- [ ] Add connection pooling
- [ ] Profile and optimize hot paths

### Phase 6: Advanced Features (Week 6)
**Goal: Feature parity with modern messengers**

- [ ] Message reactions
- [ ] File/media sharing via Blossom
- [ ] Message editing/deletion
- [ ] Typing indicators
- [ ] Read receipts
- [ ] Group administration tools
- [ ] Message search
- [ ] Export/backup functionality

## Real-time Messaging Architecture

### Overview
The real-time messaging system uses WebSocket subscriptions via nostr-sdk to receive events as they arrive. This differs from the CLI's polling approach and enables true real-time chat.

### Key Components

#### 1. Subscription Management
```rust
// In RealMlsService
pub async fn subscribe_to_all_groups(&self) -> Result<()> {
    let groups = self.nostr_mls.get_groups()?;
    let mut filters = Vec::new();
    
    // Create filters for all group messages
    for group in groups {
        let filter = Filter::new()
            .kind(Kind::MlsGroupMessage)
            .custom_tag(SingleLetterTag::lowercase(Alphabet::H), 
                       hex::encode(&group.nostr_group_id));
        filters.push(filter);
    }
    
    // Subscribe to gift wraps for invites
    let giftwrap_filter = Filter::new()
        .kind(Kind::GiftWrap)
        .pubkey(self.keys.public_key());
    filters.push(giftwrap_filter);
    
    // Create subscription
    let subscription_id = SubscriptionId::new("dialog_messages");
    self.client.subscribe_with_id(subscription_id, filters, None).await?;
}
```

#### 2. Event Processing Pipeline
```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌────────┐
│   Relay     │────▶│ Subscription │────▶│   Process   │────▶│  UI    │
│ WebSocket   │     │   Handler    │     │  & Decrypt  │     │ Update │
└─────────────┘     └──────────────┘     └─────────────┘     └────────┘
```

#### 3. Message Flow
1. **Event Arrival**: WebSocket receives new MLS message event
2. **Validation**: Check event signature and group membership
3. **Processing**: Pass to nostr_mls for MLS decryption
4. **Storage**: Cache decrypted message in memory
5. **UI Update**: Send message to UI via channel

### Implementation Strategy

#### For dialog_lib:
- Add event stream handling to RealMlsService
- Implement message caching layer
- Provide both fetch (one-shot) and subscribe (streaming) APIs
- Handle subscription lifecycle (connect, reconnect, cleanup)

#### For dialog_tui:
- Start background task on app initialization
- Process incoming messages without blocking UI
- Update conversation view in real-time
- Show connection status and handle disconnects

#### For dialog_cli:
- Continue using fetch-based approach
- No changes needed - uses same underlying storage

## Testing Strategy

Following whitenoise best practices:

### Test Infrastructure
```rust
struct TestEnvironment {
    relay: EphemeralRelay,
    users: Vec<TestUser>,
}

impl TestEnvironment {
    async fn setup(user_count: usize) -> Result<Self> {
        // 1. Start ephemeral relay
        // 2. Create test users with dialog_lib
        // 3. Connect all users
        // 4. Return configured environment
    }
}
```

### Test Scenarios
1. **Basic Messaging**: Two users create group and exchange messages
2. **Group Scaling**: Test with 10, 50, 100 participants
3. **Network Resilience**: Message delivery with relay failures
4. **State Recovery**: Rejoin groups after data loss
5. **Concurrent Operations**: Multiple users sending simultaneously

### Success Metrics
- 100% message delivery success
- < 500ms message send latency
- < 1s group creation time
- Zero MLS decryption failures
- 99.9% uptime in stress tests

## Implementation Guidelines

### Code Quality Standards
- All new code must have tests
- Functions should be < 50 lines
- Clear error types and handling
- Comprehensive logging
- Performance profiling for hot paths

### Security Considerations
- Never log private keys or messages
- Validate all external inputs
- Use constant-time comparisons
- Implement rate limiting
- Regular security audits

### Development Workflow
1. Feature development in dialog_lib
2. Unit and integration tests
3. TUI implementation
4. CLI implementation  
5. Cross-client testing
6. Performance optimization
7. Documentation updates

## Milestones and Deliverables

### Milestone 1: Group Lifecycle ✅ COMPLETED
- ✓ Deliverable: Working group creation and invite system
- ✓ Success Criteria: Can create groups and process invites via TUI
- ✓ Actual: All group operations working, commands integrated

### Milestone 2: Real-time Messaging (1 week) 🚧 IN PROGRESS
- ⬜ Deliverable: Live message reception and display
- ⬜ Success Criteria: Messages appear instantly in TUI as they're sent
- 🎯 Current: Implementing message fetch and subscription system

### Milestone 3: Cross-Client Messaging (1 week)
- ⬜ Deliverable: TUI ↔ CLI interoperability  
- ⬜ Success Criteria: Messages flow both directions without errors

### Milestone 4: Unified Architecture (1 week)
- ⬜ Deliverable: dialog_cli using dialog_lib exclusively
- ⬜ Success Criteria: Zero code duplication between CLI and TUI

### Milestone 5: Production Ready (1 week)
- ⬜ Deliverable: Tested, optimized, documented system
- ⬜ Success Criteria: Passing all test scenarios, <500ms latency

## Risks and Mitigations

### Technical Risks
1. **MLS State Synchronization**
   - Risk: Clients get out of sync
   - Mitigation: Robust sync protocol, state verification

2. **Relay Reliability**
   - Risk: Message loss due to relay issues
   - Mitigation: Multi-relay redundancy, retry logic

3. **Performance at Scale**
   - Risk: Slowdown with large groups
   - Mitigation: Pagination, lazy loading, caching

### Project Risks
1. **Scope Creep**
   - Risk: Adding features before core is solid
   - Mitigation: Strict phase gates, feature freeze

2. **Testing Complexity**
   - Risk: Hard to test distributed scenarios
   - Mitigation: Invest in test infrastructure early

## Conclusion

The Dialog project is well-architected with clear separation of concerns. The immediate priority is completing the core group management features, proving the architecture with cross-client messaging, then achieving full unification by migrating dialog_cli to use dialog_lib. With disciplined execution of this roadmap, we'll have a production-ready, unified messaging system in 6 weeks.

The key to success is maintaining architectural discipline: all business logic in dialog_lib, UIs as thin presentation layers, and comprehensive testing at every step. By following the whitenoise patterns and focusing on fundamentals first, we'll build a robust foundation for future enhancements.