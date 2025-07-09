use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;
use tokio::sync::mpsc;
use dialog_lib::{DialogLib, Contact, Conversation, ConnectionStatus, AppMode, AppResult, ToBech32, hex, GroupId, UiUpdate};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

#[derive(Debug, Clone)]
pub struct ContactSuggestion {
    pub contact: Contact,
    pub score: i64,
    pub display_text: String,
}

#[derive(Debug)]
pub struct App {
    pub mode: AppMode,
    pub text_area: TextArea<'static>,
    pub connection_status: ConnectionStatus,
    pub active_conversation: Option<String>,
    pub contact_count: usize,
    pub pending_invites: usize,
    pub messages: Vec<String>,
    pub scroll_offset: usize,
    pub contacts: Vec<Contact>,
    pub conversations: Vec<Conversation>,
    pub delayed_message_rx: Option<mpsc::UnboundedReceiver<String>>,
    pub delayed_message_tx: Option<mpsc::UnboundedSender<String>>,
    pub dialog_lib: DialogLib,
    
    // Search functionality
    pub search_suggestions: Vec<ContactSuggestion>,
    pub selected_suggestion: usize,
    pub is_searching: bool,
    pub search_query: String,
    pub search_start_pos: usize,
    
    // Real-time update receiver
    pub ui_update_rx: Option<mpsc::Receiver<UiUpdate>>,
}

impl App {
    pub async fn new_with_service(dialog_lib: DialogLib) -> Result<Self, Box<dyn std::error::Error>> {
        let mut text_area = TextArea::default();
        text_area.set_cursor_line_style(ratatui::style::Style::default());
        text_area.set_placeholder_text("Type '/' to start a command");
        
        // Create async message channel for delayed responses
        let (delayed_tx, delayed_rx) = mpsc::unbounded_channel();

        // Create channel for UI updates
        let (ui_update_tx, ui_update_rx) = mpsc::channel(100);

        // Get initial data from the real service
        let contacts = dialog_lib.get_contacts().await.unwrap_or_default();
        let conversations = dialog_lib.get_conversations().await.unwrap_or_default();
        let connection_status = dialog_lib.get_connection_status().await.unwrap_or(ConnectionStatus::Disconnected);
        let pending_invites = dialog_lib.get_pending_invites_count().await.unwrap_or(0);
        let active_conversation = dialog_lib.get_active_conversation().await.unwrap_or(None);

        // Don't auto-start subscription - let user connect manually

        let mut app = Self {
            mode: AppMode::Normal,
            text_area,
            connection_status,
            active_conversation,
            contact_count: contacts.len(),
            pending_invites,
            messages: Vec::new(),
            scroll_offset: 0,
            contacts,
            conversations,
            delayed_message_rx: Some(delayed_rx),
            delayed_message_tx: Some(delayed_tx),
            dialog_lib,
            
            // Initialize search fields
            search_suggestions: Vec::new(),
            selected_suggestion: 0,
            is_searching: false,
            search_query: String::new(),
            search_start_pos: 0,
            
            // Real-time updates
            ui_update_rx: Some(ui_update_rx),
        };

        // Add welcome messages
        app.add_message("* Welcome to Dialog!");
        app.add_message("");
        app.add_message("⚡ Use /connect to connect to the relay");
        app.add_message("/help for help, /status for your current setup");
        app.add_message("");
        app.add_message(&format!("cwd: {}", std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".to_string())));
        app.add_message("");

        // Show initial state info
        if app.contacts.is_empty() {
            app.add_message("No contacts yet. MLS mode active.");
        }
        if app.conversations.is_empty() {
            app.add_message("No conversations yet. Use CLI to create groups and invite this TUI.");
        }

        Ok(app)
    }

    pub async fn refresh_data(&mut self) {
        // Refresh contacts
        if let Ok(contacts) = self.dialog_lib.get_contacts().await {
            self.contacts = contacts;
            self.contact_count = self.contacts.len();
        }

        // Refresh conversations
        if let Ok(conversations) = self.dialog_lib.get_conversations().await {
            self.conversations = conversations;
        }

        // Refresh connection status
        if let Ok(status) = self.dialog_lib.get_connection_status().await {
            self.connection_status = status;
        }

        // Refresh active conversation
        if let Ok(active) = self.dialog_lib.get_active_conversation().await {
            self.active_conversation = active;
        }

        // Refresh pending invites
        if let Ok(invites) = self.dialog_lib.get_pending_invites_count().await {
            self.pending_invites = invites;
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> AppResult {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return AppResult::Exit;
            }
            KeyCode::Esc => {
                if self.mode != AppMode::Normal {
                    self.mode = AppMode::Normal;
                    self.text_area.delete_line_by_head();
                    self.text_area.delete_line_by_end();
                    self.update_placeholder();
                }
                return AppResult::Continue;
            }
            KeyCode::Enter => {
                // If we're in search mode, accept the selected suggestion
                if self.is_searching && !self.search_suggestions.is_empty() {
                    self.accept_suggestion();
                    return AppResult::Continue;
                }
                
                let input = self.text_area.lines().join("\n");
                if !input.trim().is_empty() {
                    self.process_input(&input).await;
                    self.text_area.delete_line_by_head();
                    self.text_area.delete_line_by_end();
                    self.mode = AppMode::Normal;
                    self.update_placeholder();
                }
                return AppResult::Continue;
            }
            KeyCode::Up => {
                if self.is_searching {
                    self.move_suggestion_up();
                    return AppResult::Continue;
                } else {
                    self.scroll_up();
                    return AppResult::Continue;
                }
            }
            KeyCode::Down => {
                if self.is_searching {
                    self.move_suggestion_down();
                    return AppResult::Continue;
                } else {
                    self.scroll_down();
                    return AppResult::Continue;
                }
            }
            KeyCode::PageUp => {
                if !self.is_searching {
                    self.scroll_up();
                }
                return AppResult::Continue;
            }
            KeyCode::PageDown => {
                if !self.is_searching {
                    self.scroll_down();
                }
                return AppResult::Continue;
            }
            KeyCode::Char('/') if self.mode == AppMode::Normal => {
                self.mode = AppMode::CommandInput;
                self.text_area.delete_line_by_head();
                self.text_area.delete_line_by_end();
                self.text_area.insert_char('/');
                self.update_placeholder();
                return AppResult::Continue;
            }
            _ => {}
        }

        // Handle text input
        if self.text_area.input(key) {
            // Check if we're switching modes based on input
            let current_text = self.text_area.lines().join("");
            if current_text.starts_with('/') && self.mode != AppMode::CommandInput {
                self.mode = AppMode::CommandInput;
                self.update_placeholder();
            } else if !current_text.starts_with('/') && !current_text.is_empty() && self.mode != AppMode::MessageInput {
                self.mode = AppMode::MessageInput;
                self.update_placeholder();
            } else if current_text.is_empty() && self.mode != AppMode::Normal {
                self.mode = AppMode::Normal;
                self.update_placeholder();
            }
            
            // Check for @ search in message input mode
            if self.mode == AppMode::MessageInput {
                self.detect_at_search(&current_text);
            }
        }

        AppResult::Continue
    }

    fn detect_at_search(&mut self, input: &str) {
        // Find the last @ in the current input
        if let Some(at_pos) = input.rfind('@') {
            // Make sure we can safely get the text after @
            if at_pos + 1 <= input.len() {
                let after_at = &input[at_pos + 1..];
                if !after_at.contains(' ') {
                    self.is_searching = true;
                    self.search_query = after_at.to_string();
                    self.search_start_pos = at_pos;
                    self.update_search_suggestions();
                    return;
                }
            }
        }
        
        // If no valid @ search found, disable searching
        if self.is_searching {
            self.is_searching = false;
            self.search_suggestions.clear();
            self.selected_suggestion = 0;
        }
    }

    fn update_search_suggestions(&mut self) {
        if !self.is_searching {
            return;
        }

        let matcher = SkimMatcherV2::default();
        let mut suggestions = Vec::new();

        for contact in &self.contacts {
            if let Some(score) = matcher.fuzzy_match(&contact.name, &self.search_query) {
                suggestions.push(ContactSuggestion {
                    contact: contact.clone(),
                    score,
                    display_text: format!("{} ({})", contact.name, 
                        contact.pubkey.to_bech32().unwrap_or_else(|_| contact.pubkey.to_hex()[..8].to_string())),
                });
            }
        }

        // Sort by score descending
        suggestions.sort_by(|a, b| b.score.cmp(&a.score));
        
        // Take top 5 suggestions
        suggestions.truncate(5);
        
        self.search_suggestions = suggestions;
        self.selected_suggestion = 0;
    }

    fn accept_suggestion(&mut self) {
        if !self.is_searching || self.search_suggestions.is_empty() {
            return;
        }

        let suggestion = &self.search_suggestions[self.selected_suggestion];
        let current_text = self.text_area.lines().join("");
        
        // Replace the @query with @contactname - with safe bounds checking
        if self.search_start_pos > current_text.len() {
            // Invalid state, just clear search
            self.is_searching = false;
            self.search_suggestions.clear();
            self.selected_suggestion = 0;
            return;
        }
        
        let before_at = &current_text[..self.search_start_pos];
        let after_query_start = self.search_start_pos + 1 + self.search_query.len();
        let after_query = if after_query_start <= current_text.len() {
            &current_text[after_query_start..]
        } else {
            ""
        };
        let new_text = format!("{}@{}{}", before_at, suggestion.contact.name, after_query);
        
        // Clear and set new text
        self.text_area.delete_line_by_head();
        self.text_area.delete_line_by_end();
        self.text_area.insert_str(&new_text);
        
        // Clear search state
        self.is_searching = false;
        self.search_suggestions.clear();
        self.selected_suggestion = 0;
    }

    fn move_suggestion_up(&mut self) {
        if !self.search_suggestions.is_empty() && self.selected_suggestion > 0 {
            self.selected_suggestion -= 1;
        }
    }

    fn move_suggestion_down(&mut self) {
        if !self.search_suggestions.is_empty() && self.selected_suggestion < self.search_suggestions.len() - 1 {
            self.selected_suggestion += 1;
        }
    }

    fn update_placeholder(&mut self) {
        match self.mode {
            AppMode::Normal => self.text_area.set_placeholder_text("Type '/' to start a command"),
            AppMode::CommandInput => self.text_area.set_placeholder_text("Enter command"),
            AppMode::MessageInput => {
                if self.is_searching {
                    self.text_area.set_placeholder_text("Type @ and contact name for suggestions");
                } else {
                    self.text_area.set_placeholder_text("Type message and press Enter to send");
                }
            }
        }
    }

    async fn process_input(&mut self, input: &str) {
        if input.starts_with('/') {
            self.process_command(input).await;
        } else {
            self.process_message(input).await;
        }
    }

    async fn process_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "/quit" | "/q" => {
                // Could add confirmation here
                // For now, just exit
            }
            "/help" | "/h" => {
                self.add_message("Available commands:");
                self.add_message("");
                self.add_message("/help - Show this help message");
                self.add_message("/quit - Exit the application");
                self.add_message("/status - Show current setup and stats");
                self.add_message("/connect - Toggle connection status");
                self.add_message("/pk - Show your public key");
                self.add_message("");
                self.add_message("Contacts & Groups:");
                self.add_message("/add <pubkey> - Add a new contact");
                self.add_message("/contacts - List all contacts");
                self.add_message("/keypackage - Publish your key package (required for receiving invites)");
                self.add_message("/create <name> <contact1> [contact2]... - Create a group");
                self.add_message("/invites - View pending group invitations");
                self.add_message("/accept <group_id> - Accept a group invitation");
                self.add_message("");
                self.add_message("Conversations:");
                self.add_message("/conversations - List active conversations");
                self.add_message("/switch <number> - Switch to a conversation");
                self.add_message("/fetch - Fetch and display messages in the active conversation");
                self.add_message("");
                self.add_message("Features:");
                self.add_message("  @ search - Type '@' followed by contact name for fuzzy search");
                self.add_message("  Use Up/Down arrows to navigate suggestions, Enter to select");
                self.add_message("");
                self.add_message("Navigation:");
                self.add_message("  PageUp/PageDown - Scroll through messages");
                self.add_message("  Up/Down arrows - Navigate @ search suggestions");
                self.add_message("  Ctrl+C - Exit");
                self.add_message("  Esc - Clear input");
                self.add_message("");
            }
            "/add" => {
                if parts.len() > 1 {
                    let contact = parts[1];
                    
                    // Check and warn user about profile loading if not connected
                    match self.dialog_lib.get_connection_status().await {
                        Ok(status) => {
                            if status != ConnectionStatus::Connected {
                                self.add_message("⚠️  Not connected to relay - profile loading disabled");
                                self.add_message("Contact names will show as truncated pubkeys");
                                self.add_message("Use /connect to enable profile loading");
                                self.add_message("");
                            }
                        }
                        _ => {}
                    }
                    
                    match self.dialog_lib.add_contact(contact).await {
                        Ok(()) => {
                            self.add_message(&format!("✅ Contact added: {}", contact));
                            if self.connection_status == ConnectionStatus::Connected {
                                self.add_message("Profile loading attempted from relay");
                            }
                            self.refresh_data().await;
                        }
                        Err(e) => {
                            self.add_message(&format!("❌ Error adding contact: {}", e));
                        }
                    }
                } else {
                    self.add_message("Usage: /add <pubkey|nip05>");
                }
            }
            "/new" => {
                self.add_message("Use /create to create a new group conversation");
            }
            "/create" => {
                if parts.len() > 2 {
                    let group_name = parts[1];
                    let contact_names = parts[2..].to_vec();
                    
                    // Check if we're connected first
                    if self.connection_status != ConnectionStatus::Connected {
                        self.add_message("❌ Cannot create group - not connected to relay");
                        self.add_message("Use /connect to establish a connection first");
                        return;
                    }
                    
                    // Collect public keys for all mentioned contacts
                    let mut participants = Vec::new();
                    let mut missing_contacts = Vec::new();
                    
                    for name in &contact_names {
                        if let Some(contact) = self.contacts.iter().find(|c| c.name.to_lowercase() == name.to_lowercase()) {
                            participants.push(contact.pubkey);
                        } else {
                            missing_contacts.push(name.as_ref());
                        }
                    }
                    
                    if !missing_contacts.is_empty() {
                        self.add_message(&format!("❌ Contacts not found: {}", missing_contacts.join(", ")));
                        self.add_message("Use /contacts to see available contacts.");
                        return;
                    }
                    
                    if participants.is_empty() {
                        self.add_message("❌ No valid participants specified");
                        return;
                    }
                    
                    self.add_message(&format!("Creating group '{}' with {} participant(s)...", group_name, participants.len()));
                    match self.dialog_lib.create_conversation(group_name, participants).await {
                        Ok(group_id) => {
                            self.add_message(&format!("✅ Group '{}' created successfully!", group_name));
                            self.add_message(&format!("Group ID: {}", group_id));
                            self.add_message("Invitations have been sent to all participants.");
                            self.refresh_data().await;
                        }
                        Err(e) => {
                            self.add_message(&format!("❌ Error creating group: {}", e));
                            if e.to_string().contains("key package") {
                                self.add_message("Make sure all participants have published their key packages.");
                                self.add_message("They can use /keypackage command to publish.");
                            }
                        }
                    }
                } else {
                    self.add_message("Usage: /create <group_name> <contact1> [contact2] [contact3] ...");
                    self.add_message("Example: /create \"Coffee Chat\" Alice Bob");
                    self.add_message("Note: All participants must have published key packages");
                }
            }
            "/conversations" => {
                self.refresh_data().await;
                if self.conversations.is_empty() {
                    self.add_message("No active conversations");
                } else {
                    self.add_message("Active conversations:");
                    self.add_message("");
                    
                    // Collect all conversation display info first to avoid borrowing issues
                    let conversations = self.conversations.clone();
                    let active_conv = self.active_conversation.clone();
                    let contacts = self.contacts.clone();
                    
                    for (i, conv) in conversations.iter().enumerate() {
                        let unread = if conv.unread_count > 0 {
                            format!(" ({} unread)", conv.unread_count)
                        } else {
                            String::new()
                        };
                        let active = if Some(conv.id.clone()) == active_conv {
                            " [ACTIVE]"
                        } else {
                            ""
                        };
                        let group_indicator = if conv.is_group { "[GROUP] " } else { "" };
                        self.add_message(&format!("  {}: {}{}{}{}", i + 1, group_indicator, conv.name, unread, active));
                        if conv.is_group && !conv.participants.is_empty() {
                            let participant_names: Vec<String> = conv.participants.iter().map(|pk| {
                                // Try to find the name for this public key, otherwise use a short hex representation
                                contacts.iter()
                                    .find(|c| c.pubkey == *pk)
                                    .map(|c| c.name.clone())
                                    .unwrap_or_else(|| format!("{}", pk.to_bech32().unwrap_or_else(|_| pk.to_hex()[..8].to_string())))
                            }).collect();
                            self.add_message(&format!("      Participants: {}", participant_names.join(", ")));
                        }
                        if let Some(ref last_msg) = conv.last_message {
                            self.add_message(&format!("      Last: {}", last_msg));
                        }
                    }
                    self.add_message("");
                    self.add_message("Use '/switch <number>' to switch to a conversation");
                }
            }
            "/contacts" => {
                self.refresh_data().await;
                if self.contacts.is_empty() {
                    self.add_message("You have no contacts");
                } else {
                    self.add_message(&format!("You have {} contacts:", self.contacts.len()));
                    self.add_message("");
                    
                    // Clone contacts to avoid borrowing issues
                    let contacts = self.contacts.clone();
                    for (i, contact) in contacts.iter().enumerate() {
                        let status = if contact.online { "online" } else { "offline" };
                        let pubkey_display = contact.pubkey.to_bech32().unwrap_or_else(|_| contact.pubkey.to_hex()[..16].to_string());
                        self.add_message(&format!("  {}: {} ({}) - {}", i + 1, contact.name, status, pubkey_display));
                    }
                    self.add_message("");
                }
            }
            "/invites" => {
                // Check if we're connected first
                if self.connection_status != ConnectionStatus::Connected {
                    self.add_message("❌ Cannot fetch invites - not connected to relay");
                    self.add_message("Use /connect to establish a connection first");
                    return;
                }
                
                self.add_message("Fetching pending invites...");
                match self.dialog_lib.list_pending_invites().await {
                    Ok(result) => {
                        // First show any processing errors
                        if !result.processing_errors.is_empty() {
                            self.add_message("Processing errors encountered:");
                            for error in &result.processing_errors {
                                self.add_message(&format!("  {}", error));
                            }
                            self.add_message("");
                        }
                        
                        // Then show the invites
                        if result.invites.is_empty() {
                            self.add_message("No pending invites found.");
                        } else {
                            self.add_message(&format!("You have {} pending invites:", result.invites.len()));
                            self.add_message("");
                            for (idx, invite) in result.invites.iter().enumerate() {
                                self.add_message(&format!("{}. {}", idx + 1, invite.group_name));
                                self.add_message(&format!("   Group ID: {}", hex::encode(invite.group_id.as_slice())));
                                self.add_message(&format!("   Members: {}", invite.member_count));
                                self.add_message("");
                            }
                            self.add_message("Use /accept <group_id> to join a group");
                        }
                        // Update the pending invites count
                        self.pending_invites = result.invites.len();
                    }
                    Err(e) => {
                        self.add_message(&format!("❌ Error fetching invites: {}", e));
                    }
                }
            }
            "/keypackage" => {
                // Check if we're connected first
                if self.connection_status != ConnectionStatus::Connected {
                    self.add_message("❌ Cannot publish key package - not connected to relay");
                    self.add_message("Use /connect to establish a connection first");
                    return;
                }
                
                self.add_message("Publishing key package to relay...");
                match self.dialog_lib.publish_key_packages().await {
                    Ok(()) => {
                        self.add_message("✅ Key package published successfully!");
                        self.add_message("Other users can now invite you to groups.");
                    }
                    Err(e) => {
                        self.add_message(&format!("❌ Error publishing key package: {}", e));
                    }
                }
            }
            "/pk" => {
                match self.dialog_lib.get_own_pubkey().await {
                    Ok(pubkey) => {
                        self.add_message("Your public key:");
                        self.add_message("");
                        match pubkey.to_bech32() {
                            Ok(bech32) => self.add_message(&format!("  Bech32: {}", bech32)),
                            Err(_) => self.add_message("  Bech32: (encoding error)"),
                        }
                        self.add_message(&format!("  Hex: {}", pubkey.to_hex()));
                        self.add_message("");
                    }
                    Err(e) => {
                        self.add_message(&format!("Error getting public key: {}", e));
                    }
                }
            }
            "/status" => {
                self.refresh_data().await;
                self.add_message("Current setup:");
                self.add_message("");
                self.add_message(&format!("  Working directory: {}", std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".to_string())));
                
                // Add relay URL
                match self.dialog_lib.get_relay_url().await {
                    Ok(relay_url) => self.add_message(&format!("  Relay URL: {}", relay_url)),
                    Err(_) => self.add_message("  Relay URL: (error)"),
                }
                
                self.add_message(&format!("  Connection status: {:?}", self.connection_status));
                self.add_message(&format!("  Active conversation: {}", self.active_conversation.as_ref().unwrap_or(&"None".to_string())));
                
                // Show active conversation group ID if available
                if let Some(ref active_id) = self.active_conversation {
                    if let Some(conv) = self.conversations.iter().find(|c| c.id == *active_id) {
                        if let Some(ref group_id) = conv.group_id {
                            self.add_message(&format!("  Active group ID: {}", hex::encode(group_id.as_slice())));
                        }
                    }
                }
                
                self.add_message(&format!("  Contacts: {}", self.contacts.len()));
                self.add_message(&format!("  Conversations: {}", self.conversations.len()));
                self.add_message(&format!("  Pending invites: {}", self.pending_invites));
                self.add_message(&format!("  Total messages: {}", self.messages.len()));
                
                // Add pubkey information
                match self.dialog_lib.get_own_pubkey().await {
                    Ok(pubkey) => {
                        match pubkey.to_bech32() {
                            Ok(bech32) => self.add_message(&format!("  Public key: {}", bech32)),
                            Err(_) => self.add_message(&format!("  Public key: {}", pubkey.to_hex()[..16].to_string())),
                        }
                    }
                    Err(_) => {
                        self.add_message("  Public key: (error)");
                    }
                }
                
                self.add_message("");
            }
            "/connect" => {
                match self.dialog_lib.toggle_connection().await {
                    Ok(status) => {
                        self.connection_status = status;
                        self.add_message(&format!("Connection status changed to: {:?}", self.connection_status));
                        
                        // If we just connected, start the subscription for real-time messages
                        if status == ConnectionStatus::Connected {
                            // Create new channel for UI updates
                            let (ui_update_tx, ui_update_rx) = mpsc::channel(100);
                            self.ui_update_rx = Some(ui_update_rx);
                            
                            // Start subscription
                            if let Err(e) = self.dialog_lib.subscribe_to_groups(ui_update_tx).await {
                                self.add_message(&format!("⚠️  Failed to start real-time message subscription: {}", e));
                            } else {
                                self.add_message("✅ Real-time message updates enabled");
                            }
                        }
                    }
                    Err(e) => {
                        self.add_message(&format!("Connection failed: {}", e));
                        // Update to show current status
                        if let Ok(status) = self.dialog_lib.get_connection_status().await {
                            self.connection_status = status;
                        }
                    }
                }
            }
            "/switch" => {
                if parts.len() > 1 {
                    if let Ok(num) = parts[1].parse::<usize>() {
                        if num > 0 && num <= self.conversations.len() {
                            let conv = self.conversations[num - 1].clone();
                            if let Ok(()) = self.dialog_lib.switch_conversation(&conv.id).await {
                                self.active_conversation = Some(conv.id.clone());
                                self.add_message(&format!("Switched to conversation: {}", conv.name));
                                self.add_message("");
                                self.add_message("Use /fetch to load messages from this conversation");
                            }
                        } else {
                            self.add_message(&format!("Invalid conversation number. Use 1-{}", self.conversations.len()));
                        }
                    } else {
                        self.add_message("Usage: /switch <conversation_number>");
                    }
                } else {
                    self.add_message("Usage: /switch <conversation_number>");
                }
            }
            "/accept" => {
                if parts.len() > 1 {
                    let group_id = parts[1];
                    
                    // Check if we're connected first
                    if self.connection_status != ConnectionStatus::Connected {
                        self.add_message("❌ Cannot accept invite - not connected to relay");
                        self.add_message("Use /connect to establish a connection first");
                        return;
                    }
                    
                    self.add_message(&format!("Accepting invite for group {}...", group_id));
                    match self.dialog_lib.accept_invite(group_id).await {
                        Ok(()) => {
                            self.add_message("✅ Successfully joined group!");
                            self.add_message("The group should now appear in your conversations.");
                            self.refresh_data().await;
                        }
                        Err(e) => {
                            self.add_message(&format!("❌ Error accepting invite: {}", e));
                        }
                    }
                } else {
                    self.add_message("Usage: /accept <group_id>");
                    self.add_message("Get the group ID from /invites command");
                }
            }
            "/dangerously_publish_profile" => {
                if parts.len() > 1 {
                    let name = parts[1..].join(" "); // Join all parts after the command as the name
                    
                    // Check if we're connected first
                    if self.connection_status != ConnectionStatus::Connected {
                        self.add_message("❌ Cannot publish profile - not connected to relay");
                        self.add_message("Use /connect to establish a connection first");
                        return;
                    }
                    
                    self.add_message("⚠️  WARNING: This will publish your profile to the relay, making it publicly visible!");
                    self.add_message(&format!("Publishing profile with name: '{}'", name));
                    
                    match self.dialog_lib.publish_simple_profile(&name).await {
                        Ok(()) => {
                            self.add_message("✅ Profile published successfully!");
                            self.add_message("Your name is now visible to other users when they add you as a contact.");
                        }
                        Err(e) => {
                            self.add_message(&format!("❌ Error publishing profile: {}", e));
                            self.add_message("This could be due to:");
                            self.add_message("  - Relay connection issues");
                            self.add_message("  - Network problems");
                            self.add_message("  - Relay not accepting events");
                        }
                    }
                } else {
                    self.add_message("Usage: /dangerously_publish_profile <your_name>");
                    self.add_message("Example: /dangerously_publish_profile Alice");
                    self.add_message("⚠️  WARNING: This makes your name publicly visible on the relay!");
                }
            }
            "/fetch" => {
                // Check if we're connected first
                if self.connection_status != ConnectionStatus::Connected {
                    self.add_message("❌ Cannot fetch messages - not connected to relay");
                    self.add_message("Use /connect to establish a connection first");
                    return;
                }
                
                if let Some(ref active_id) = self.active_conversation {
                    if let Some(conv) = self.conversations.iter().find(|c| c.id == *active_id).cloned() {
                        self.add_message("Fetching messages...");
                        
                        if let Ok(bytes) = hex::decode(&conv.id) {
                            let group_id = GroupId::from_slice(&bytes);
                            match self.dialog_lib.fetch_messages(&group_id).await {
                                Ok(result) => {
                                    // First show any processing errors
                                    if !result.processing_errors.is_empty() {
                                        self.add_message("Processing errors encountered:");
                                        for error in &result.processing_errors {
                                            self.add_message(&format!("  {}", error));
                                        }
                                        self.add_message("");
                                    }
                                    
                                    // Then show the messages
                                    if result.messages.is_empty() {
                                        self.add_message("No messages in this conversation yet.");
                                    } else {
                                        self.add_message(&format!("Fetched {} messages:", result.messages.len()));
                                        self.add_message("");
                                        
                                        for msg in result.messages {
                                            // Get sender name from contacts or use truncated pubkey
                                            let own_pubkey = self.dialog_lib.get_own_pubkey().await.ok();
                                            let sender_name = if own_pubkey.as_ref() == Some(&msg.sender) {
                                                "You".to_string()
                                            } else if let Some(contact) = self.contacts.iter().find(|c| c.pubkey == msg.sender) {
                                                contact.name.clone()
                                            } else {
                                                format!("{}...", &msg.sender.to_hex()[0..8])
                                            };
                                            
                                            self.add_message(&format!("{}: {}", sender_name, msg.content));
                                        }
                                        
                                        self.add_message("");
                                        self.add_message("--- End of messages ---");
                                    }
                                }
                                Err(e) => {
                                    self.add_message(&format!("❌ Error fetching messages: {}", e));
                                }
                            }
                        } else {
                            self.add_message("Error: Invalid conversation ID format");
                        }
                    } else {
                        self.add_message("Error: Active conversation not found.");
                    }
                } else {
                    self.add_message("No active conversation. Use /switch to select a conversation first.");
                }
            }
            _ => {
                self.add_message(&format!("Unknown command: {}", parts[0]));
            }
        }
    }

    async fn process_message(&mut self, message: &str) {
        if let Some(ref active_id) = self.active_conversation {
            if let Some(conv) = self.conversations.iter().find(|c| c.id == *active_id).cloned() {
                // Show user message immediately
                self.add_message(&format!("You: {}", message));
                
                // Send the message via the dialog library
                if let Ok(bytes) = hex::decode(&conv.id) {
                    let group_id = GroupId::from_slice(&bytes);
                    match self.dialog_lib.send_message(&group_id, message).await {
                        Ok(()) => {
                            self.add_message("Message sent successfully");
                        }
                        Err(e) => {
                            self.add_message(&format!("Error sending message: {}", e));
                        }
                    }
                } else {
                    self.add_message("Error: Invalid conversation ID format");
                }
            } else {
                self.add_message("Error: Active conversation not found.");
            }
        } else {
            self.add_message("No active conversation. Use /conversations to see available conversations or /new to start one.");
        }
    }

    pub fn add_message(&mut self, message: &str) {
        // Wrap long messages to fit in terminal (leaving some margin for UI elements)
        let max_width = 120; // Conservative width that should work on most terminals
        
        if message.len() <= max_width {
            self.messages.push(message.to_string());
        } else {
            // Split long messages into multiple lines
            let words: Vec<&str> = message.split_whitespace().collect();
            let mut current_line = String::new();
            let mut lines = Vec::new();
            
            for word in words {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + 1 + word.len() <= max_width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    lines.push(current_line);
                    current_line = word.to_string();
                }
            }
            
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            
            // Add continuation marker for wrapped lines
            for (i, line) in lines.into_iter().enumerate() {
                if i == 0 {
                    self.messages.push(line);
                } else {
                    self.messages.push(format!("  {}", line)); // Indent continuation lines
                }
            }
        }
        
        // Auto-scroll to bottom when new message is added
        if self.messages.len() > 0 {
            self.scroll_offset = self.messages.len().saturating_sub(1);
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.messages.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    pub fn check_delayed_messages(&mut self) -> bool {
        let mut messages = Vec::new();
        if let Some(ref mut rx) = self.delayed_message_rx {
            while let Ok(message) = rx.try_recv() {
                messages.push(message);
            }
        }
        // Add all the messages after we're done with the receiver
        let had_messages = !messages.is_empty();
        for message in messages {
            self.add_message(&message);
        }
        had_messages
    }

    pub async fn check_ui_updates(&mut self) -> bool {
        let mut had_updates = false;
        let mut updates = Vec::new();
        
        if let Some(ref mut rx) = self.ui_update_rx {
            while let Ok(update) = rx.try_recv() {
                updates.push(update);
                had_updates = true;
            }
        }
        
        // Process updates after we're done with the receiver
        for update in updates {
            match update {
                UiUpdate::NewMessage { group_id, message } => {
                    // Check if this message is for the active conversation
                    if let Some(ref active_id) = self.active_conversation {
                        if let Some(conv) = self.conversations.iter().find(|c| c.id == *active_id) {
                            if let Some(ref conv_group_id) = conv.group_id {
                                if conv_group_id == &group_id {
                                    // Get sender name
                                    let own_pubkey = self.dialog_lib.get_own_pubkey().await.ok();
                                    let sender_name = if own_pubkey.as_ref() == Some(&message.sender) {
                                        "You".to_string()
                                    } else if let Some(contact) = self.contacts.iter().find(|c| c.pubkey == message.sender) {
                                        contact.name.clone()
                                    } else {
                                        format!("{}...", &message.sender.to_hex()[0..8])
                                    };
                                    
                                    // Add the message to the display
                                    self.add_message(&format!("{}: {}", sender_name, message.content));
                                }
                            }
                        }
                    }
                }
                UiUpdate::NewInvite(_invite) => {
                    // Update pending invite count
                    if let Ok(count) = self.dialog_lib.get_pending_invites_count().await {
                        self.pending_invites = count;
                        self.add_message("📨 New group invitation received! Use /invites to view.");
                    }
                }
                UiUpdate::ConnectionStatus(status) => {
                    self.connection_status = status;
                }
                UiUpdate::GroupStateChange { .. } => {
                    // Could refresh conversations here if needed
                }
            }
        }
        
        had_updates
    }

    fn send_delayed_message(&self, message: String, delay_ms: u64) {
        if let Some(ref tx) = self.delayed_message_tx {
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                let _ = tx_clone.send(message);
            });
        }
    }

    pub fn handle_paste(&mut self, text: &str) {
        // Insert the pasted text all at once without character-by-character animation
        self.text_area.insert_str(text);
        
        // Check if we need to update the mode based on the new text
        let current_text = self.text_area.lines().join("");
        if current_text.starts_with('/') && self.mode != AppMode::CommandInput {
            self.mode = AppMode::CommandInput;
            self.update_placeholder();
        } else if !current_text.starts_with('/') && !current_text.is_empty() && self.mode != AppMode::MessageInput {
            self.mode = AppMode::MessageInput;
            self.update_placeholder();
        }
        
        // Check for @ search in message input mode
        if self.mode == AppMode::MessageInput {
            self.detect_at_search(&current_text);
        }
    }

    pub fn get_status_text(&self) -> String {
        let input_context = match self.mode {
            AppMode::Normal => "Type '/' to start a command",
            AppMode::CommandInput => "Enter command",
            AppMode::MessageInput => "Type message and press Enter to send",
        };

        let conversation_info = match &self.active_conversation {
            Some(active_id) => {
                if let Some(conv) = self.conversations.iter().find(|c| c.id == *active_id) {
                    if conv.is_group {
                        format!("Group: {}", conv.name)
                    } else {
                        format!("Talking to {}", conv.name)
                    }
                } else {
                    "Unknown conversation".to_string()
                }
            }
            None => "No active conversation".to_string(),
        };

        let contact_info = format!("{} contacts", self.contact_count);

        let pending_info = if self.pending_invites > 0 {
            format!("{} pending invites", self.pending_invites)
        } else {
            String::new()
        };

        let connection_info = match self.connection_status {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Connecting => "Connecting...",
            ConnectionStatus::Disconnected => "Disconnected",
        };

        let parts: Vec<&str> = vec![
            input_context,
            &conversation_info,
            &contact_info,
            &pending_info,
            connection_info,
        ].into_iter().filter(|s| !s.is_empty()).collect();

        parts.join(" • ")
    }


    // Public getters for the UI
    pub fn get_search_suggestions(&self) -> &[ContactSuggestion] {
        &self.search_suggestions
    }

    pub fn get_selected_suggestion(&self) -> usize {
        self.selected_suggestion
    }

    pub fn is_in_search_mode(&self) -> bool {
        self.is_searching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_at_search_detection() {
        let dialog_lib = DialogLib::new().await.expect("Failed to create DialogLib");
        let mut app = App::new_with_service(dialog_lib).await.expect("Failed to create App");
        app.refresh_data().await;

        // Test @ search detection
        app.detect_at_search("Hello @ali");
        assert!(app.is_searching);
        assert_eq!(app.search_query, "ali");

        // Test no @ should not trigger search
        app.detect_at_search("Hello world");
        assert!(!app.is_searching);

        // Test @ with space should not trigger search
        app.detect_at_search("Hello @ alice");
        assert!(!app.is_searching);

        // Test multiple @ should use the last one
        app.detect_at_search("@bob says hi to @ali");
        assert!(app.is_searching);
        assert_eq!(app.search_query, "ali");
    }

    #[tokio::test]
    async fn test_fuzzy_search_suggestions() {
        let dialog_lib = DialogLib::new().await.expect("Failed to create DialogLib");
        let mut app = App::new_with_service(dialog_lib).await.expect("Failed to create App");
        app.refresh_data().await;

        // Simulate typing "@al" to search for Alice
        app.detect_at_search("Hello @al");
        assert!(app.is_searching);
        
        if !app.contacts.is_empty() {
            // We should get some suggestions if we have contacts
            app.update_search_suggestions();
            // The exact results depend on the data, so we'll just check the mechanism works
        }
    }

    #[tokio::test]
    async fn test_suggestion_navigation() {
        let dialog_lib = DialogLib::new().await.expect("Failed to create DialogLib");
        let mut app = App::new_with_service(dialog_lib).await.expect("Failed to create App");
        app.refresh_data().await;

        // Set up search
        app.detect_at_search("@a");
        if !app.search_suggestions.is_empty() {
            assert_eq!(app.selected_suggestion, 0);

            // Test moving down
            app.move_suggestion_down();
            if app.search_suggestions.len() > 1 {
                assert_eq!(app.selected_suggestion, 1);
            } else {
                assert_eq!(app.selected_suggestion, 0); // Should stay at 0 if only one suggestion
            }

            // Test moving up
            app.move_suggestion_up();
            assert_eq!(app.selected_suggestion, 0);
        }
    }

    #[tokio::test]
    async fn test_edge_cases_no_panic() {
        let dialog_lib = DialogLib::new().await.expect("Failed to create DialogLib");
        let mut app = App::new_with_service(dialog_lib).await.expect("Failed to create App");
        app.refresh_data().await;

        // Test @ at end of string
        app.detect_at_search("@");
        // Should not panic

        // Test empty string
        app.detect_at_search("");
        // Should not panic

        // Test single character
        app.detect_at_search("a");
        // Should not panic

        // Test @ with immediate space
        app.detect_at_search("@ ");
        assert!(!app.is_searching);

        // Test successful search then accept suggestion
        app.detect_at_search("@al");
        if !app.search_suggestions.is_empty() {
            app.accept_suggestion();
            // Should not panic
        }
    }
}