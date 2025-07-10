# Dialog TUI UX Improvement PRD

## Overview
This is a focused plan for improving Dialog TUI's user experience, prioritizing high-impact changes that maintain the power-user focus while reducing friction.

## Key UX Insights from Testing
- **Visual clutter** from status messages is the biggest pain point
- **Command discoverability** is poor - users need to type `/help` to know what's available
- **Navigation** between conversations needs improvement - the modal overlay approach interrupts flow
- **Message formatting** needs better visual hierarchy for quick scanning

## Priority 1: Core Improvements (2-3 days)

### Command History ⭐
- **What**: Up/down arrow navigation through command history
- **Why**: Essential TUI behavior that makes power users happy
- **Implementation**: Store command history vector, handle arrow key events

### Visual Message Categories ⭐
- **What**: Color-code info/warning/error messages differently
- **Why**: Instant visual parsing of what needs attention
- **Example**: 
  - Info: dim gray
  - Success: green ✅
  - Warning: yellow ⚠️
  - Error: red ❌

### Clear Command
- **What**: `/clear` to wipe status messages
- **Why**: Quick decluttering when messages pile up
- **Note**: Simple but satisfying

### Group Info Command
- **What**: `/info` shows current group details
- **Why**: Users get lost about which group they're in

### Better Command Hints
- **What**: Vim-style status bar: "Press / for commands, ? for help"
- **Why**: Teaches without being intrusive
- **Detail**: Context-sensitive - shows different hints based on current state

## Priority 2: Navigation & Layout (3-4 days)

### Sidebar Navigation ⭐
- **What**: Toggle-able sidebar for groups/contacts (Ctrl+B or similar)
- **Why**: Current modal overlay breaks flow
- **Implementation**: 
  - Shows active conversations
  - Indicates unread messages
  - Quick switch with arrow keys or numbers

### Selection UI Improvement
- **What**: Move group/contact selection into main chat area or slide-in sidebar
- **Why**: Modal overlays are jarring
- **Example**: When joining group, show options inline where messages appear

### Visual Zones (Light Touch)
- **What**: Ensure consistent layout zones
- **Current State**: Already working well
- **Maintain**:
  - Top: Group name, connection status
  - Middle: Messages only (no status messages mixed in)
  - Bottom: Input line
  - Status messages: Separate area that doesn't interfere

## Priority 3: Polish (1-2 days)

### Contact Names (If Not Already Done)
- **What**: Show contact names instead of truncated pubkeys
- **Note**: Verify if this already works

### Color Coding Participants
- **What**: Assign consistent colors to different message senders
- **Why**: Quick visual differentiation in group chats
- **Implementation**: Hash pubkey to color palette

### Status Message Auto-Scroll
- **What**: Let status messages naturally scroll out of view
- **Why**: No need for complex collapsing - just let new activity push them away

## What We're NOT Doing (For Now)
- Relative timestamps (power users like precision)
- Hiding technical details (it's for power users)
- Tab completion (nice to have, not essential)
- Circular buffer for status messages (let them scroll)
- Search functionality (big feature, needs own project)
- Rich text, pagination, typing indicators (scope creep)

## Implementation Order
1. Start with command history - it's fun and immediately useful
2. Add visual message categories - high impact, low effort
3. Implement sidebar navigation - biggest UX improvement
4. Polish with remaining items

## Success Metrics
- Command history usage (should be immediate adoption)
- Reduced `/help` command usage (better discoverability)
- Faster conversation switching
- Cleaner message area (less clutter)

## Technical Notes
- Keep changes minimal and surgical
- Don't break what's working
- Test with real multi-party conversations
- Maintain ephemeral storage model