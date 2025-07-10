# Message Duplication Bug in SQLite Mode

## Problem Description

When using SQLite storage (the new default), messages appear duplicated in conversations. The same message content appears multiple times with different timestamps. This did NOT happen in ephemeral mode.

**Evidence**: Screenshots show messages like "yo" appearing 4+ times in the same conversation between Alice and Bob.

## What Works
- ✅ Ephemeral mode (`--ephemeral` flag) works correctly
- ✅ Real-time message updates work (you see incoming messages immediately)
- ✅ `/fetch` command returns messages (but duplicated)
- ✅ Message encryption/decryption works

## What's Broken
- ❌ Messages duplicated in SQLite mode
- ❌ Same content appears multiple times with different timestamps
- ❌ Affects both sides of conversation (Alice and Bob both see duplicates)

## Root Cause Analysis

The issue appears when we switched from ephemeral to SQLite as the default storage. The message caching logic in `dialog_lib/src/mls_service.rs` was designed for ephemeral mode where everything starts fresh on each restart.

### Key Differences:
- **Ephemeral**: Cache and storage both start empty each session
- **SQLite**: Storage persists across sessions, but cache starts empty
- **Mismatch**: Cache thinks it needs to process old events, but storage already has the decrypted messages

## Failed Fix Attempts

### Attempt 1: Clear Cache on Fetch
**File**: `dialog_lib/src/mls_service.rs:1096`
**Change**: Added `message_cache.remove(&group_id);` before fetching
**Result**: ❌ Still duplicated
**Problem**: Clearing cache forces re-processing of all events every time

### Attempt 2: Rewrite Message Handling (Complex)
**Files**: `dialog_lib/src/mls_service.rs` (lines 1090-1160)
**Changes**: 
- Removed `CachedMessage` struct
- Replaced `HashMap<GroupId, Vec<CachedMessage>>` with `HashSet<EventId>`
- Simplified real-time handler
- Tried to use only storage as source of truth
**Result**: ❌ Lost real-time updates, still had issues
**Problem**: Broke working functionality, overly complex

### Attempt 3: Remove Real-time UI Updates
**File**: `dialog_lib/src/mls_service.rs` (real-time handler)
**Change**: Made real-time handler only decrypt, not send UI updates
**Result**: ❌ No real-time messages (bad UX)
**Problem**: Users couldn't see incoming messages without manual `/fetch`

### Attempt 4: Restore Real-time + Event Deduplication
**Files**: `dialog_lib/src/mls_service.rs`
**Changes**: 
- Added event ID checking before processing
- Restored real-time UI updates
- Used `HashSet<EventId>` to track processed events
**Result**: ❌ Still duplicated
**Problem**: Event deduplication didn't solve cache vs storage mismatch

### Attempt 5: Revert to Original + Targeted Fix
**Action**: `git checkout aa678c9 -- dialog_lib/src/mls_service.rs`
**Additional**: Added cache clearing only during fetch
**Result**: ❌ Still duplicated (user confirmed)
**Problem**: The original logic has fundamental issues with SQLite persistence

## SOLUTION FOUND! ✅

After investigating the SQLite storage backend, we discovered that event IDs ARE stored with messages. The fix was to use these event IDs for proper deduplication instead of unreliable content matching.

### The Working Fix (Attempt 6: Event ID Tracking)
**File**: `dialog_lib/src/mls_service.rs`
**Changes**:
1. Added `displayed_messages: Arc<RwLock<HashMap<GroupId, HashSet<String>>>>` to track displayed event IDs per group
2. Updated real-time handler to:
   - Get all messages from storage after processing an event
   - Check each message's event ID against displayed set
   - Only display messages with new event IDs
   - Add displayed IDs to the tracking set
3. Updated `fetch_messages` to:
   - Mark all fetched message IDs as displayed
   - Include event IDs in returned Message structs
**Result**: ✅ NO DUPLICATES! Real-time works perfectly!

### Why This Works
- SQLite storage tracks event IDs in the `messages` table (`id` field)
- Each message has a unique event ID that persists across restarts
- By tracking which event IDs we've displayed, we ensure each message appears exactly once
- No reliance on content matching or `messages.last()` assumptions

## Debugging Information

### Original Working Logic (Ephemeral Mode)
```rust
// In fetch_messages():
// 1. Get cached messages for group
// 2. Track processed event IDs from cache
// 3. For each new event:
//    - Skip if already processed
//    - Call process_message() to decrypt
//    - Get all messages from storage
//    - Find "new" message by matching content/sender
//    - Add to cache
// 4. Return cached messages

// In real-time handler:
// 1. Process event with process_message()
// 2. Get all messages from storage
// 3. Take the "last" message
// 4. Add to cache and send UI update
```

### The Flawed Matching Logic
```rust
// Lines 1127-1133 in mls_service.rs
if let Some(msg) = stored_messages.iter().find(|m| {
    // Find a message that we haven't cached yet
    !cached_messages.iter().any(|cm| 
        cm.message.sender == m.pubkey && 
        cm.message.content == m.content
    )
}) {
    // This is BROKEN - it can match wrong messages!
}
```

**Problems with this approach:**
1. **Content matching is unreliable**: Multiple messages can have same content
2. **Race conditions**: Real-time and fetch can process same event
3. **SQLite persistence**: Old messages in storage confuse the matching
4. **"Last message" assumption**: Real-time handler assumes last message in storage is the new one

## Hypotheses for Real Fix

### Hypothesis 1: Event ID Correlation Missing
The `nostr-mls` library doesn't expose which event ID corresponds to which decrypted message. The current "content matching" is a hack that fails with SQLite.

**Potential Solution**: Store event IDs alongside messages in SQLite, or modify nostr-mls to provide this correlation.

### Hypothesis 2: Double Processing
Events are being processed both in real-time AND during fetch, leading to double decryption and storage.

**Potential Solution**: Implement proper event deduplication at the nostr-mls level.

### Hypothesis 3: Cache-Storage Sync Issues
The in-memory cache and SQLite storage get out of sync, especially across restarts.

**Potential Solution**: Either eliminate the cache entirely and use only storage, or implement proper cache invalidation.

### Hypothesis 4: Real-time Handler Logic Wrong
The real-time handler's assumption that `messages.last()` is the newly decrypted message is incorrect with persistent storage.

**Potential Solution**: Track which messages are new vs old, possibly with timestamps or sequence numbers.

## Files Modified

1. `dialog_tui/src/main.rs` - Added SQLite as default, `--ephemeral` flag
2. `dialog_tui/src/app.rs` - Added invite confirmation modal, auto-switch logic
3. `dialog_tui/src/ui.rs` - Added modal UI for invite confirmation
4. `dialog_lib/src/mls_service.rs` - Multiple failed attempts to fix message caching

## Working Integration Test

The integration test in `integration/README.md` works fine in ephemeral mode:
1. Start `nak serve --verbose`
2. Start Alice with `--ephemeral`
3. Start Bob with CLI (uses SQLite)
4. Create group, send messages
5. Messages appear correctly without duplication

## Recommendations for Real Fix

1. **Test with ephemeral mode first** to confirm current behavior works
2. **Don't modify the real-time handler** - it works correctly in ephemeral mode
3. **Focus on the fetch logic** - specifically the message correlation problem
4. **Consider whitenoise patterns** - they have sophisticated message aggregation
5. **Add logging** to see exactly when/why messages get duplicated
6. **Investigate nostr-mls** - might need changes to expose event-message correlation

## What NOT to Do

- ❌ Don't clear the entire cache on every fetch (performance nightmare)
- ❌ Don't remove real-time updates (breaks UX)
- ❌ Don't rewrite the entire message handling system
- ❌ Don't assume the problem is in the UI layer

## Test Case for Verification

1. Clean databases: `rm ~/Library/Application\ Support/dialog/*.db`
2. Start fresh Alice and Bob with SQLite (default)
3. Create conversation and exchange a few messages
4. Check if messages appear duplicated
5. Run `/fetch` and see if duplication increases
6. Restart one client and repeat - does it get worse?

## Failed Approaches Summary

### Why Initial Fixes Failed:
1. **Cache elimination**: Removed cache entirely but still used `messages.last()` which was unreliable
2. **Content matching**: Multiple messages can have same content, causing false matches
3. **Before/after comparison**: Still relied on content/timestamp matching which wasn't unique
4. **Count-based detection**: Assumed new message would be `last()` which wasn't true
5. **Disabling real-time**: Would have worked but terrible UX

### Root Cause:
The fundamental issue was that the code wasn't using the event IDs that SQLite storage was already tracking! The `nostr-mls-sqlite-storage` properly stores event IDs with each message, but the application layer was trying to correlate events to messages using unreliable content matching.

### Key Insights:
- ✅ SQLite storage DOES track event IDs (in `messages.id` field)
- ✅ Event IDs are unique and persistent
- ✅ Content matching is unreliable (duplicate content, timing issues)
- ✅ `messages.last()` assumption breaks with persistent storage
- ❌ The cache itself wasn't the problem - it was the correlation logic

The fix was surprisingly simple once we discovered that event IDs were available - just track which ones we've displayed!