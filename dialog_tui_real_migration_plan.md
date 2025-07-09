# Dialog TUI → RealMlsService Migration Plan

## Goal
Switch dialog_tui from MockMlsService to RealMlsService to enable real MLS messaging operations.

## Current State
- dialog_tui uses `DialogLib::new_mock_with_data()` 
- All data is fake/generated for UI testing
- No real MLS operations or message persistence

## Target State  
- dialog_tui uses `DialogLib::new_real()` for production mode
- Real MLS key generation, group management, message encryption
- Messages persist across sessions via memory storage
- TUI ready for cross-client messaging with dialog_cli

## Implementation Steps

### Step 1: Add Mode Selection to main.rs

**File**: `dialog_tui/src/main.rs`

```rust
use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("dialog_tui")
        .version("0.1.0")
        .about("Terminal UI for Dialog messaging")
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_name("MODE")
                .help("Run mode: 'mock' for testing, 'real' for production")
                .value_parser(["mock", "real"])
                .default_value("mock")
        )
        .get_matches();

    let mode = matches.get_one::<String>("mode").unwrap();
    
    let dialog_lib = match mode.as_str() {
        "real" => {
            tracing::info!("Starting TUI in REAL mode with actual MLS operations");
            dialog_lib::DialogLib::new_real().await?
        }
        "mock" | _ => {
            tracing::info!("Starting TUI in MOCK mode with fake data");
            dialog_lib::DialogLib::new_mock_with_data().await
        }
    };

    let mut app = App::new_with_service(dialog_lib).await?;
    // ... rest of main function
}
```

### Step 2: Update App Initialization

**File**: `dialog_tui/src/app.rs`

**Current Pattern**:
```rust
impl App {
    pub fn new() -> Self {
        let dialog_lib = DialogLib::new_mock();
        // ... initialization
    }
}
```

**New Pattern**:
```rust
impl App {
    // Keep existing new() for backward compatibility (mock mode)
    pub fn new() -> Self {
        // Use tokio::runtime for sync context
        let rt = tokio::runtime::Runtime::new().unwrap();
        let dialog_lib = rt.block_on(async {
            DialogLib::new_mock_with_data().await
        });
        Self::new_with_service(dialog_lib)
    }
    
    // New async constructor that accepts any DialogLib instance  
    pub async fn new_with_service(dialog_lib: DialogLib) -> Result<Self, Box<dyn std::error::Error>> {
        let mut text_area = TextArea::default();
        text_area.set_cursor_line_style(ratatui::style::Style::default());
        text_area.set_placeholder_text("Type '/' to start a command");
        
        let (delayed_tx, delayed_rx) = mpsc::unbounded_channel();

        // Get initial data from the service
        let contacts = dialog_lib.get_contacts().await.unwrap_or_default();
        let conversations = dialog_lib.get_conversations().await.unwrap_or_default();
        let connection_status = dialog_lib.get_connection_status().await.unwrap_or(ConnectionStatus::Disconnected);
        let pending_invites = dialog_lib.get_pending_invites_count().await.unwrap_or(0);

        Ok(Self {
            mode: AppMode::Normal,
            text_area,
            connection_status,
            active_conversation: None,
            contact_count: contacts.len(),
            pending_invites,
            messages: Vec::new(),
            scroll_offset: 0,
            contacts,
            conversations,
            delayed_message_rx: Some(delayed_rx),
            delayed_message_tx: Some(delayed_tx),
            dialog_lib,
            
            // Search functionality
            search_suggestions: Vec::new(),
            selected_suggestion: 0,
            is_searching: false,
            search_query: String::new(),
            search_start_pos: 0,
        })
    }
}
```

### Step 3: Update Cargo.toml Dependencies

**File**: `dialog_tui/Cargo.toml`

```toml
[dependencies]
# Add clap for command line argument parsing
clap = { version = "4.0", features = ["derive"] }

# Keep existing dependencies...
ratatui = "0.28"
tui-textarea = { version = "0.4", features = ["crossterm"] }
crossterm = "0.27"
tokio = { version = "1.40", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = "0.4"
uuid = { version = "1.10", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"
fake = { version = "2.9", features = ["derive"] }
fuzzy-matcher = "0.3"

# Dialog library (no direct nostr dependencies!)
dialog_lib = { version = "0.1.0", path = "../dialog_lib" }
```

### Step 4: Test Real Mode Operations

**Testing Commands**:
```bash
# Test mock mode (should work as before)
cd dialog_tui && cargo run

# Test real mode (new functionality)  
cd dialog_tui && cargo run -- --mode real

# Test that real mode starts without errors
# Test that UI remains responsive
# Test that conversations can be created (when available)
```

### Step 5: Handle Real Mode Differences

**Key Differences to Address**:

1. **Empty Initial State**: Real mode starts with no data
2. **Async Operations**: Real MLS operations take time
3. **Error Handling**: Network/MLS errors need graceful handling
4. **Key Management**: Real cryptographic keys are generated

**UI Updates Needed**:
```rust
// dialog_tui/src/app.rs - Handle empty real state
impl App {
    fn render_conversations(&self) -> Vec<String> {
        if self.conversations.is_empty() {
            vec![
                "No conversations yet.".to_string(),
                "In real mode, use CLI to create groups and invite this TUI.".to_string(),
                format!("Your public key: {}", self.get_public_key_display()),
            ]
        } else {
            // Render actual conversations
            self.conversations.iter()
                .map(|conv| format!("{}: {}", conv.name, conv.last_message.as_deref().unwrap_or("No messages")))
                .collect()
        }
    }
    
    fn get_public_key_display(&self) -> String {
        // TODO: Add method to DialogLib to get user's public key
        // For now, placeholder
        "Real public key (TODO: get from DialogLib)".to_string()
    }
}
```

### Step 6: Add Development Conveniences

**Environment Variable Support**:
```rust
// dialog_tui/src/main.rs - Check environment variable
let mode = matches.get_one::<String>("mode")
    .map(|s| s.as_str())
    .or_else(|| std::env::var("DIALOG_TUI_MODE").ok().as_deref())
    .unwrap_or("mock");
```

**Configuration File** (optional):
```toml
# dialog_tui/config.toml
mode = "real"  # or "mock"
relay_url = "ws://localhost:8080"
```

## Testing Plan

### Phase 1: Basic Functionality
1. ✅ TUI starts in real mode without crashing
2. ✅ UI remains responsive (no blocking on MLS operations)
3. ✅ Empty state displays helpful information
4. ✅ Mock mode still works (backward compatibility)

### Phase 2: Real Operations
1. 🔄 Real key generation works
2. 🔄 Real conversations can be displayed (when created externally)
3. 🔄 Real message sending works (when groups exist)
4. 🔄 Real connection status reflects actual relay connection

### Phase 3: Integration Testing
1. 🔄 CLI creates group → TUI shows invitation
2. 🔄 CLI sends message → TUI receives and displays message
3. 🔄 TUI sends message → CLI can retrieve with get-messages
4. 🔄 MLS state stays synchronized between CLI and TUI

## Potential Issues & Solutions

### Issue 1: Async Operations in Sync UI Context
**Problem**: TUI event loop is sync, MLS operations are async
**Solution**: Use tokio::task::spawn for background operations, update UI via channels

### Issue 2: Empty Initial State in Real Mode
**Problem**: Real mode starts with no conversations/contacts
**Solution**: Show helpful empty state with public key and instructions

### Issue 3: Error Handling for Network Operations  
**Problem**: Real MLS operations can fail (network, crypto errors)
**Solution**: Add error display in UI, graceful degradation

### Issue 4: Key Persistence
**Problem**: Real keys need to persist across sessions
**Solution**: dialog_lib memory storage handles this (keys stay consistent)

## Success Criteria

- ✅ `cargo run -- --mode real` starts successfully
- ✅ TUI displays real public key and connection status
- ✅ TUI remains responsive during async MLS operations  
- ✅ Ready for cross-client messaging tests with dialog_cli
- ✅ Mock mode still works for development

## Next Step After Completion

Once dialog_tui works in real mode, proceed to **Step 3B: Enable Cross-Client Messaging** to test the end-to-end messaging workflow between dialog_cli and dialog_tui.