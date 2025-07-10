# Dialog TUI UX Improvement Notes

Based on the interop testing session, here are observations and suggestions for improving the Dialog TUI user experience.

## Current Strengths
- Real-time message updates work smoothly
- Clear visual feedback for successful operations (✅ icons)
- Auto-switching to newly joined groups is helpful
- Good use of emoji indicators for different states

## Areas for Improvement

### 1. Message Display & Formatting

**Issue**: Sender public keys are truncated (`75427ab8...`) which makes it hard to distinguish between users
- **Suggestion**: Show contact names when available, fall back to truncated pubkey
- **Suggestion**: Add color coding for different participants
- **Suggestion**: Consider showing bech32 npub format for consistency with other Nostr apps

**Issue**: Timestamps show only time `[23:06]`, not date
- **Suggestion**: Show relative timestamps ("2 min ago", "Yesterday 3:45pm")
- **Suggestion**: Show full timestamp on hover/selection

### 2. Navigation & Commands

**Issue**: The ephemeral mode warnings take up significant screen space and persist
- **Suggestion**: Collapse warnings after initial display, show a small indicator instead
- **Suggestion**: Add a `/dismiss` command to clear old status messages

**Issue**: No clear indication of available commands without typing `/help`
- **Suggestion**: Add a status bar showing common commands (like vim: "Press / for commands, ? for help")
- **Suggestion**: Context-sensitive command hints based on current state

### 3. Group & Contact Management

**Issue**: No way to see group members or group info after joining
- **Suggestion**: Add `/info` command to show current group details
- **Suggestion**: Add `/members` to list group participants

**Issue**: Contact management is disconnected from messaging
- **Suggestion**: Auto-add group members as contacts with their names from group context
- **Suggestion**: Show contact names in message display

### 4. Status & Feedback

**Issue**: Multiple status messages stack up and clutter the display
- **Suggestion**: Implement a rolling status log that shows only the last 3-5 messages
- **Suggestion**: Add message categories (info, warning, error) with different styling

**Issue**: "Total messages: 38" counter seems incorrect/confusing
- **Suggestion**: Show per-conversation message count
- **Suggestion**: Or remove this metric if not meaningful

### 5. Invite Flow

**Issue**: The invite modal covers important context
- **Suggestion**: Show invites in a sidebar or bottom panel instead of modal
- **Suggestion**: Allow viewing conversation while deciding on invites

**Issue**: After accepting invite, previous status messages remain
- **Suggestion**: Clear the message area when switching conversations
- **Suggestion**: Or clearly separate old messages with a divider

### 6. Visual Hierarchy

**Issue**: Important information gets lost in the status messages
- **Suggestion**: Use consistent zones:
  - Top: Current context (group name, connection status)
  - Middle: Messages only
  - Bottom: Input and command hints
  - Sidebar (optional): Contacts, groups, invites

**Issue**: Key package event IDs are shown but not useful for users
- **Suggestion**: Hide technical details unless in debug mode
- **Suggestion**: Show simple "Key packages published ✅" instead

### 7. Error Handling

**Issue**: No clear recovery path when things go wrong
- **Suggestion**: Add connection retry indicators
- **Suggestion**: Show clear error messages with suggested actions

### 8. Quality of Life Features

**Missing Features**:
- Message search (`/search <term>`)
- Message history pagination (currently seems to load everything)
- Typing indicators
- Read receipts or delivery confirmations
- Message editing/deletion
- Rich text or markdown support
- File/image sharing

**Suggested Additions**:
- `/clear` command to clear current view
- `/mute` to temporarily hide a conversation
- `/pin` to pin important messages
- Tab completion for commands and user names
- Command history (up/down arrows)

### 9. Onboarding

**Issue**: New users see technical warnings about ephemeral mode immediately
- **Suggestion**: Add a welcome screen with simple setup flow
- **Suggestion**: Explain ephemeral vs persistent mode in user-friendly terms
- **Suggestion**: Provide guided first steps

### 10. Performance & Reliability

**Observations**:
- Message delivery is fast and reliable
- Real-time updates work well
- Connection status is clearly indicated

**Suggestions**:
- Add reconnection logic with exponential backoff
- Show pending/sending status for outgoing messages
- Cache recent messages for faster conversation switching

## Priority Recommendations

1. **High Priority**: Improve message display with contact names and better formatting
2. **High Priority**: Reduce visual clutter from status messages
3. **Medium Priority**: Add group info and member list commands
4. **Medium Priority**: Implement command hints and better navigation
5. **Low Priority**: Add rich messaging features (search, editing, etc.)

## Technical Considerations

- The ephemeral storage model is a key constraint that affects UX
- Consider offering a hybrid mode where contacts/metadata persist but messages remain ephemeral
- The real-time subscription model works well and should be preserved
- Integration with dialog_lib provides good architectural consistency