use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;
use tokio::sync::mpsc;
use ratatui::widgets::ListState;
use dialog_lib::{DialogLib, Contact, Conversation, ConnectionStatus, AppMode, AppResult, ToBech32, hex, GroupId, UiUpdate, PendingInvite};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use chrono::{DateTime, Local};

/// Helper function to format current timestamp in IRC style
fn format_timestamp() -> String {
    let now: DateTime<Local> = Local::now();
    format!("[{}]", now.format("%H:%M"))
}


#[derive(Debug, Clone)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
    Normal,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub content: String,
    pub message_type: MessageType,
}

#[derive(Debug, Clone)]
pub struct ConversationSuggestion {
    pub conversation: Conversation,
    pub score: i64,
    pub display_text: String,
}

#[derive(Debug)]
pub enum SelectionMode {
    None,
    InviteSelection { 
        invites: Vec<PendingInvite>, 
        state: ListState,
    },
    ConversationSelection { 
        state: ListState,
    },
    ContactSelection { 
        group_name: String,
        selections: Vec<bool>,
        state: ListState,
    },
}

#[derive(Debug)]
pub struct App {
    pub mode: AppMode,
    pub text_area: TextArea<'static>,
    pub connection_status: ConnectionStatus,
    pub active_conversation: Option<String>,
    pub contact_count: usize,
    pub pending_invites: usize,
    pub pending_invites_list: Vec<PendingInvite>,
    pub messages: Vec<StatusMessage>,
    pub scroll_offset: usize,
    pub contacts: Vec<Contact>,
    pub conversations: Vec<Conversation>,
    pub dialog_lib: DialogLib,
    
    // Search functionality
    pub conversation_suggestions: Vec<ConversationSuggestion>,
    pub selected_suggestion: usize,
    pub is_searching: bool,
    pub is_chat_switching: bool, // True when @ is used for chat switching
    pub search_query: String,
    pub search_start_pos: usize,
    
    // Real-time update receiver
    pub ui_update_rx: Option<mpsc::Receiver<UiUpdate>>,
    
    // Selection mode for interactive commands
    pub selection_mode: SelectionMode,
    
    // Command history
    pub command_history: Vec<String>,
    pub history_index: Option<usize>,
    
    // Sidebar state
    pub show_sidebar: bool,
    pub sidebar_selection: usize,
}

impl App {
    pub async fn new_with_service(dialog_lib: DialogLib) -> Result<Self, Box<dyn std::error::Error>> {
        let mut text_area = TextArea::default();
        text_area.set_cursor_line_style(ratatui::style::Style::default());
        text_area.set_placeholder_text("Type '/' to start a command");
        

        // Create channel for UI updates
        let (_ui_update_tx, ui_update_rx) = mpsc::channel(100);

        // Get initial data from the real service
        let contacts = dialog_lib.get_contacts().await.unwrap_or_default();
        let conversations = dialog_lib.get_conversations().await.unwrap_or_default();
        let connection_status = dialog_lib.get_connection_status().await.unwrap_or(ConnectionStatus::Disconnected);
        let pending_invites = dialog_lib.get_pending_invites_count().await.unwrap_or(0);
        let pending_invites_list = Vec::new(); // Will be populated when needed
        let active_conversation = dialog_lib.get_active_conversation().await.unwrap_or(None);

        // Don't auto-start subscription - let user connect manually

        let mut app = Self {
            mode: AppMode::Normal,
            text_area,
            connection_status,
            active_conversation,
            contact_count: contacts.len(),
            pending_invites,
            pending_invites_list,
            messages: Vec::new(),
            scroll_offset: 0,
            contacts,
            conversations,
            dialog_lib,
            
            // Initialize search fields
            conversation_suggestions: Vec::new(),
            selected_suggestion: 0,
            is_searching: false,
            is_chat_switching: false,
            search_query: String::new(),
            search_start_pos: 0,
            
            // Real-time updates
            ui_update_rx: Some(ui_update_rx),
            
            // Interactive selection mode
            selection_mode: SelectionMode::None,
            
            // Command history
            command_history: Vec::new(),
            history_index: None,
            
            // Sidebar state
            show_sidebar: false,
            sidebar_selection: 0,
        };

        // Add welcome messages
        app.add_message("* Welcome to Dialog!");
        app.add_message("");
        app.add_message("/help for help, /status for your current setup");
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
        // Handle selection mode navigation first
        if !matches!(self.selection_mode, SelectionMode::None) {
            return self.handle_selection_key(key).await;
        }
        
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return AppResult::Exit;
            }
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.show_sidebar = !self.show_sidebar;
                if self.show_sidebar {
                    // Fetch pending invites when opening sidebar
                    if self.connection_status == ConnectionStatus::Connected {
                        if let Ok(result) = self.dialog_lib.list_pending_invites().await {
                            self.pending_invites_list = result.invites;
                            self.pending_invites = self.pending_invites_list.len();
                        }
                    }
                    
                    // Ensure sidebar selection is within bounds
                    let total_items = self.conversations.len() + self.contacts.len() + self.pending_invites_list.len();
                    if self.sidebar_selection >= total_items && total_items > 0 {
                        self.sidebar_selection = 0;
                    }
                }
                return AppResult::Continue;
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
                // If sidebar is open, select the current item
                if self.show_sidebar {
                    self.sidebar_select().await;
                    return AppResult::Continue;
                }
                
                // If we're in search mode, accept the selected suggestion
                if self.is_searching && self.is_chat_switching && !self.conversation_suggestions.is_empty() {
                    if let Some(conversation_id) = self.accept_suggestion() {
                        // Handle conversation switch asynchronously
                        let _ = self.dialog_lib.switch_conversation(&conversation_id).await;
                    }
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
                if self.show_sidebar {
                    self.sidebar_up();
                    return AppResult::Continue;
                } else if self.is_searching {
                    self.move_suggestion_up();
                    return AppResult::Continue;
                } else if self.mode == AppMode::CommandInput || self.mode == AppMode::MessageInput {
                    // Navigate command history
                    self.navigate_history_up();
                    return AppResult::Continue;
                } else {
                    self.scroll_up();
                    return AppResult::Continue;
                }
            }
            KeyCode::Down => {
                if self.show_sidebar {
                    self.sidebar_down();
                    return AppResult::Continue;
                } else if self.is_searching {
                    self.move_suggestion_down();
                    return AppResult::Continue;
                } else if self.mode == AppMode::CommandInput || self.mode == AppMode::MessageInput {
                    // Navigate command history
                    self.navigate_history_down();
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
            KeyCode::Char('?') if self.mode == AppMode::Normal => {
                // Quick help shortcut
                self.process_command("/help").await;
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
                    self.is_chat_switching = true; // Enable chat switching mode
                    self.search_query = after_at.to_string();
                    self.search_start_pos = at_pos;
                    self.update_conversation_suggestions(); // Use conversation suggestions instead
                    return;
                }
            }
        }
        
        // If no valid @ search found, disable searching
        if self.is_searching {
            self.is_searching = false;
            self.is_chat_switching = false;
            self.conversation_suggestions.clear();
            self.selected_suggestion = 0;
        }
    }


    fn update_conversation_suggestions(&mut self) {
        if !self.is_searching || !self.is_chat_switching {
            return;
        }

        let matcher = SkimMatcherV2::default();
        let mut suggestions = Vec::new();

        for conversation in &self.conversations {
            // Match against conversation name
            let score = if self.search_query.is_empty() {
                1000 // Show all conversations when no query
            } else {
                matcher.fuzzy_match(&conversation.name, &self.search_query).unwrap_or(0)
            };

            if score > 0 {
                suggestions.push(ConversationSuggestion {
                    conversation: conversation.clone(),
                    score,
                    display_text: conversation.name.clone(),
                });
            }
        }

        // Sort by score descending
        suggestions.sort_by(|a, b| b.score.cmp(&a.score));
        
        // Take top 5 suggestions
        suggestions.truncate(5);
        
        self.conversation_suggestions = suggestions;
        self.selected_suggestion = 0;
    }

    fn accept_suggestion(&mut self) -> Option<String> {
        if !self.is_searching || !self.is_chat_switching {
            return None;
        }

        // Handle conversation switching
        if self.conversation_suggestions.is_empty() {
            return None;
        }

        let suggestion = &self.conversation_suggestions[self.selected_suggestion];
        let conversation_id = suggestion.conversation.id.clone();
        let conversation_name = suggestion.conversation.name.clone();
        
        // Update UI state immediately
        self.active_conversation = Some(conversation_id.clone());
        self.add_message(&format!("📍 Switching to: {}", conversation_name));
        
        // Clear the @ from input text
        self.text_area.delete_line_by_head();
        self.text_area.delete_line_by_end();
        
        let conversation_to_switch = Some(conversation_id);
        
        // Clear search state
        self.is_searching = false;
        self.is_chat_switching = false;
        self.conversation_suggestions.clear();
        self.selected_suggestion = 0;
        
        conversation_to_switch
    }

    fn move_suggestion_up(&mut self) {
        let suggestion_count = self.conversation_suggestions.len();
        
        if suggestion_count > 0 && self.selected_suggestion > 0 {
            self.selected_suggestion -= 1;
        }
    }

    fn move_suggestion_down(&mut self) {
        let suggestion_count = self.conversation_suggestions.len();
        
        if suggestion_count > 0 && self.selected_suggestion < suggestion_count - 1 {
            self.selected_suggestion += 1;
        }
    }

    fn update_placeholder(&mut self) {
        match self.mode {
            AppMode::Normal => self.text_area.set_placeholder_text("Type '/' to start a command"),
            AppMode::CommandInput => self.text_area.set_placeholder_text("Enter command"),
            AppMode::MessageInput => {
                if self.is_searching {
                    if self.is_chat_switching {
                        self.text_area.set_placeholder_text("Type @ and chat name to switch conversations");
                    } else {
                        self.text_area.set_placeholder_text("Type @ and contact name for suggestions");
                    }
                } else {
                    self.text_area.set_placeholder_text("Type message and press Enter to send, or @ to switch chats");
                }
            }
        }
    }
    
    async fn handle_selection_key(&mut self, key: KeyEvent) -> AppResult {
        match key.code {
            KeyCode::Esc => {
                self.selection_mode = SelectionMode::None;
                self.add_message("Selection cancelled");
                return AppResult::Continue;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match &mut self.selection_mode {
                    SelectionMode::InviteSelection { state, invites } => {
                        if !invites.is_empty() {
                            let i = match state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        invites.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            state.select(Some(i));
                        }
                    }
                    SelectionMode::ConversationSelection { state } => {
                        if !self.conversations.is_empty() {
                            let i = match state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        self.conversations.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            state.select(Some(i));
                        }
                    }
                    SelectionMode::ContactSelection { state, selections, .. } => {
                        if !selections.is_empty() {
                            let i = match state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        selections.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            state.select(Some(i));
                        }
                    }
                    _ => {}
                }
                return AppResult::Continue;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match &mut self.selection_mode {
                    SelectionMode::InviteSelection { state, invites } => {
                        if !invites.is_empty() {
                            let i = match state.selected() {
                                Some(i) => {
                                    if i >= invites.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            state.select(Some(i));
                        }
                    }
                    SelectionMode::ConversationSelection { state } => {
                        if !self.conversations.is_empty() {
                            let i = match state.selected() {
                                Some(i) => {
                                    if i >= self.conversations.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            state.select(Some(i));
                        }
                    }
                    SelectionMode::ContactSelection { state, selections, .. } => {
                        if !selections.is_empty() {
                            let i = match state.selected() {
                                Some(i) => {
                                    if i >= selections.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            state.select(Some(i));
                        }
                    }
                    _ => {}
                }
                return AppResult::Continue;
            }
            KeyCode::Char(' ') => {
                // Space toggles selection in ContactSelection mode
                if let SelectionMode::ContactSelection { state, selections, .. } = &mut self.selection_mode {
                    if let Some(i) = state.selected() {
                        if i < selections.len() {
                            selections[i] = !selections[i];
                        }
                    }
                }
                return AppResult::Continue;
            }
            KeyCode::Enter => {
                match &self.selection_mode {
                    SelectionMode::InviteSelection { state, invites } => {
                        if let Some(i) = state.selected() {
                            if i < invites.len() {
                                let invite = &invites[i];
                                let group_id = hex::encode(invite.group_id.as_slice());
                                self.selection_mode = SelectionMode::None;
                                
                                // Process the accept command
                                self.add_message(&format!("Accepting invite for group {}...", group_id));
                                match self.dialog_lib.accept_invite(&group_id).await {
                                    Ok(()) => {
                                        self.add_message_with_type("✅ Successfully joined group!", MessageType::Success);
                                        self.add_message("The group should now appear in your conversations.");
                                        self.refresh_data().await;
                                        
                                        // Auto-switch to the newly joined group
                                        if let Ok(()) = self.dialog_lib.switch_conversation(&group_id).await {
                                            self.active_conversation = Some(group_id.clone());
                                            if let Some(conv) = self.conversations.iter().find(|c| c.id == group_id) {
                                                self.add_message(&format!("📍 Auto-switched to group: {}", conv.name));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.add_message(&format!("❌ Error accepting invite: {}", e));
                                    }
                                }
                            }
                        }
                    }
                    SelectionMode::ConversationSelection { state } => {
                        if let Some(i) = state.selected() {
                            if i < self.conversations.len() {
                                let conv = self.conversations[i].clone();
                                self.selection_mode = SelectionMode::None;
                                
                                if let Ok(()) = self.dialog_lib.switch_conversation(&conv.id).await {
                                    self.active_conversation = Some(conv.id.clone());
                                    self.add_message(&format!("Switched to conversation: {}", conv.name));
                                    self.add_message("");
                                    self.add_message("Use /fetch to load messages from this conversation");
                                }
                            }
                        }
                    }
                    SelectionMode::ContactSelection { group_name, selections, .. } => {
                        // Collect selected contacts
                        let selected_contacts: Vec<_> = self.contacts.iter()
                            .zip(selections.iter())
                            .filter(|(_, selected)| **selected)
                            .map(|(contact, _)| contact.pubkey)
                            .collect();
                        
                        if selected_contacts.is_empty() {
                            self.add_message("❌ Please select at least one contact");
                            return AppResult::Continue;
                        }
                        
                        let group_name = group_name.clone();
                        self.selection_mode = SelectionMode::None;
                        
                        self.add_message(&format!("Creating group '{}' with {} participant(s)...", group_name, selected_contacts.len()));
                        
                        // Show which participants we're inviting (for observability)
                        self.add_message_with_type("📋 Fetching key packages for:", MessageType::Info);
                        for pubkey in &selected_contacts {
                            let name = self.contacts.iter()
                                .find(|c| c.pubkey == *pubkey)
                                .map(|c| c.name.as_str())
                                .unwrap_or("Unknown");
                            self.add_message(&format!("    - {} ({}...{})", 
                                name,
                                &pubkey.to_hex()[0..8],
                                &pubkey.to_hex()[pubkey.to_hex().len()-8..]
                            ));
                        }
                        
                        match self.dialog_lib.create_conversation(&group_name, selected_contacts).await {
                            Ok(group_id) => {
                                self.add_message_with_type(&format!("✅ Group '{}' created successfully!", group_name), MessageType::Success);
                                self.add_message(&format!("Group ID: {}", group_id));
                                self.add_message_with_type("✅ Welcome messages sent to all participants", MessageType::Success);
                                self.add_message("");
                                self.add_message("⚠️  EPHEMERAL MODE: Participants must accept invites during THIS session");
                                self.add_message("    (Their key packages are only valid until they restart)");
                                self.refresh_data().await;
                                
                                // Auto-switch to the newly created group
                                if let Ok(()) = self.dialog_lib.switch_conversation(&group_id).await {
                                    self.active_conversation = Some(group_id.clone());
                                    self.add_message(&format!("📍 Auto-switched to group: {}", group_name));
                                }
                            }
                            Err(e) => {
                                self.add_message(&format!("❌ Error creating group: {}", e));
                                if e.to_string().contains("key package") {
                                    self.add_message("");
                                    self.add_message("⚠️  EPHEMERAL MODE: This likely means:");
                                    self.add_message("    - Participant is offline (hasn't published packages this session)");
                                    self.add_message("    - They restarted and old packages are orphaned");
                                    self.add_message("    - They need to run /keypackage to publish fresh ones");
                                }
                            }
                        }
                    }
                    _ => {}
                }
                return AppResult::Continue;
            }
            _ => {}
        }
        
        AppResult::Continue
    }

    async fn process_input(&mut self, input: &str) {
        // Add to command history (but don't duplicate consecutive commands)
        if !input.trim().is_empty() {
            if self.command_history.is_empty() || self.command_history.last() != Some(&input.to_string()) {
                self.command_history.push(input.to_string());
                // Limit history size to prevent unbounded growth
                if self.command_history.len() > 100 {
                    self.command_history.remove(0);
                }
            }
        }
        // Reset history index when a new command is entered
        self.history_index = None;
        
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
            "/clear" => {
                self.messages.clear();
                self.scroll_offset = 0;
                self.add_message_with_type("Screen cleared", MessageType::Info);
            }
            "/help" | "/h" => {
                self.add_message("Available commands:");
                self.add_message("");
                self.add_message("/help - Show this help message");
                self.add_message("/quit - Exit the application");
                self.add_message("/clear - Clear all messages from the screen");
                self.add_message("/status - Show current setup and stats");
                self.add_message("/connect - Toggle connection status");
                self.add_message("/pk - Show your public key");
                self.add_message("");
                self.add_message("Contacts & Groups:");
                self.add_message("/add <pubkey> - Add a new contact");
                self.add_message("/contacts - List all contacts");
                self.add_message("/keypackage - Publish your key package (required for receiving invites)");
                self.add_message("/refresh-keys - Publish fresh key packages (replaces old ones)");
                self.add_message("/create <name> - Create a group (with interactive contact selection)");
                self.add_message("/invites - Open sidebar to view and accept pending invitations");
                self.add_message("");
                self.add_message("Conversations:");
                self.add_message("/switch - Switch to a conversation (interactive)");
                self.add_message("/info - Show details about the current conversation");
                self.add_message("/fetch - Fetch and display messages in the active conversation");
                self.add_message("");
                self.add_message("Features:");
                self.add_message("  @ search - Type '@' followed by contact name for fuzzy search");
                self.add_message("  Use Up/Down arrows to navigate suggestions, Enter to select");
                self.add_message("");
                self.add_message("Navigation:");
                self.add_message("  PageUp/PageDown - Scroll through messages");
                self.add_message("  Up/Down arrows - Navigate @ search suggestions or command history");
                self.add_message("  Ctrl+B - Toggle sidebar for conversations/contacts");
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
                            self.add_message_with_type(&format!("✅ Contact added: {}", contact), MessageType::Success);
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
                if parts.len() > 1 {
                    let group_name = parts[1..].join(" ");
                    
                    // Check if we're connected first
                    if self.connection_status != ConnectionStatus::Connected {
                        self.add_message("❌ Cannot create group - not connected to relay");
                        self.add_message("Use /connect to establish a connection first");
                        return;
                    }
                    
                    // Check if we have any contacts
                    if self.contacts.is_empty() {
                        self.add_message("❌ No contacts available. Add contacts first using /add <pubkey>");
                        return;
                    }
                    
                    // Enter contact selection mode
                    let selections = vec![false; self.contacts.len()];
                    let mut state = ListState::default();
                    state.select(Some(0));
                    self.selection_mode = SelectionMode::ContactSelection {
                        group_name,
                        selections,
                        state,
                    };
                    self.add_message("Select contacts for the group. Use arrow keys to navigate, Space to toggle, Enter to create, Esc to cancel.");
                    self.add_message("");
                    self.add_message_with_type("⚠️  EPHEMERAL MODE WARNING:", MessageType::Warning);
                    self.add_message("    Make sure selected contacts are ONLINE NOW");
                    self.add_message("    They must have published key packages THIS SESSION");
                    self.add_message("    (Invites to old/offline key packages will fail)");
                } else {
                    self.add_message("Usage: /create <group_name>");
                    self.add_message("Example: /create Coffee Chat");
                    self.add_message("Note: You'll be able to select participants interactively");
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
                    self.add_message_with_type("❌ Cannot fetch invites - not connected to relay", MessageType::Error);
                    self.add_message("Use /connect to establish a connection first");
                    return;
                }
                
                self.add_message_with_type("Fetching pending invites...", MessageType::Info);
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
                        
                        // Update the invites list
                        self.pending_invites_list = result.invites;
                        self.pending_invites = self.pending_invites_list.len();
                        
                        if self.pending_invites_list.is_empty() {
                            self.add_message("No pending invites found.");
                        } else {
                            // Open sidebar and select first invite
                            self.show_sidebar = true;
                            self.sidebar_selection = 0; // First item will be the first invite
                            self.add_message_with_type(&format!("You have {} pending invites. Sidebar opened - use ↑↓ to navigate, Enter to accept.", self.pending_invites), MessageType::Info);
                        }
                    }
                    Err(e) => {
                        self.add_message_with_type(&format!("❌ Error fetching invites: {}", e), MessageType::Error);
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
                
                self.add_message("Publishing key packages to relay...");
                match self.dialog_lib.publish_key_packages().await {
                    Ok(event_ids) => {
                        self.add_message_with_type(&format!("✅ Published {} key packages successfully!", event_ids.len()), MessageType::Success);
                        
                        // Show event IDs for observability
                        self.add_message("📋 Key package event IDs:");
                        for (i, event_id) in event_ids.iter().enumerate() {
                            self.add_message(&format!("    {}: {}...{}", 
                                i + 1, 
                                &event_id[0..8], 
                                &event_id[event_id.len()-8..]
                            ));
                        }
                        
                        self.add_message("");
                        self.add_message("Other users can now invite you to groups using these packages.");
                    }
                    Err(e) => {
                        self.add_message(&format!("❌ Error publishing key packages: {}", e));
                    }
                }
            }
            "/refresh-keys" => {
                // Check if we're connected first
                if self.connection_status != ConnectionStatus::Connected {
                    self.add_message("❌ Cannot refresh key packages - not connected to relay");
                    self.add_message("Use /connect to establish a connection first");
                    return;
                }
                
                self.add_message("Refreshing key packages...");
                self.add_message("⚠️  Note: This will publish new key packages. Old packages will remain valid.");
                
                // For now, we'll use the same publish_key_packages method
                // In the future, this could delete old packages first
                match self.dialog_lib.publish_key_packages().await {
                    Ok(event_ids) => {
                        self.add_message_with_type(&format!("✅ Published {} fresh key packages!", event_ids.len()), MessageType::Success);
                        
                        // Show event IDs for observability
                        self.add_message("📋 Fresh key package event IDs:");
                        for (i, event_id) in event_ids.iter().enumerate() {
                            self.add_message(&format!("    {}: {}...{}", 
                                i + 1, 
                                &event_id[0..8], 
                                &event_id[event_id.len()-8..]
                            ));
                        }
                        
                        self.add_message("");
                        self.add_message("💡 Tip: Groups created with old key packages may still fail.");
                        self.add_message("    Consider asking contacts to use your latest packages.");
                    }
                    Err(e) => {
                        self.add_message(&format!("❌ Error refreshing key packages: {}", e));
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
                
                // Add ephemeral mode warning
                self.add_message("🔐 EPHEMERAL MODE ACTIVE");
                self.add_message("  Storage: Memory (NostrMlsMemoryStorage)");
                self.add_message("  HPKE Keys: Lost on restart");
                self.add_message("  Key Packages: Fresh ones published each session");
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
                                self.add_message_with_type("✅ Real-time message updates enabled", MessageType::Success);
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
                if self.conversations.is_empty() {
                    self.add_message("No conversations available. Create or join a group first.");
                } else {
                    // Enter selection mode
                    let mut state = ListState::default();
                    state.select(Some(0));
                    self.selection_mode = SelectionMode::ConversationSelection { state };
                    self.add_message("Select a conversation. Use arrow keys to navigate, Enter to switch, Esc to cancel.");
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
                    
                    self.add_message_with_type("⚠️  WARNING: This will publish your profile to the relay, making it publicly visible!", MessageType::Warning);
                    self.add_message(&format!("Publishing profile with name: '{}'", name));
                    
                    match self.dialog_lib.publish_simple_profile(&name).await {
                        Ok(()) => {
                            self.add_message_with_type("✅ Profile published successfully!", MessageType::Success);
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
                    self.add_message_with_type("⚠️  WARNING: This makes your name publicly visible on the relay!", MessageType::Warning);
                }
            }
            "/info" => {
                if let Some(ref active_id) = self.active_conversation {
                    if let Some(conv) = self.conversations.iter().find(|c| c.id == *active_id).cloned() {
                        self.add_message_with_type("═══ Group Information ═══", MessageType::Info);
                        self.add_message(&format!("Name: {}", conv.name));
                        self.add_message(&format!("Group ID: {}", &conv.id[0..16]));
                        self.add_message(&format!("Type: {}", if conv.is_group { "Group Chat" } else { "Direct Message" }));
                        self.add_message(&format!("Participants: {} members", conv.participants.len()));
                        
                        // Show participant names if we have them in contacts
                        if !conv.participants.is_empty() {
                            self.add_message("");
                            self.add_message("Members:");
                            for participant in &conv.participants {
                                if let Some(contact) = self.contacts.iter().find(|c| &c.pubkey == participant) {
                                    self.add_message(&format!("  • {} ({}...)", contact.name, &participant.to_hex()[0..8]));
                                } else {
                                    self.add_message(&format!("  • {}...", &participant.to_hex()[0..8]));
                                }
                            }
                        }
                        
                        if let Some(ref last_msg) = conv.last_message {
                            self.add_message("");
                            self.add_message(&format!("Last message preview: {}", 
                                if last_msg.len() > 50 { 
                                    format!("{}...", &last_msg[0..50]) 
                                } else { 
                                    last_msg.clone() 
                                }
                            ));
                        }
                        self.add_message("═══════════════════════");
                    } else {
                        self.add_message_with_type("Error: Active conversation not found in list", MessageType::Error);
                    }
                } else {
                    self.add_message_with_type("No active conversation. Use /switch to select one.", MessageType::Warning);
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
                                            
                                            self.add_message(&format!("{} {}: {}", format_timestamp(), sender_name, msg.content));
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
                            self.add_message_with_type("Error: Invalid conversation ID format", MessageType::Error);
                        }
                    } else {
                        self.add_message_with_type("Error: Active conversation not found.", MessageType::Error);
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
                // Show user message immediately with timestamp
                self.add_message(&format!("{} You: {}", format_timestamp(), message));
                
                // Send the message via the dialog library
                if let Ok(bytes) = hex::decode(&conv.id) {
                    let group_id = GroupId::from_slice(&bytes);
                    match self.dialog_lib.send_message(&group_id, message).await {
                        Ok(()) => {
                            // Message sent successfully - no need to display confirmation
                        }
                        Err(e) => {
                            self.add_message(&format!("Error sending message: {}", e));
                        }
                    }
                } else {
                    self.add_message_with_type("Error: Invalid conversation ID format", MessageType::Error);
                }
            } else {
                self.add_message_with_type("Error: Active conversation not found.", MessageType::Error);
            }
        } else {
            self.add_message("No active conversation. Use /switch to see available conversations or /create to start one.");
        }
    }

    pub fn add_message(&mut self, message: &str) {
        self.add_message_with_type(message, MessageType::Normal);
    }
    
    pub fn add_message_with_type(&mut self, message: &str, message_type: MessageType) {
        // Wrap long messages to fit in terminal (leaving some margin for UI elements)
        let max_width = 120; // Conservative width that should work on most terminals
        
        if message.len() <= max_width {
            self.messages.push(StatusMessage {
                content: message.to_string(),
                message_type,
            });
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
                    self.messages.push(StatusMessage {
                        content: line,
                        message_type: message_type.clone(),
                    });
                } else {
                    self.messages.push(StatusMessage {
                        content: format!("  {}", line), // Indent continuation lines
                        message_type: message_type.clone(),
                    });
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
    
    pub fn navigate_history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }
        
        match self.history_index {
            None => {
                // Start from the most recent command
                self.history_index = Some(self.command_history.len() - 1);
                self.text_area.delete_line_by_head();
                self.text_area.delete_line_by_end();
                self.text_area.insert_str(&self.command_history[self.command_history.len() - 1]);
            }
            Some(idx) if idx > 0 => {
                // Go to older command
                self.history_index = Some(idx - 1);
                self.text_area.delete_line_by_head();
                self.text_area.delete_line_by_end();
                self.text_area.insert_str(&self.command_history[idx - 1]);
            }
            _ => {
                // Already at oldest command, do nothing
            }
        }
    }
    
    pub fn navigate_history_down(&mut self) {
        match self.history_index {
            Some(idx) if idx < self.command_history.len() - 1 => {
                // Go to newer command
                self.history_index = Some(idx + 1);
                self.text_area.delete_line_by_head();
                self.text_area.delete_line_by_end();
                self.text_area.insert_str(&self.command_history[idx + 1]);
            }
            Some(idx) if idx == self.command_history.len() - 1 => {
                // At the newest command, clear to show current input
                self.history_index = None;
                self.text_area.delete_line_by_head();
                self.text_area.delete_line_by_end();
            }
            _ => {
                // No history navigation active, do nothing
            }
        }
    }
    
    pub fn sidebar_up(&mut self) {
        let total_items = self.conversations.len() + self.contacts.len() + self.pending_invites_list.len();
        if total_items > 0 && self.sidebar_selection > 0 {
            self.sidebar_selection -= 1;
        }
    }
    
    pub fn sidebar_down(&mut self) {
        let total_items = self.conversations.len() + self.contacts.len() + self.pending_invites_list.len();
        if total_items > 0 && self.sidebar_selection < total_items - 1 {
            self.sidebar_selection += 1;
        }
    }
    
    pub async fn sidebar_select(&mut self) {
        let conv_count = self.conversations.len();
        let contact_count = self.contacts.len();
        
        if self.sidebar_selection < conv_count {
            // Selected a conversation
            if let Some(conv) = self.conversations.get(self.sidebar_selection) {
                let _ = self.dialog_lib.switch_conversation(&conv.id).await;
                self.active_conversation = Some(conv.id.clone());
                self.show_sidebar = false;
                self.add_message_with_type(&format!("Switched to: {}", conv.name), MessageType::Info);
            }
        } else if self.sidebar_selection < conv_count + contact_count {
            // Selected a contact - could implement DM functionality later
            let contact_idx = self.sidebar_selection - conv_count;
            if let Some(contact) = self.contacts.get(contact_idx) {
                self.add_message_with_type(&format!("Direct messages with {} not yet implemented", contact.name), MessageType::Warning);
            }
        } else {
            // Selected an invite
            let invite_idx = self.sidebar_selection - conv_count - contact_count;
            if let Some(invite) = self.pending_invites_list.get(invite_idx).cloned() {
                self.show_sidebar = false;
                self.add_message(&format!("Accepting invite for group {}...", invite.group_name));
                match self.dialog_lib.accept_invite(&hex::encode(invite.group_id.as_slice())).await {
                    Ok(()) => {
                        self.add_message_with_type("✅ Successfully joined group!", MessageType::Success);
                        self.add_message("The group should now appear in your conversations.");
                        self.refresh_data().await;
                    }
                    Err(e) => {
                        self.add_message_with_type(&format!("Error accepting invite: {}", e), MessageType::Error);
                    }
                }
            }
        }
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
                                    
                                    // Add the message to the display with timestamp
                                    self.add_message(&format!("{} {}: {}", format_timestamp(), sender_name, message.content));
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
        let input_context = if self.show_sidebar {
            "Sidebar • ↑↓ Navigate • Enter: Select • Ctrl+B: Close"
        } else {
            match (&self.mode, &self.selection_mode) {
                (_, SelectionMode::InviteSelection { .. }) => "↑↓ Navigate • Enter: Accept • Esc: Cancel",
                (_, SelectionMode::ConversationSelection { .. }) => "↑↓ Navigate • Enter: Switch • Esc: Cancel",
                (_, SelectionMode::ContactSelection { .. }) => "↑↓ Navigate • Space: Toggle • Enter: Create • Esc: Cancel",
                (AppMode::Normal, _) => "Press / for commands, ? for help",
                (AppMode::CommandInput, _) => "Command mode • ↑↓ History • Enter: Execute • Esc: Cancel",
                (AppMode::MessageInput, _) => {
                    if self.is_searching {
                        "@ search • ↑↓ Navigate • Enter: Select • Esc: Cancel"
                    } else {
                        "Message mode • Enter: Send • Esc: Cancel"
                    }
                },
            }
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

    pub fn get_conversation_suggestions(&self) -> &[ConversationSuggestion] {
        &self.conversation_suggestions
    }

    pub fn is_chat_switching(&self) -> bool {
        self.is_chat_switching
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
        if !app.conversation_suggestions.is_empty() {
            let _ = app.accept_suggestion();
            // Should not panic
        }
    }
}