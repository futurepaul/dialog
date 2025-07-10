# Whitenoise-Dialog Interoperability PRD

## Executive Summary

This PRD outlines the comprehensive plan for achieving true interoperability between **whitenoise** (production-grade persistent client) and the **dialog suite** (dialog_tui + dialog_cli using unified dialog_lib architecture). Success means users from any client can seamlessly create groups, invite members, and exchange messages in real-time.

**Key Challenge**: Two different MLS implementations with different storage models, message flows, and architectural approaches need to communicate flawlessly over the Nostr protocol.

**Phase Approach**: This work builds on the foundation established by the [Dialog CLI Rewrite PRD](dialog_cli_rewrite_prd.md), which first unifies the dialog architecture and proves interop patterns between dialog_tui and dialog_cli.

## Table of Contents

1. [Current State Analysis](#current-state-analysis)
2. [Technical Architecture Comparison](#technical-architecture-comparison)
3. [Compatibility Issues & Solutions](#compatibility-issues--solutions)
4. [Test Scenarios & Automation](#test-scenarios--automation)
5. [Implementation Plan](#implementation-plan)
6. [Success Metrics](#success-metrics)
7. [Risk Assessment](#risk-assessment)

---

## Current State Analysis

### Whitenoise Architecture
- **Storage**: Persistent SQLite storage for accounts, groups, messages
- **MLS Implementation**: Full-featured with reactions, replies, deletions  
- **Relay Management**: Multi-relay support (Nostr, Inbox, KeyPackage relays)
- **Welcome Processing**: Sophisticated gift-wrap handling with state management
- **Testing**: Comprehensive integration test binary (`integration_test.rs`)

### Dialog Suite Architecture (Post CLI Rewrite)
- **Unified Library**: Both dialog_tui and dialog_cli use dialog_lib
- **Storage Options**: Memory (TUI) and SQLite (CLI) via storage abstraction
- **MLS Implementation**: Comprehensive feature set via dialog_lib::MlsService
- **Relay Management**: Configurable multi-relay support
- **Welcome Processing**: Dual compatibility (gift-wrapped + direct MLS)
- **Testing**: ht-mcp automation for TUI + CLI automation + proven interop patterns

### Prerequisites Established
✅ **Dialog-Internal Interop**: dialog_tui ↔ dialog_cli interoperability proven  
✅ **Storage Validation**: Both memory and SQLite storage backends validated  
✅ **Automation Framework**: ht-mcp automation patterns established  
✅ **Test Infrastructure**: Comprehensive interop testing suite created

### Current Compatibility Status (Whitenoise ↔ Dialog)
✅ **Working**: Both use compatible nostr-mls library foundations  
✅ **Working**: Both support MLS wire protocol standards  
✅ **Working**: Dialog suite interop patterns proven and tested
⚠️ **Partial**: Welcome message handling differs between whitenoise and dialog
⚠️ **Partial**: Group ID mapping inconsistencies (whitenoise vs dialog_lib)
⚠️ **Partial**: Different subscription patterns (background vs real-time)
❌ **Unknown**: Key package lifecycle compatibility needs testing  

---

## Technical Architecture Comparison

### MLS Event Types & Formats

| Component | Whitenoise | Dialog Suite | Compatibility |
|-----------|------------|--------------|---------------|
| **Key Packages** | `Kind::MlsKeyPackage` | `Kind::MlsKeyPackage` | ✅ Compatible |
| **Group Messages** | `Kind::MlsGroupMessage` | `Kind::MlsGroupMessage` | ✅ Compatible |
| **Welcome Messages** | `Kind::GiftWrap` only | `Kind::GiftWrap` + `Kind::MlsWelcome` | ⚠️ Needs alignment |
| **Group Tagging** | `h` tag with Nostr Group ID | `h` tag with Nostr Group ID | ✅ Compatible |
| **Message Routing** | Background event processor | Real-time subscriptions | ⚠️ Different patterns |
| **Storage** | SQLite only | Memory + SQLite | ✅ Compatible backends |

### Group Creation Flows

#### Whitenoise Process:
1. Fetch key packages from member key package relays
2. Create MLS group with admin configuration
3. Generate welcome rumors per member
4. Send gift-wrapped welcomes to inbox relays
5. Publish evolution event to group relays
6. Setup background subscriptions

#### Dialog Suite Process:
1. Member selection (interactive TUI or CLI arguments)
2. Fetch key packages via dialog_lib from configured relays
3. Create MLS group via dialog_lib (creator = admin)
4. Send dual-format welcomes (gift-wrapped + direct)
5. Auto-refresh subscriptions for new group
6. Real-time message display (TUI) or on-demand fetch (CLI)

### Key Package Lifecycle

| Aspect | Whitenoise | Dialog Suite | Impact |
|--------|------------|--------------|---------|
| **Storage** | Persistent (SQLite) | Memory (TUI) + SQLite (CLI) | TUI still needs fresh packages, CLI can persist |
| **Publishing** | Automatic on account creation | Manual commands (both TUI/CLI) | Timing coordination required |
| **Expiration** | Long-lived on relays | Session-based (TUI), configurable (CLI) | CLI can match whitenoise patterns |
| **Refresh** | Manual via API | Manual via commands | Coordination needed, but CLI can automate |

---

## Compatibility Issues & Solutions

### Issue 1: Welcome Message Format Mismatch
**Problem**: Whitenoise expects only gift-wrapped welcomes; Dialog_TUI sends both formats  
**Solution**: Enhance whitenoise to process both gift-wrapped and direct MLS welcomes  
**Priority**: HIGH - Blocks group creation flow  

### Issue 2: Group ID Mapping Inconsistencies  
**Problem**: Different internal group ID usage patterns  
**Solution**: Standardize on Nostr Group ID for UI, MLS Group ID for protocol  
**Priority**: HIGH - Breaks message routing  

### Issue 3: Subscription Pattern Differences
**Problem**: Background processing vs real-time subscriptions  
**Solution**: Ensure both patterns handle same event types consistently  
**Priority**: MEDIUM - Affects message delivery timing  

### Issue 4: Key Package Timing Dependencies
**Problem**: Dialog's ephemeral packages may expire before group creation  
**Solution**: Add pre-flight key package validation and refresh logic  
**Priority**: HIGH - Causes group creation failures  

### Issue 5: Evolution Event Processing
**Problem**: Different MLS state synchronization approaches  
**Solution**: Ensure both clients process evolution events before message sending  
**Priority**: MEDIUM - Affects group consistency  

---

## Test Scenarios & Automation

### Scenario 1: Whitenoise Creates Group, Invites Dialog_TUI

**Setup**:
```bash
# Terminal 1: Start infrastructure
cd ~/dev/heavy/whitenoise
docker compose up  # Starts relays on ports 7777, 8080

# Terminal 2: Start dialog_tui 
cd ~/dev/heavy/denoise
nak serve --verbose &  # Backup relay on 10547
DIALOG_RELAY_URLS=ws://localhost:8080,ws://localhost:7777 cargo run --bin dialog_tui -- --key alice

# Terminal 3: ht-mcp automation for dialog_tui
ht_create_session --command dialog_tui_automation
```

**ht-mcp Dialog_TUI Automation Script**:
```python
# dialog_tui_automation.py
import ht_mcp_client

async def setup_dialog_tui():
    session = await ht_mcp_client.create_session()
    
    # Connect to relay
    await session.send_keys(["/connect", "Enter"])
    await session.wait_for_text("Connected")
    
    # Publish key packages  
    await session.send_keys(["/keypackage", "Enter"])
    await session.wait_for_text("Published")
    
    # Get pubkey for whitenoise invitation
    await session.send_keys(["/pk", "Enter"])
    pubkey = await session.extract_text_matching(r"Hex: ([a-f0-9]{64})")
    
    return pubkey

async def accept_invite_and_chat():
    # Check for invites
    await session.send_keys(["/invites", "Enter"])
    
    # Accept first invite (arrow keys + Enter)
    await session.send_keys(["Enter"])
    await session.wait_for_text("Successfully joined")
    
    # Send test message
    await session.send_keys(["Hello from dialog_tui!", "Enter"])
    
    # Fetch messages to verify
    await session.send_keys(["/fetch", "Enter"])
    return await session.capture_messages()
```

**Whitenoise Integration Test Flow**:
```rust
// Enhanced integration_test.rs
#[tokio::test]
async fn test_dialog_tui_interop() -> Result<()> {
    // 1. Setup whitenoise account
    let whitenoise = setup_test_environment().await?;
    let alice_account = create_test_account("alice").await?;
    
    // 2. Get dialog_tui pubkey from ht-mcp automation
    let dialog_pubkey = get_dialog_tui_pubkey().await?;
    
    // 3. Create group with dialog_tui as member
    let group_id = whitenoise.create_group(
        &alice_account,
        vec![dialog_pubkey],  // members
        vec![alice_account.pubkey],  // admins
        test_group_config()
    ).await?;
    
    // 4. Send welcome and wait for acceptance
    wait_for_group_member_join(&group_id, &dialog_pubkey).await?;
    
    // 5. Exchange messages bi-directionally
    whitenoise.send_message_to_group(&alice_account, &group_id, "Hello from whitenoise!").await?;
    let messages = wait_for_dialog_response(&group_id).await?;
    
    // 6. Verify message exchange
    assert!(messages.iter().any(|m| m.content.contains("Hello from dialog_tui!")));
    
    Ok(())
}
```

### Scenario 2: Dialog_TUI Creates Group, Invites Whitenoise

**ht-mcp Group Creation Flow**:
```python
async def create_group_and_invite_whitenoise(whitenoise_pubkey):
    # Add contact first
    await session.send_keys([f"/add {whitenoise_pubkey}", "Enter"])
    await session.wait_for_text("Contact added")
    
    # Create group with interactive selection
    await session.send_keys(["/create TestGroup", "Enter"])
    
    # Navigate to contact and select with space
    await session.send_keys([" ", "Enter"])  # Select + confirm
    await session.wait_for_text("Group 'TestGroup' created successfully")
    
    # Send initial message
    await session.send_keys(["Welcome to the group!", "Enter"])
    
    return extract_group_id()
```

**Whitenoise Welcome Processing**:
```rust
async fn accept_dialog_invite_and_respond() -> Result<()> {
    // 1. Fetch pending welcomes (should find dialog's invitation)
    let welcomes = whitenoise.fetch_welcomes(&alice_account).await?;
    let dialog_welcome = welcomes.iter()
        .find(|w| w.group_name == "TestGroup")
        .ok_or("Dialog invite not found")?;
    
    // 2. Accept the welcome
    whitenoise.accept_welcome(&alice_account, dialog_welcome.event_id.clone()).await?;
    
    // 3. Send response message
    let group_id = GroupId::from(dialog_welcome.group_id.clone());
    whitenoise.send_message_to_group(&alice_account, &group_id, "Thanks for the invite!").await?;
    
    Ok(())
}
```

### Scenario 3: Bi-directional Message Exchange

**Test Message Flow**:
1. **Setup Phase**: Both clients connected, group established
2. **Message Burst**: Send multiple messages from both sides rapidly
3. **Evolution Events**: Add/remove members while messaging
4. **Error Recovery**: Simulate network issues, reconnection
5. **Long-lived Session**: Test session persistence over time

**Verification Points**:
- ✅ All messages received by both clients
- ✅ Message order consistency  
- ✅ MLS state synchronization
- ✅ Evolution event processing
- ✅ Error recovery capabilities

---

## Implementation Plan

**Prerequisites**: Complete [Dialog CLI Rewrite PRD](dialog_cli_rewrite_prd.md) phases 1-4 first.

### Phase 1: Dialog-Whitenoise Integration Foundation (Week 1)
- [ ] **Extend interop test framework** to support three-way testing (TUI + CLI + Whitenoise)
- [ ] **Create whitenoise automation helpers** similar to ht-mcp patterns
- [ ] **Setup unified relay infrastructure** for all three clients
- [ ] **Establish cross-client verification patterns** building on dialog interop

### Phase 2: Welcome Message Compatibility (Week 2)  
- [ ] **Update whitenoise** to process both gift-wrapped and direct MLS welcomes
- [ ] **Enhance dialog_lib welcome handling** for whitenoise compatibility
- [ ] **Test welcome message roundtrip** across all three clients
- [ ] **Validate welcome timing and retry logic** across implementations

### Phase 3: Protocol Alignment (Week 3)
- [ ] **Align group ID usage patterns** between whitenoise and dialog_lib
- [ ] **Standardize subscription management** patterns
- [ ] **Add key package lifecycle compatibility** testing
- [ ] **Implement cross-client group synchronization** validation

### Phase 4: Comprehensive Interop Testing (Week 4)
- [ ] **Three-way interop scenarios**: TUI ↔ CLI ↔ Whitenoise
- [ ] **Bi-directional messaging verification** across all client pairs
- [ ] **Evolution event compatibility** testing (add/remove members)
- [ ] **Error recovery scenarios** with mixed client types

### Phase 5: Production Readiness (Week 5)
- [ ] **Performance testing** under realistic loads
- [ ] **Long-running stability** testing with mixed sessions
- [ ] **Comprehensive documentation** and troubleshooting guides
- [ ] **CI/CD integration** for ongoing compatibility verification

---

## Success Metrics

### Primary Success Criteria
1. **✅ Universal Group Creation**: Any client can create groups and invite members from other clients
2. **✅ Universal Group Joining**: All clients can accept invitations from any other client
3. **✅ Three-way Messaging**: Messages flow seamlessly between all client combinations
4. **✅ Storage Compatibility**: Both ephemeral (TUI) and persistent (CLI/Whitenoise) patterns work
5. **✅ Session Persistence**: Long-running conversations work reliably across client types

### Secondary Success Criteria  
1. **⚠️ Member Management**: Add/remove members across client types
2. **⚠️ Admin Operations**: Admin permissions work across clients
3. **⚠️ Error Recovery**: Graceful handling of network issues
4. **⚠️ Performance**: Sub-second message delivery under normal conditions
5. **⚠️ Monitoring**: Comprehensive logging for debugging issues

### Quality Metrics
- **Reliability**: 99% message delivery success rate
- **Performance**: <1s message delivery latency  
- **Compatibility**: Support for all common MLS operations
- **Usability**: Clear error messages and recovery flows

---

## Risk Assessment

### High-Risk Items
1. **MLS State Divergence**: Different clients getting out of sync
   - *Mitigation*: Add state validation checkpoints
   - *Monitoring*: Log MLS state hashes for comparison

2. **Key Package Expiration**: Dialog's ephemeral packages timing out
   - *Mitigation*: Pre-flight validation and auto-refresh
   - *Monitoring*: Key package freshness tracking

3. **Welcome Message Processing**: Format incompatibilities
   - *Mitigation*: Comprehensive welcome format testing
   - *Monitoring*: Welcome delivery success rates

### Medium-Risk Items  
1. **Relay Coordination**: Multiple relays with different data
   - *Mitigation*: Use consistent relay sets for testing
   - *Monitoring*: Relay synchronization health checks

2. **Evolution Event Ordering**: Race conditions in group changes
   - *Mitigation*: Add event ordering validation
   - *Monitoring*: Evolution event sequence tracking

### Low-Risk Items
1. **UI Differences**: Different presentation of same data
   - *Mitigation*: Focus on protocol compatibility
   - *Impact*: Doesn't affect interoperability

2. **Feature Gaps**: One client missing features
   - *Mitigation*: Test common feature subset
   - *Impact*: Graceful degradation acceptable

---

## Development Guidelines

### Testing Philosophy
- **Protocol First**: Focus on MLS wire compatibility over UI consistency
- **Automated Validation**: Use ht-mcp for repeatable TUI testing
- **Real-world Scenarios**: Test actual user workflows, not just happy paths
- **Cross-client Verification**: Validate operations from both client perspectives

### Debugging Strategy
- **Comprehensive Logging**: Both clients log MLS operations with correlation IDs
- **Event Tracing**: Track nostr events through their lifecycle
- **State Dumps**: Capture MLS state at key checkpoints for comparison
- **Network Monitoring**: Use relay logs to verify message delivery

### Quality Assurance
- **Integration CI**: Automated interop testing in CI pipeline
- **Manual Testing**: Regular manual validation of critical flows
- **Performance Monitoring**: Track message delivery times and reliability
- **Regression Testing**: Prevent compatibility breaks with version updates

---

## Conclusion

Achieving true interoperability between whitenoise and the dialog suite represents a significant milestone for the nostr-mls ecosystem. Success will demonstrate that multiple independent implementations can seamlessly communicate, paving the way for broader adoption of the protocol.

### Phased Approach Benefits

**Risk Reduction**: By first proving interoperability between dialog_tui and dialog_cli, we establish:
- Validated automation patterns with ht-mcp
- Proven cross-client testing methodologies  
- Robust storage abstraction (memory + SQLite)
- Comprehensive error handling and recovery

**Technical Foundation**: The dialog CLI rewrite provides:
- Unified architecture across dialog clients
- Proven interop patterns to extend to whitenoise
- Established debugging and verification tools
- Performance baselines and optimization strategies

**Incremental Complexity**: Adding whitenoise as the third client to an already-working two-client system:
- Reduces integration risk significantly
- Allows focus on whitenoise-specific compatibility issues
- Provides fallback patterns when new issues arise
- Enables comprehensive three-way validation

### Key Success Factors

The key to success lies in:
1. **Methodical progression** through the dialog CLI rewrite first
2. **Comprehensive automation** of test scenarios using proven ht-mcp patterns
3. **Protocol-first focus** on core MLS operations that enable secure group communication
4. **Incremental validation** at each step before proceeding to the next phase

**Next Steps**: Complete the [Dialog CLI Rewrite PRD](dialog_cli_rewrite_prd.md) implementation, then proceed with whitenoise integration using the established patterns and infrastructure.