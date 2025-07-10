# SQLite Storage is Hard: The Real-Time vs Fetch Discrepancy

## The Problem

The real-time messaging experience is completely broken compared to the `/fetch` results:

### What's Happening:
1. **Real-time**: Shows duplicate messages, wrong order, and messages appearing multiple times
2. **Fetch**: Shows correct, deduplicated messages in proper order

### Evidence from Screenshots:
- Alice sends "heyo" → appears once in real-time but correctly once in fetch
- Bob's "yooo" appears THREE TIMES in real-time on Bob's side
- Messages like "oh PLEASE" appear twice in Alice's real-time view
- The fetch results are perfect - no duplicates, correct order

## Root Cause Analysis

The issue is that we have **two different code paths** for displaying messages:

1. **Real-time handler** (dialog_lib/src/mls_service.rs:944-993): 
   - Processes incoming events
   - Gets ALL messages from storage after processing
   - Tries to figure out which ones are "new" to display
   - This is BROKEN

2. **Fetch handler** (dialog_lib/src/mls_service.rs:1059-1143):
   - Processes events
   - Gets all messages from storage
   - Returns them all (with proper deduplication via event IDs)
   - This WORKS

## The Dumb Code We're Doing

Looking at the real-time handler (lines 960-987), here's the insanity:

```rust
// Get all messages from storage (they have event IDs!)
if let Ok(messages) = nostr_mls.get_messages(&group.mls_group_id).await {
    // Get displayed messages for this group
    let mut displayed_msgs = displayed_messages_clone.write().await;
    let group_displayed = displayed_msgs.entry(group.mls_group_id.clone())
        .or_insert_with(std::collections::HashSet::new);
    
    // Find messages that haven't been displayed yet
    for msg in messages.iter() {
        let msg_id = msg.id.to_hex();
        if !group_displayed.contains(&msg_id) {
            // This is a new message we haven't displayed
            group_displayed.insert(msg_id.clone());
            
            let message = Message {
                sender: msg.pubkey,
                content: msg.content.clone(),
                timestamp: msg.created_at.as_u64() as i64,
                id: Some(msg_id),
            };
            
            // Send UI update
            let _ = ui_sender.send(UiUpdate::NewMessage {
                group_id: group.mls_group_id.clone(),
                message,
            }).await;
        }
    }
}
```

**THE PROBLEM**: When we process an event, we get ALL messages from storage and try to figure out which ones are "new". But:
- We might process the same event multiple times (relay duplicates, reconnections)
- The "displayed_messages" tracking is in-memory and gets reset on restart
- We're iterating through ALL messages every time an event comes in

## Why Ephemeral Mode Probably Works

In ephemeral mode:
- Storage starts empty each session
- No persistence means no old messages to confuse things
- The "displayed_messages" tracking matches the actual state

## Solution Options

### Option 1: Delete the Real-Time Handler's "Smart" Logic (RECOMMENDED)
**Concept**: The real-time handler should ONLY notify that a new message exists in a group, not try to figure out which message it is.

```rust
// In real-time handler, after processing:
if let Ok(_) = nostr_mls.process_message(&event).await {
    // Just notify that the group has new messages
    let _ = ui_sender.send(UiUpdate::GroupHasNewMessages {
        group_id: group.mls_group_id.clone(),
    }).await;
}
```

Then the UI would:
- Mark the conversation as having unread messages
- When user switches to that conversation, do a proper fetch
- Real-time feel but with correct data

**Pros**: 
- Deletes all the broken deduplication logic
- Simple and correct
- No more duplicate messages

**Cons**: 
- Slightly less "instant" - you'd see a notification but not the actual message until you fetch

### Option 2: Process Only the Current Event's Message
**Concept**: Instead of getting ALL messages and guessing, the real-time handler should only send the message that was just decrypted from the current event.

Problem: The nostr-mls library doesn't expose which message corresponds to which event. We'd need to modify nostr-mls to return the decrypted message directly from `process_message()`.

### Option 3: Track Event IDs Properly in Storage
**Concept**: Store which event IDs we've already displayed in SQLite, not in memory.

```sql
CREATE TABLE displayed_events (
    event_id TEXT PRIMARY KEY,
    group_id TEXT NOT NULL,
    displayed_at INTEGER NOT NULL
);
```

**Pros**: 
- Would survive restarts
- Could properly deduplicate

**Cons**: 
- More complexity
- Still doesn't solve the "get ALL messages every time" performance issue

### Option 4: Trust the Storage Layer Completely
**Concept**: Real-time handler doesn't send ANY UI updates. Instead:
1. Process the event (decrypt and store)
2. The UI polls or watches the storage for changes
3. UI always shows what's in storage

This is basically making everything work like `/fetch`.

## Testing Ephemeral Mode

To test if ephemeral mode still works:
```bash
./target/debug/dialog_tui --key alice --ephemeral
./target/debug/dialog_tui --key bob --ephemeral
```

Expected: Should work fine because there's no persistent storage to confuse the deduplication logic.

**UPDATE**: Confirmed ephemeral mode starts correctly:
- ✅ Both Alice and Bob start in ephemeral mode
- ✅ Key packages are published fresh each time
- ✅ Shows warning about HPKE keys being lost on restart
- ✅ The deduplication logic should work because there's no old data in storage to confuse things

The ephemeral mode likely works correctly for real-time messaging because:
- Storage starts empty
- No persistent messages to iterate through
- The "displayed_messages" tracking matches reality

## My Recommendation

**GO WITH OPTION 1** - Delete the "smart" real-time logic. Here's why:

1. It's the simplest fix
2. It deletes the most code
3. It's guaranteed to be correct
4. The user experience is still good (notification → fetch on view)
5. We can always add back smarter real-time later if needed

The current code is trying to be too clever. When you process an MLS message, you don't know which message in the group it corresponds to without complex correlation. Let's stop trying to guess and just notify that something changed.

## Alternative Quick Fix

If you really want to keep instant messaging, here's a hack:
- In the real-time handler, only show messages from OTHER people (not self)
- Track sent message IDs separately to never show them in real-time
- This would at least prevent the "own message duplication" issue

But honestly, this is still a band-aid on fundamentally broken logic.