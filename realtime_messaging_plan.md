# Real-Time Messaging Implementation Plan

## UPDATE: Real-Time Messaging is Already Implemented! 🎉

After investigation, I discovered that real-time messaging is **fully implemented** but may have a bug in the subscription setup.

### What's Already Working ✅
1. **WebSocket subscriptions are active** - Connected on startup
2. **Real-time invite notifications work** - New invites appear instantly
3. **Background event processing** - `subscribe_to_groups()` is running
4. **UI update channel** - `UiUpdate` enum with `NewMessage` variant
5. **Message handling in `check_ui_updates()`** - Already processes `UiUpdate::NewMessage`
6. **Message subscription filters** - ✅ ALREADY subscribing to group messages!
7. **Message decryption on arrival** - ✅ ALREADY processing and decrypting!
8. **UI updates** - ✅ ALREADY sending to UI!

### Bug Found and Fixed 🐛
The issue was in `subscribe_to_groups()`:
```rust
// OLD (bug - overwrites filters)
for filter in filters {
    client.subscribe_with_id(subscription_id.clone(), filter, None).await?;
}

// NEW (fixed - sends all filters at once)
client.subscribe_with_id(subscription_id.clone(), filters, None).await?;
```

### What's Actually Happening

The complete flow is already implemented in `dialog_lib/src/mls_service.rs`:

1. **Subscription Setup** (lines 730-760)
   - Gets all groups
   - Creates filters for each group's messages
   - Subscribes to gift wraps for invites
   - Fixed: Now sends all filters at once

2. **Message Processing** (lines 780-825)
   - Receives MLS group messages
   - Processes with `nostr_mls.process_message()`
   - Extracts group ID from tags
   - Gets decrypted message content
   - Caches the message
   - Sends `UiUpdate::NewMessage` to UI

3. **UI Update** (app.rs lines 969-992)
   - Receives `UiUpdate::NewMessage`
   - Checks if message is for active conversation
   - Formats sender name
   - Adds message to display

## Debug Logging Added

To help diagnose any remaining issues, I've added debug logging:

1. **Subscription setup**: `📡 Setting up real-time subscriptions for X groups`
2. **Message receipt**: `📨 Received MLS group message event: <id>`
3. **UI update sent**: `✅ Sent real-time message to UI`

## Testing Instructions

1. **Start relay with verbose logging**:
   ```bash
   nak serve --verbose
   ```

2. **Start two TUI instances**:
   ```bash
   # Terminal 1 (Alice)
   cargo run -- --key alice
   
   # Terminal 2 (Bob)
   cargo run -- --key bob
   ```

3. **Create a group and test**:
   - Alice: `/create TestGroup` and select Bob
   - Bob: `/invites` and accept
   - Both: Send messages without using `/fetch`

4. **Watch the terminal output** for debug messages

## Potential Issues

1. **Subscription timing** - Groups created after startup won't have filters
   - **CONFIRMED**: This is the main issue!
   - When you create/join a group, subscriptions aren't updated
   - Messages only appear after restart (when subscriptions are recreated)

2. **Multiple instances** - Same key in multiple places might cause issues
3. **Relay configuration** - Some relays might not support all filter types

## The Real Fix Needed

We need to update subscriptions when:
1. Creating a new group
2. Accepting an invite
3. Leaving a group

### Solution Options

1. **Restart subscription with new filters** (recommended)
   - Cancel old subscription
   - Create new one with updated group list

2. **Add filters to existing subscription** (if supported by nostr-sdk)
   - More efficient but may not be supported

3. **Create separate subscription per group**
   - More granular control
   - More complex to manage

## Implementation Plan

### Step 1: Fix Message Subscription Filters

In `RealMlsService::subscribe_to_groups()`:

```rust
// Current: Only subscribing to gift wraps
let filters = vec![
    Filter::new()
        .kind(Kind::GiftWrap)
        .pubkey(self.keys.public_key())
];

// Need to add: MLS group message filters
for group in groups {
    filters.push(
        Filter::new()
            .kind(Kind::MlsGroupMessage)
            .custom_tag(SingleLetterTag::lowercase(Alphabet::H), 
                       hex::encode(&group.nostr_group_id))
    );
}
```

### Step 2: Process Incoming Messages

Add message processing to the subscription handler:

```rust
// In the event processing loop
match event.kind {
    Kind::GiftWrap => {
        // Existing invite processing
    }
    Kind::MlsGroupMessage => {
        // Extract group ID from tags
        let group_id = extract_group_id_from_tags(&event)?;
        
        // Process with nostr_mls to decrypt
        match nostr_mls.process_message(&event) {
            Ok(decrypted) => {
                // Convert to our Message type
                let message = Message {
                    sender: event.author,
                    content: extract_content(&decrypted),
                    timestamp: event.created_at.as_i64(),
                    id: Some(event.id.to_hex()),
                };
                
                // Send to UI
                ui_sender.send(UiUpdate::NewMessage {
                    group_id,
                    message,
                }).await?;
            }
            Err(e) => {
                // Log but don't fail - might be for a group we left
                tracing::debug!("Failed to decrypt message: {}", e);
            }
        }
    }
}
```

### Step 3: Message Caching

Update the in-memory cache when messages arrive:

```rust
// In RealMlsService
impl RealMlsService {
    async fn cache_message(&self, group_id: &GroupId, message: Message) {
        let mut cache = self.message_cache.write().await;
        let messages = cache.entry(group_id.clone()).or_insert_with(Vec::new);
        
        // Insert in sorted order by timestamp
        match messages.binary_search_by_key(&message.timestamp, |m| m.timestamp) {
            Ok(pos) | Err(pos) => messages.insert(pos, CachedMessage {
                message: message.clone(),
                event_id: message.id.clone().unwrap_or_default().into(),
            }),
        }
        
        // Limit cache size per group
        if messages.len() > MAX_CACHED_MESSAGES {
            messages.drain(0..messages.len() - MAX_CACHED_MESSAGES);
        }
    }
}
```

### Step 4: Update Fetch to Use Cache

Make `/fetch` instant by using cached messages:

```rust
async fn fetch_messages(&self, group_id: &GroupId) -> Result<MessageFetchResult> {
    // First return cached messages immediately
    let cache = self.message_cache.read().await;
    if let Some(cached) = cache.get(group_id) {
        let messages = cached.iter()
            .map(|c| c.message.clone())
            .collect();
        return Ok(MessageFetchResult {
            messages,
            processing_errors: vec![],
        });
    }
    
    // If no cache, fetch from relay (existing code)
    ...
}
```

### Step 5: Handle Edge Cases

1. **Duplicate Detection**
   - Check event ID before adding to cache
   - Prevent duplicate UI updates

2. **Out-of-order Messages**
   - Sort by timestamp when displaying
   - Handle clock skew gracefully

3. **Large Groups**
   - Limit subscription filters (max N groups)
   - Pagination for message history

4. **Reconnection**
   - Re-establish subscriptions on disconnect
   - Fetch missed messages during downtime

## Testing Plan

1. **Two-client Test**
   - Start two TUI instances (Alice and Bob)
   - Create group, send messages
   - Verify instant delivery without `/fetch`

2. **Message Ordering**
   - Send rapid messages
   - Verify correct order

3. **Reconnection Test**
   - Disconnect one client
   - Send messages
   - Reconnect and verify backfill

## Code Locations

- **Subscription setup**: `dialog_lib/src/mls_service.rs` - `subscribe_to_groups()`
- **Event processing**: Need to add to subscription handler
- **Message caching**: Already have `message_cache` field, need to use it
- **UI updates**: `dialog_tui/src/app.rs` - `check_ui_updates()` already handles it

## Next Steps

1. Start with Step 1 - Add group message filters to subscription
2. Test with existing infrastructure to see what breaks
3. Add message processing incrementally
4. Verify with two-client test

The beauty is that most of the infrastructure is already there - we just need to connect the dots!