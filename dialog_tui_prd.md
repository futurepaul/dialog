# Dialog TUI - Product Requirements Document

## Executive Summary

Dialog TUI is a terminal user interface application for secure, end-to-end encrypted messaging using the Nostr-MLS protocol. The UI is inspired by Claude Code's clean, command-driven interface, providing a modern and intuitive experience for encrypted group communications.

## Vision

Create a best-in-class terminal messaging application that combines the security of MLS encryption with the decentralized nature of Nostr, all wrapped in an elegant, keyboard-driven interface that power users will love.

## Core Design Principles

1. **Command-First**: All actions initiated through "/" commands
2. **Single-Column Layout**: Clean, focused interface like Claude Code
3. **Keyboard-Driven**: Full functionality without mouse
4. **Real-Time Feedback**: Immediate UI responses to all actions
5. **Progressive Disclosure**: Show only what's needed, when it's needed

## Architecture Overview

### Technology Stack
- **Language**: Rust
- **TUI Framework**: Ratatui
- **Text Input**: tui-textarea crate (recommended for robust text input)
- **Async Runtime**: Tokio
- **State Management**: Modified Elm Architecture (TEA)
- **Backend**: Stubbed Nostr-MLS functionality (for MVP)

### Application Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Main Event Loop                    │
├─────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │   Terminal  │  │ Event Handler│  │  Renderer  │ │
│  │   Backend   │  │   (Async)    │  │   (Pure)   │ │
│  └─────────────┘  └──────────────┘  └────────────┘ │
├─────────────────────────────────────────────────────┤
│                  State Management                    │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  App State  │  │    Update    │  │  Commands  │ │
│  │   (Model)   │  │  Function    │  │  (Effects) │ │
│  └─────────────┘  └──────────────┘  └────────────┘ │
├─────────────────────────────────────────────────────┤
│                     UI Components                    │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  Chat View  │  │ Command Input│  │ Status Bar │ │
│  │             │  │              │  │            │ │
│  └─────────────┘  └──────────────┘  └────────────┘ │
└─────────────────────────────────────────────────────┘
```

## UI Layout

### Main Screen Layout

```
┌─────────────────────────────────────────────────────────────┐
│ * Welcome to Dialog!                                        │
│                                                             │
│ /help for help, /status for your current setup             │
│                                                             │
│ cwd: /Users/username/dialog                                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ What's new:                                                 │
│ • Added /add command for adding contacts                    │
│ • Added /new command for starting conversations             │
│ • Added /keypackage command for key management              │
│ • Added /invites command for pending invitations            │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ > /█                                                        │
├─────────────────────────────────────────────────────────────┤
│ Type '/' to start a command • No active conversation • 0 contacts • Connected │
└─────────────────────────────────────────────────────────────┘
```

### Active Conversation Layout

```
┌─────────────────────────────────────────────────────────────┐
│ Conversation with Alice                                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Alice: Hey! How's the new UI coming along?                 │
│                                                             │
│ You: It's looking great! The Claude Code style really      │
│      works well for this.                                   │
│                                                             │
│ Alice: Awesome! Can't wait to see it in action.            │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ > █                                                         │
├─────────────────────────────────────────────────────────────┤
│ Type message and press Enter to send • Alice (online) • Group: Development Team • Encrypted │
└─────────────────────────────────────────────────────────────┘
```

### Command Palette

```
┌─────────────────────────────────────────────────────────────┐
│ > /                                                         │
├─────────────────────────────────────────────────────────────┤
│ /add           Add a new contact by pubkey or nip-05       │
│ /new           Start a new conversation                     │
│ /keypackage    Publish your key package                     │
│ /invites       View pending invitations                     │
│ /contacts      List all contacts                            │
│ /conversations List active conversations                    │
│ /help          Show available commands                      │
│ /quit          Exit the application                         │
│ /clear         Clear current conversation                   │
│ /status        Show connection status                       │
└─────────────────────────────────────────────────────────────┘
```

### Sub-Command UI: Searchable List (/conversations)

```
┌─────────────────────────────────────────────────────────────┐
│ /conversations                                              │
├─────────────────────────────────────────────────────────────┤
│ Select Conversation                                         │
│ Search or navigate to switch conversations                  │
│                                                             │
│ Search: alice█                                              │
│                                                             │
│ ❯ 1. Alice                      Last msg: 2 minutes ago    │
│   2. Alice & Bob Group          Last msg: 1 hour ago       │
│   3. Dev Team (Alice, Bob, +3)  Last msg: Yesterday        │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ 3 conversations found • ↑↓ Navigate • Enter Select • Esc Cancel │
└─────────────────────────────────────────────────────────────┘
```

### Sub-Command UI: Simple Selection (/invites)

```
┌─────────────────────────────────────────────────────────────┐
│ /invites                                                    │
├─────────────────────────────────────────────────────────────┤
│ Pending Invitations                                         │
│ You have 2 pending group invitations                        │
│                                                             │
│ ❯ 1. Engineering Team     Invited by: Bob                  │
│   2. Weekend Hangout      Invited by: Charlie              │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Enter to view • A to accept • R to reject • Esc to cancel  │
└─────────────────────────────────────────────────────────────┘
```

### Sub-Command UI: Action Selection (After selecting an invite)

```
┌─────────────────────────────────────────────────────────────┐
│ /invites                                                    │
├─────────────────────────────────────────────────────────────┤
│ Engineering Team                                            │
│ Group invitation from Bob                                   │
│                                                             │
│   Members: Bob, Alice, David, Eve                           │
│   Created: 2 days ago                                       │
│   Description: Main engineering discussion group            │
│                                                             │
│ What would you like to do?                                 │
│                                                             │
│ ❯ 1. Accept invitation      Join this group                 │
│   2. Reject invitation      Decline and remove              │
│   3. View more details      See full member list            │
│                                                             │
│                                                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Enter to confirm • Esc to go back                          │
└─────────────────────────────────────────────────────────────┘
```

### Sub-Command UI: Contact Search (/new)

```
┌─────────────────────────────────────────────────────────────┐
│ /new                                                        │
├─────────────────────────────────────────────────────────────┤
│ Start New Conversation                                      │
│ Select contacts to start a conversation with                │
│                                                             │
│ Search: █                                                   │
│                                                             │
│ ❯ □ Alice                    alice@nostr.com               │
│   ☑ Bob                      npub1qqs...8xts              │
│   □ Charlie                  charlie.btc@nostr.id          │
│   □ David                    david@example.com             │
│   □ Eve                      npub1zxc...9asd              │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ 1 selected • Space to toggle • Enter to create • Esc Cancel│
└─────────────────────────────────────────────────────────────┘
```

### Sub-Command UI: Confirmation Dialog (/keypackage)

```
┌─────────────────────────────────────────────────────────────┐
│ /keypackage                                                 │
├─────────────────────────────────────────────────────────────┤
│ Publish Key Package                                         │
│ This will publish your MLS key package to Nostr relays     │
│                                                             │
│   Current Identity: npub1abc...def                          │
│   Last Published: Never                                     │
│   Relay: wss://relay.damus.io                               │
│                                                             │
│ Publishing a key package allows others to invite you to    │
│ encrypted group conversations.                              │
│                                                             │
│ Publish key package now?                                    │
│                                                             │
│ ❯ Yes, publish                                              │
│   No, cancel                                                │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Enter to confirm • Esc to cancel                           │
└─────────────────────────────────────────────────────────────┘
```

### Sub-Command UI: Contact List (/contacts)

```
┌─────────────────────────────────────────────────────────────┐
│ /contacts                                                   │
├─────────────────────────────────────────────────────────────┤
│ Your Contacts (5)                                           │
│ Search or select a contact                                  │
│                                                             │
│ Search: █                                                   │
│                                                             │
│ ❯ Alice            alice@nostr.com         Added: 2d ago   │
│   Bob              npub1qqs...8xts         Added: 1w ago   │
│   Charlie          charlie.btc@nostr.id    Added: 1w ago   │
│   David            david@example.com       Added: 2w ago   │
│   Eve              npub1zxc...9asd         Added: 1m ago   │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Enter to message • D to delete • E to edit • Esc to close  │
└─────────────────────────────────────────────────────────────┘
```

### Sub-Command UI: Help Topics (/help)

```
┌─────────────────────────────────────────────────────────────┐
│ /help                                                       │
├─────────────────────────────────────────────────────────────┤
│ Help Topics                                                 │
│ Select a topic or command for detailed help                 │
│                                                             │
│ ❯ Getting Started      First steps with Dialog              │
│   Commands            List of all available commands        │
│   Keyboard Shortcuts  Navigation and shortcuts              │
│   Security            Encryption and privacy info           │
│   Troubleshooting     Common issues and solutions           │
│                                                             │
│ Commands:                                                   │
│   /add               Add a new contact                      │
│   /new               Start a new conversation               │
│   /keypackage        Manage your key packages               │
│   /invites           View and manage invitations            │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Enter to view • Esc to close                               │
└─────────────────────────────────────────────────────────────┘
```

## State Management

### Application State Structure

```rust
pub struct AppState {
    // UI State
    mode: AppMode,
    input: InputState,
    command_palette: CommandPaletteState,
    
    // Data
    contacts: HashMap<ContactId, Contact>,
    conversations: HashMap<ConversationId, Conversation>,
    active_conversation: Option<ConversationId>,
    pending_invites: Vec<PendingInvite>,
    
    // Status
    connection_status: ConnectionStatus,
    user_profile: Option<UserProfile>,
    
    // UI Components
    scroll_state: ScrollState,
    search_state: SearchState,
}

pub enum AppMode {
    Normal,
    CommandInput,
    CommandPalette,
    ContactSearch,
    InviteReview,
    Error(String),
}

pub enum InputMode {
    Normal,
    Command(CommandInput),
    Message(MessageInput),
    Search(SearchInput),
}
```

### Message Types

```rust
pub enum Msg {
    // UI Events
    KeyPress(KeyEvent),
    Resize(u16, u16),
    
    // Command Events
    CommandSubmitted(String),
    CommandPaletteToggled,
    CommandSelected(Command),
    
    // Contact Management
    AddContactRequested(String),
    ContactAdded(Contact),
    ContactSearchUpdated(String),
    ContactSelected(ContactId),
    
    // Conversation Management
    NewConversationRequested,
    ConversationCreated(ConversationId),
    ConversationSelected(ConversationId),
    MessageSent(String),
    MessageReceived(Message),
    
    // Invitations
    InviteListRequested,
    InvitesReceived(Vec<PendingInvite>),
    InviteAccepted(InviteId),
    InviteRejected(InviteId),
    
    // System
    KeyPackagePublishRequested,
    KeyPackagePublished,
    ConnectionStatusChanged(ConnectionStatus),
    Error(String),
}
```

### Commands (Side Effects)

```rust
pub enum Cmd {
    None,
    Batch(Vec<Cmd>),
    
    // Network Operations (Stubbed)
    FetchContact(String),
    CreateConversation(Vec<ContactId>),
    SendMessage(ConversationId, String),
    PublishKeyPackage,
    FetchInvites,
    AcceptInvite(InviteId),
    RejectInvite(InviteId),
    
    // UI Operations
    ShowNotification(String, NotificationType),
    ScrollTo(ScrollTarget),
    FocusInput,
    
    // Storage Operations (Stubbed)
    SaveContact(Contact),
    SaveConversation(Conversation),
    SaveMessage(Message),
}
```

## Command System

### Command Structure

```rust
pub struct Command {
    name: String,
    description: String,
    aliases: Vec<String>,
    handler: CommandHandler,
    sub_command_mode: SubCommandMode,
}

pub enum CommandHandler {
    Simple(fn(&mut AppState) -> Cmd),
    WithArgs(fn(&mut AppState, Vec<String>) -> Cmd),
    Interactive(fn(&mut AppState) -> (AppMode, Cmd)),
}

pub enum SubCommandMode {
    None,                    // Direct execution
    SearchableList,          // Like /conversations, /contacts
    SimpleSelection,         // Like /invites, /help topics
    ConfirmAction,          // Like /keypackage, /quit
}
```

### Core Commands

| Command | Description | Arguments | Sub-Command Mode | Behavior |
|---------|-------------|-----------|-----------------|----------|
| `/add` | Add a contact | `<pubkey\|nip05>` | None | Validates input, fetches profile, adds to contacts |
| `/new` | Start conversation | None | SearchableList | Opens contact search UI for creating conversation |
| `/keypackage` | Publish key package | None | ConfirmAction | Shows confirmation before publishing |
| `/invites` | View invitations | None | SimpleSelection | Shows list with accept/reject options |
| `/contacts` | List contacts | None | SearchableList | Shows searchable contact list |
| `/conversations` | List conversations | None | SearchableList | Shows searchable conversation list |
| `/help` | Show help | `[command]` | SimpleSelection | Shows command categories or specific help |
| `/clear` | Clear conversation | None | None | Clears current chat view |
| `/quit` | Exit application | None | ConfirmAction | Shows confirmation before exit |

## UI Components

### 1. Input Component
```rust
pub struct InputComponent {
    content: String,
    cursor_position: usize,
    mode: InputMode,
    history: Vec<String>,
    history_index: Option<usize>,
}
```

### 2. Chat View Component
```rust
pub struct ChatViewComponent {
    messages: Vec<DisplayMessage>,
    scroll_offset: usize,
    selected_message: Option<usize>,
}
```

### 3. Command Palette Component
```rust
pub struct CommandPaletteComponent {
    commands: Vec<Command>,
    filtered_commands: Vec<usize>,
    selected_index: usize,
    search_query: String,
}
```

### 4. Contact Search Component
```rust
pub struct ContactSearchComponent {
    contacts: Vec<Contact>,
    filtered_contacts: Vec<usize>,
    selected_index: usize,
    search_query: String,
    multi_select: bool,
    selected_contacts: HashSet<ContactId>,
}
```

### 5. Status Bar Component
```rust
pub struct StatusBarComponent {
    left_text: String,
    center_text: String,
    right_text: String,
}
```

## Keyboard Navigation

### Global Shortcuts
- `Ctrl+C`: Quit application
- `Ctrl+L`: Clear screen
- `Esc`: Cancel current operation / Go back
- `/`: Open command input
- `Tab`: Next focus area
- `Shift+Tab`: Previous focus area

### Command Input Mode
- `Enter`: Submit command
- `Up/Down`: Navigate command history
- `Tab`: Autocomplete command
- `Esc`: Cancel command

### Chat View
- `Up/Down`: Scroll messages
- `Page Up/Down`: Scroll page
- `Home/End`: Jump to top/bottom
- `Enter`: Reply to selected message
- `Space`: Select/deselect message

### Search/Selection
- `Up/Down`: Navigate items
- `Enter`: Select item
- `Space`: Multi-select toggle
- `Esc`: Cancel selection

## Error Handling

### Error Display
```rust
pub enum ErrorDisplay {
    Toast(String, Duration),
    Modal(ErrorModal),
    Inline(String),
}

pub struct ErrorModal {
    title: String,
    message: String,
    actions: Vec<ErrorAction>,
}
```

### Error Categories
1. **Input Errors**: Invalid command syntax, invalid pubkey format
2. **Network Errors**: Connection failures, timeout (stubbed)
3. **Protocol Errors**: MLS errors, Nostr errors (stubbed)
4. **UI Errors**: Rendering failures, state inconsistencies

## Mock/Stub Interfaces

### Nostr-MLS Service (Stubbed)
```rust
pub trait NostrMlsService {
    async fn publish_key_package(&self) -> Result<(), Error>;
    async fn fetch_contact(&self, identifier: &str) -> Result<Contact, Error>;
    async fn create_group(&self, contacts: Vec<ContactId>) -> Result<ConversationId, Error>;
    async fn send_message(&self, group_id: &ConversationId, content: &str) -> Result<(), Error>;
    async fn fetch_invites(&self) -> Result<Vec<PendingInvite>, Error>;
    async fn accept_invite(&self, invite_id: &InviteId) -> Result<(), Error>;
}

pub struct MockNostrMlsService {
    // Generates fake data for UI testing
}
```

### Mock Data Generators
```rust
impl MockNostrMlsService {
    fn generate_contact(&self) -> Contact {
        Contact {
            id: ContactId::new(),
            pubkey: self.generate_pubkey(),
            name: self.generate_name(),
            nip05: Some(self.generate_nip05()),
            picture: None,
            about: Some(self.generate_about()),
            added_at: Utc::now(),
        }
    }
    
    fn generate_message(&self) -> Message {
        Message {
            id: MessageId::new(),
            sender: ContactId::new(),
            content: self.generate_content(),
            timestamp: Utc::now(),
            status: MessageStatus::Delivered,
        }
    }
}
```

## Color Scheme

### Claude Code Inspired Theme
```rust
pub struct Theme {
    // Background
    bg_primary: Color::Rgb(30, 31, 38),      // Main background
    bg_secondary: Color::Rgb(39, 40, 49),    // Input/status bar
    bg_highlight: Color::Rgb(48, 49, 59),    // Selected items
    
    // Foreground
    fg_primary: Color::Rgb(248, 248, 242),   // Main text
    fg_secondary: Color::Rgb(139, 143, 150), // Muted text
    fg_accent: Color::Rgb(139, 233, 253),    // Commands/highlights
    
    // Semantic
    success: Color::Rgb(80, 250, 123),       // Success messages
    error: Color::Rgb(255, 85, 85),          // Error messages
    warning: Color::Rgb(255, 184, 108),      // Warnings
    info: Color::Rgb(189, 147, 249),         // Info messages
    
    // Borders
    border: Color::Rgb(68, 71, 90),          // UI borders
    border_focused: Color::Rgb(139, 233, 253), // Focused borders
}
```

## Text Input Implementation Research

### Recommended Approach: tui-textarea
Based on research of modern Ratatui applications and best practices, the **tui-textarea** crate is the recommended solution for robust text input handling.

#### Key Benefits:
- **Robust Multi-line Support**: Handles complex text editing scenarios
- **Rich Key Bindings**: Extensive default key mappings (Ctrl+H, Ctrl+D, Ctrl+K, etc.)
- **Backend Agnostic**: Works with crossterm, termion, and termwiz
- **Vim-like Support**: Optional vim emulation for power users
- **Search Integration**: Built-in search functionality with regex support

#### Implementation Example from Oatmeal App:
The Oatmeal terminal chat application (similar single-column layout) successfully uses `TextArea` for text input with:

```rust
// Layout structure
let layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(vec![
        Constraint::Min(1),  // Top section (messages)
        Constraint::Max(textarea_len)  // Bottom section (input)
    ])
    .split(frame.size());

// Event handling
textarea.input(key_event);
```

#### Recommended Dependencies:
```toml
[dependencies]
ratatui = "0.26"
tui-textarea = "0.4"
crossterm = "0.27"  # Or matching version with ratatui
```

### Status Line Specification
The status line provides critical user context and should display:

#### Primary Status Indicators:
1. **Input Mode Context**: Clear indication of current input state
   - Command mode: `"Type '/' to start a command"`
   - Message mode: `"Type message and press Enter to send"`
   - Search mode: `"Type to search, Enter to select"`

2. **Connection State**: Real-time connection status
   - `"Connected"` (green)
   - `"Connecting..."` (yellow)
   - `"Disconnected"` (red)

3. **Conversation Context**: Current conversation info
   - `"No active conversation"` (when none selected)
   - `"Alice (online)"` (for direct messages)
   - `"Group: Development Team"` (for group chats)

4. **User State**: Additional context
   - Contact count: `"5 contacts"`
   - Pending invites: `"2 pending invites"`
   - Encryption status: `"Encrypted"`

#### Status Line Format:
```
[Input Context] • [Conversation Info] • [Contact Count] • [Connection Status]
```

### Scroll Handling Strategy: Native Terminal Scrolling

Following Claude Code's approach, Dialog TUI will use **native terminal scrolling** rather than alternate screen mode. This provides a superior user experience where conversation history persists in the terminal's scrollback buffer.

#### Implementation Approach:
```rust
// Initialize with inline viewport (fixed TUI height at bottom)
let terminal = ratatui::init_with_options(TerminalOptions {
    viewport: Viewport::Inline(3), // Input + status + border
});

// New messages inserted above TUI area
terminal.insert_before(1, |buf| {
    Paragraph::new("Alice: Hello there!")
        .render(buf.area, buf);
});
```

#### Key Benefits:
- **Persistent History**: Messages remain in terminal scrollback after app exit
- **Native Scrolling**: Users can scroll up to see conversation history
- **Familiar UX**: Works exactly like Claude Code's interface
- **No Flickering**: Recent ratatui versions resolved flickering issues

#### Layout Structure:
```
┌─ Terminal Scrollback (native scroll) ──────────────────────┐
│ Alice: Hey! How's the new UI coming along?                │
│                                                           │
│ You: It's looking great! The Claude Code style really    │
│      works well for this.                                │
│                                                           │
│ Alice: Awesome! Can't wait to see it in action.          │
│ ... (scrollable history) ...                             │
├─ Fixed TUI Area (inline viewport) ─────────────────────────┤
│ > █                                                       │
├───────────────────────────────────────────────────────────┤
│ Type message and press Enter • Alice (online) • Encrypted │
└───────────────────────────────────────────────────────────┘
```

### Alternative to Current Broken Implementation
Rather than fixing the current broken text input implementation, this PRD recommends:

1. **Start Fresh**: Use tui-textarea as the foundation
2. **Follow Proven Patterns**: Study the Oatmeal app's successful implementation
3. **Implement Native Scrolling**: Use ratatui's inline viewport for Claude Code-style scrolling
4. **Implement Status Line**: Add clear user state indication
5. **Test Thoroughly**: Ensure all key bindings work correctly

This approach leverages battle-tested libraries and proven UI patterns rather than attempting to fix custom implementations.

## Code Quality and Architecture Issues to Address

### Current Technical Debt
Based on the current implementation, several code quality issues need immediate attention:

#### 1. Rust Warnings and Unused Code
**Problem**: Multiple warnings about unused variants, fields, and methods create noise and indicate incomplete implementation.

**Solution Strategy**:
- Remove unused `ConnectionStatus` variants (`Connecting`, `Disconnected`) or implement connection state management
- Remove unused theme fields/methods or implement proper theming throughout the app
- Remove unused `participants` field from `Conversation` or implement participant display

#### 2. Array Access Anti-patterns
**Problem**: Using numeric array indexing (`responses[message.len() % responses.len()]`) is a code smell in Rust.

**Better Patterns**:
```rust
// Instead of: responses[message.len() % responses.len()]
// Use: 
use rand::seq::SliceRandom;
let response = responses.choose(&mut rng).unwrap_or(&"Default response");

// Or deterministic but safer:
let response = responses.get(message.len() % responses.len()).unwrap_or(&"Default");
```

#### 3. Async Message Display Blocking
**Problem**: User messages don't appear immediately because the UI waits for the async response simulation.

**Solution**: Decouple message display from response generation:
```rust
// Show user message immediately
self.add_message(&format!("You: {}", message));

// Spawn async response in background
let message_clone = message.to_string();
let conv_clone = conv.clone();
tokio::spawn(async move {
    tokio::time::sleep(Duration::from_millis(500)).await;
    // Send response via channel back to main thread
});
```

### Adopting Real Nostr-MLS Types

#### Available Types from dialog_cli Analysis:
Based on the dialog_cli implementation, we should adopt these real types in mock mode:

**Core Types**:
- `GroupId` - MLS group identifier (32 bytes)
- `PublicKey` / `Keys` - Nostr key management
- `NostrGroupConfigData` - Group configuration
- `Event` / `EventBuilder` - Nostr event handling

**Storage Types**:
- `NostrMlsMemoryStorage` - For development/testing
- `NostrMlsSqliteStorage` - For production

**Message Types**:
- `Kind::MlsKeyPackage` - User key packages
- `Kind::MlsGroupMessage` - Group messages
- `Kind::GiftWrap` - Encrypted invitations

#### Implementation Strategy:

1. **Add nostr-mls Dependencies to dialog_tui**:
```toml
[dependencies]
nostr = { workspace = true }
nostr-mls = { workspace = true }
nostr-mls-memory-storage = { workspace = true }
hex = "0.4.3"
rand = "0.8"
```

2. **Replace Custom Types with Real Types**:
```rust
// Replace custom Conversation struct with real types
#[derive(Debug, Clone)]
pub struct MockConversation {
    pub mls_group_id: GroupId,           // Real MLS group ID
    pub nostr_group_id: PublicKey,       // Real Nostr public key  
    pub config: NostrGroupConfigData,    // Real group config
    pub participants: Vec<PublicKey>,    // Real public keys
    pub last_message: Option<String>,
    pub unread_count: usize,
}

// Replace custom Contact with real types
#[derive(Debug, Clone)]
pub struct MockContact {
    pub public_key: PublicKey,           // Real Nostr public key
    pub display_name: Option<String>,
    pub online: bool,
}
```

3. **Use Real Key Generation**:
```rust
// Generate realistic keys for fake contacts
use nostr_sdk::prelude::*;

impl App {
    fn setup_mock_data(&mut self) {
        // Generate real keys for fake contacts
        let alice_keys = Keys::generate();
        let bob_keys = Keys::generate();
        
        self.contacts.push(MockContact {
            public_key: alice_keys.public_key(),
            display_name: Some("Alice".to_string()),
            online: true,
        });
        
        // Create real group IDs
        let group_id = GroupId::from_slice(&rand::random::<[u8; 32]>());
        // ... etc
    }
}
```

#### Benefits of Real Types:
- **Type Safety**: Compile-time validation of IDs and keys
- **Future Compatibility**: Easy transition to real MLS implementation
- **Developer Experience**: IntelliSense and documentation from real types
- **Testing Realism**: Mock data that matches production data structures

## Updated Implementation Plan

### Phase 1: Code Quality Fix (Immediate)
1. **Fix Rust Warnings**
   - Remove or implement unused enum variants
   - Remove or implement unused struct fields
   - Remove or implement unused theme methods

2. **Fix Array Access Patterns**
   - Replace numeric indexing with `Iterator::cycle()` or `rand::choose()`
   - Add proper error handling for empty collections

3. **Fix Async Message Display**
   - Decouple user message display from response generation
   - Use channels or shared state for async response updates
   - Ensure UI responsiveness

### Phase 2: Nostr-MLS Type Adoption (Next)
1. **Add Dependencies**
   - Add nostr-mls workspace dependencies to dialog_tui
   - Add utility crates (hex, rand)

2. **Replace Custom Types**
   - Replace `Conversation` with `MockConversation` using real `GroupId`
   - Replace `Contact` with `MockContact` using real `PublicKey`
   - Use real `NostrGroupConfigData` for group settings

3. **Update Mock Data Generation**
   - Generate real keys for fake contacts
   - Create real group IDs for conversations
   - Use proper hex encoding/decoding

### Phase 3: Enhanced Functionality (Future)
1. **Real Storage Integration**
   - Integrate `NostrMlsMemoryStorage` for session persistence
   - Add save/load functionality for conversations and contacts

2. **Message Encryption Simulation**
   - Simulate MLS message encryption/decryption flow
   - Add group member management

3. **Network Simulation**
   - Mock relay connections and event publishing
   - Simulate real-time message delivery

This phased approach ensures immediate code quality improvements while laying groundwork for realistic MLS integration.

## Implementation Phases

### Phase 1: Core UI Foundation (MVP)
1. Basic Ratatui application structure with inline viewport (3-line fixed TUI)
2. tui-textarea integration for robust text input
3. Native terminal scrolling using terminal.insert_before()
4. Command input and parsing using TextArea
5. Help/welcome screen
6. Status bar with context indicators
7. Basic theming

### Phase 2: Command System
1. Command palette implementation
2. Command history
3. Autocomplete
4. Error handling for commands
5. Mock responses

### Phase 3: Contact Management
1. `/add` command with validation
2. Contact list view
3. Contact search UI
4. Mock contact data

### Phase 4: Conversation UI
1. Chat view component
2. Message rendering
3. Scrolling and navigation
4. `/new` command flow
5. Mock conversations

### Phase 5: Advanced Features
1. `/invites` management
2. `/keypackage` stubbed flow
3. Notifications
4. Multi-select UI
5. Settings/configuration

### Phase 6: Polish
1. Animations/transitions
2. Performance optimization
3. Comprehensive keyboard shortcuts
4. Help system enhancement
5. Error recovery

## Success Metrics

1. **Responsiveness**: All UI updates < 16ms
2. **Command Speed**: Command execution < 100ms
3. **Navigation**: All features accessible via keyboard
4. **Error Rate**: < 1% UI crashes
5. **User Satisfaction**: Intuitive command discovery

## Future Considerations

1. **Real Nostr-MLS Integration**: Replace mocks with actual protocol
2. **Rich Media**: Support for images, files
3. **Themes**: User-customizable color schemes
4. **Plugins**: Extension system for custom commands
5. **Mobile Bridge**: Companion mobile app

## Conclusion

This PRD defines a modern, efficient TUI for secure messaging that prioritizes user experience while maintaining the power and flexibility expected from terminal applications. The phased approach allows for iterative development and early user feedback while building toward a fully-featured messaging client.