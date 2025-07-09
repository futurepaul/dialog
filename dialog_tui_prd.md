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
│ No active conversation • 0 contacts • Connected             │
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
│ Alice (online) • Group: Development Team • Encrypted        │
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

## Implementation Phases

### Phase 1: Core UI Foundation (MVP)
1. Basic Ratatui application structure
2. Command input and parsing
3. Help/welcome screen
4. Status bar
5. Basic theming

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