# Lessons Learned: MLS + Nostr + SQLite Storage

## The Journey to Fix Message Duplication

This document captures the hard-won lessons from debugging a complex message duplication issue in a Nostr MLS messaging system with SQLite storage.

## The Core Problem

When switching from ephemeral (in-memory) storage to persistent SQLite storage, messages started appearing multiple times in the UI. The same message would show up 2-4 times with identical content but different timestamps. This was maddening because the system worked perfectly in ephemeral mode.

## Key Discoveries

### 1. MLS Messages Are Processed Twice (By Design!)

**Critical Insight**: When you send an MLS message, you MUST process it locally first for state synchronization. This is not optional - it's how MLS maintains cryptographic state consistency.

```rust
// When sending a message:
let message_event = nostr_mls.create_message(group_id, rumor).await?;

// This is REQUIRED - you must process your own message locally
let _ = nostr_mls.process_message(&message_event).await?;

// Then send to relay
client.send_event(&message_event).await?;
```

**The Trap**: That same message comes back from the relay and gets processed AGAIN by the real-time handler, creating a duplicate in storage.

### 2. Storage Persistence Exposes Design Flaws

**Ephemeral Mode Masks Problems**: 
- Everything starts fresh each session
- No old messages to confuse the system
- In-memory tracking aligns with actual state

**SQLite Mode Reveals Truth**:
- Messages persist across restarts
- Processing the same event multiple times creates duplicates
- The storage layer doesn't inherently prevent duplicates

### 3. Event IDs Are Your Friend

**The Revelation**: The `nostr-mls-sqlite-storage` crate DOES track event IDs with each message! We spent hours trying to correlate events to messages when the correlation was already there.

```rust
// Each message in storage has an event ID
pub struct StoredMessage {
    pub id: EventId,  // This is the Nostr event ID!
    pub content: String,
    pub sender: PublicKey,
    // ...
}
```

### 4. The Real-Time vs Fetch Dichotomy

We discovered there were two completely different code paths:

1. **Real-time handler**: Tried to be "smart" about which messages were new
2. **Fetch command**: Just returned everything from storage

The real-time handler's "smart" logic was the source of all problems:
- It would get ALL messages from storage after processing an event
- It tried to figure out which ones were "new" using content matching
- This failed spectacularly with duplicates, timing issues, and persistence

### 5. The Solution: Trust the Storage Layer

**Option 4 Won**: Instead of trying to be clever, we made the real-time handler work like fetch:

```rust
// OLD: Complex deduplication logic trying to guess which message is new
if let Ok(messages) = nostr_mls.get_messages(&group.mls_group_id).await {
    for msg in messages.iter() {
        if !displayed_messages.contains(&msg.id) {
            // Send UI update...
        }
    }
}

// NEW: Just notify that the group has messages
let _ = ui_sender.send(UiUpdate::GroupHasNewMessages {
    group_id: group.mls_group_id.clone(),
}).await;
```

Then the UI re-fetches all messages from storage when notified. Simple, correct, no duplication.

### 6. The Final Fix: Don't Process Your Own Messages Twice

```rust
// In the real-time handler, check if we already processed this when sending
if let Some(group_displayed) = displayed_msgs.get(&group.mls_group_id) {
    if group_displayed.contains(&event_id_hex) {
        // Skip processing - we already handled this when sending
        continue;
    }
}
```

## Architecture Lessons

### 1. MLS Requires Local Processing
- You MUST process your own messages locally when sending
- This maintains cryptographic state consistency
- The relay is just a transport - the real work happens locally

### 2. Storage Is Not Just Cache
- In ephemeral mode, storage and cache are effectively the same
- In persistent mode, storage outlives your in-memory state
- Design with persistence in mind from the start

### 3. Event Correlation Is Critical
- Every MLS message corresponds to a Nostr event
- The event ID is the unique identifier
- Don't try to correlate using content - use the event ID

### 4. Simple Beats Clever
- The "clever" real-time optimization caused all our problems
- The simple "just fetch everything" approach works perfectly
- Optimize later, after correctness is proven

## What Nostr-MLS Does

1. **Encrypts/Decrypts**: Handles all MLS cryptographic operations
2. **Stores Messages**: Persists decrypted messages with their event IDs
3. **Maintains State**: Keeps MLS group state synchronized
4. **Processes Events**: Takes Nostr events and extracts MLS messages

## What Nostr-MLS Doesn't Do

1. **Deduplication**: It will happily store the same message multiple times
2. **Correlation**: Doesn't tell you which decrypted message came from which event (you have to track this yourself)
3. **Real-time Updates**: Just stores messages - UI updates are your problem

## The Working Architecture

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│   Send      │────────▶│   Process   │────────▶│   Store     │
│  Message    │         │   Locally   │         │  (SQLite)   │
└─────────────┘         └─────────────┘         └─────────────┘
       │                                                ▲
       │                                                │
       ▼                                                │
┌─────────────┐         ┌─────────────┐                │
│   Send to   │────────▶│   Receive   │                │
│    Relay    │         │  from Relay │                │
└─────────────┘         └─────────────┘                │
                               │                        │
                               ▼                        │
                        ┌─────────────┐                │
                        │  Check if   │                │
                        │  Already    │────NO──────────┘
                        │  Processed  │
                        └─────────────┘
                               │
                              YES
                               │
                               ▼
                        ┌─────────────┐
                        │    Skip     │
                        └─────────────┘
```

## Debugging Tips

1. **Log Event IDs**: Always log event IDs when processing
2. **Check Storage**: Look directly at what's in SQLite
3. **Test Ephemeral First**: Verify behavior works without persistence
4. **Trace Both Paths**: Follow messages through both send and receive
5. **Trust the Fetch**: If fetch shows duplicates, you're storing duplicates

## Future Improvements

1. **Event-Level Deduplication**: Store processed event IDs in SQLite
2. **Batch Processing**: Process multiple events before notifying UI
3. **Optimistic UI**: Show sent messages immediately without fetch
4. **Better Correlation**: Modify nostr-mls to return event-message pairs

## The Zen of MLS + Nostr

- Your message will come back to haunt you (from the relay)
- Process once, not twice
- Event IDs are the source of truth
- Storage is forever (in SQLite mode)
- When in doubt, fetch it out

## Final Wisdom

> "The bug is not in the stars, but in ourselves - specifically in trying to be clever when simple would suffice."

The entire issue boiled down to processing the same message twice and not tracking that we'd already seen it. Hours of debugging, dozens of failed attempts, and the fix was just: "Hey, did we already process this? Then don't do it again."

Sometimes the best code is the code you delete.