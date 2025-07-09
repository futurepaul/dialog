use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;
use tokio::sync::mpsc;
use dialog_lib::{DialogLib, Contact, Conversation, ConnectionStatus, AppMode, AppResult, ToBech32, hex};
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
}

impl App {
    pub fn new() -> Self {
        let mut text_area = TextArea::default();
        text_area.set_cursor_line_style(ratatui::style::Style::default());
        text_area.set_placeholder_text("Type '/' to start a command");
        
        // Create async message channel for delayed responses
        let (delayed_tx, delayed_rx) = mpsc::unbounded_channel();

        let mut app = Self {
            mode: AppMode::Normal,
            text_area,
            connection_status: ConnectionStatus::Connected,
            active_conversation: None,
            contact_count: 0,
            pending_invites: 0,
            messages: Vec::new(),
            scroll_offset: 0,
            contacts: Vec::new(),
            conversations: Vec::new(),
            delayed_message_rx: Some(delayed_rx),
            delayed_message_tx: Some(delayed_tx),
            dialog_lib: DialogLib::new_mock(), // Will be replaced in new_async()
            
            // Initialize search fields
            search_suggestions: Vec::new(),
            selected_suggestion: 0,
            is_searching: false,
            search_query: String::new(),
            search_start_pos: 0,
        };

        // Add welcome messages
        app.add_message("* Welcome to Dialog!");
        app.add_message("");
        app.add_message("/help for help, /status for your current setup");
        app.add_message("");
        app.add_message(&format!("cwd: {}", std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".to_string())));
        app.add_message("");

        app
    }
    
    pub async fn new_async() -> Self {
        let mut text_area = TextArea::default();
        text_area.set_cursor_line_style(ratatui::style::Style::default());
        text_area.set_placeholder_text("Type '/' to start a command");
        
        // Create async message channel for delayed responses
        let (delayed_tx, delayed_rx) = mpsc::unbounded_channel();

        let dialog_lib = DialogLib::new_mock_with_data().await;

        let mut app = Self {
            mode: AppMode::Normal,
            text_area,
            connection_status: ConnectionStatus::Connected,
            active_conversation: None,
            contact_count: 0,
            pending_invites: 0,
            messages: Vec::new(),
            scroll_offset: 0,
            contacts: Vec::new(),
            conversations: Vec::new(),
            delayed_message_rx: Some(delayed_rx),
            delayed_message_tx: Some(delayed_tx),
            dialog_lib,
            
            // Initialize search fields
            search_suggestions: Vec::new(),
            selected_suggestion: 0,
            is_searching: false,
            search_query: String::new(),
            search_start_pos: 0,
        };

        // Add welcome messages
        app.add_message("* Welcome to Dialog!");
        app.add_message("");
        app.add_message("/help for help, /status for your current setup");
        app.add_message("");
        app.add_message(&format!("cwd: {}", std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".to_string())));
        app.add_message("");

        app
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
        for ch in new_text.chars() {
            self.text_area.insert_char(ch);
        }
        
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
                self.add_message("/add <pubkey|nip05> - Add a new contact");
                self.add_message("/new - Start a new conversation");
                self.add_message("/conversations - List active conversations");
                self.add_message("/switch <number> - Switch to a conversation");
                self.add_message("/contacts - List all contacts");
                self.add_message("/invites - View pending invitations");
                self.add_message("/keypackage - Publish your key package");
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
                    if let Err(e) = self.dialog_lib.add_contact(contact).await {
                        self.add_message(&format!("Error adding contact: {}", e));
                    } else {
                        self.add_message(&format!("Adding contact: {}", contact));
                        self.refresh_data().await;
                    }
                } else {
                    self.add_message("Usage: /add <pubkey|nip05>");
                }
            }
            "/new" => {
                if parts.len() > 1 {
                    let contact_name = parts[1];
                    if let Some(contact) = self.contacts.iter().find(|c| c.name.to_lowercase() == contact_name.to_lowercase()) {
                        match self.dialog_lib.create_conversation(&contact.name, vec![contact.pubkey]).await {
                            Ok(conv_id) => {
                                self.add_message(&format!("Started new conversation with {}", contact.name));
                                let _ = self.dialog_lib.switch_conversation(&conv_id).await;
                                self.refresh_data().await;
                            }
                            Err(e) => {
                                self.add_message(&format!("Error: {}", e));
                            }
                        }
                    } else {
                        self.add_message(&format!("Contact '{}' not found. Use /contacts to see available contacts.", contact_name));
                    }
                } else {
                    self.add_message("Usage: /new <contact_name>");
                    self.add_message("Example: /new Alice");
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
                self.refresh_data().await;
                self.add_message(&format!("You have {} pending invitations", self.pending_invites));
            }
            "/keypackage" => {
                self.add_message("Publishing key package (not implemented)");
            }
            "/status" => {
                self.refresh_data().await;
                self.add_message("Current setup:");
                self.add_message("");
                self.add_message(&format!("  Working directory: {}", std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".to_string())));
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
                self.add_message("");
            }
            "/connect" => {
                if let Ok(status) = self.dialog_lib.toggle_connection().await {
                    self.connection_status = status;
                    self.add_message(&format!("Connection status changed to: {:?}", self.connection_status));
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
                                
                                // Add some fake conversation history
                                self.add_message("");
                                self.add_message("--- Conversation History ---");
                                if conv.is_group {
                                    self.add_message("Alice: Hey everyone!");
                                    self.add_message("Bob: Hi Alice! How's everyone doing?");
                                    self.add_message("You: Good to see you all!");
                                } else {
                                    self.add_message(&format!("{}: Hey there!", conv.name));
                                    self.add_message("You: Hi! How are you?");
                                    self.add_message(&format!("{}: I'm doing well, thanks for asking!", conv.name));
                                }
                                self.add_message("--- End History ---");
                                self.add_message("");
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
                
                // Generate the response and send it with delay
                let response = self.generate_fake_response(message, &conv).await;
                self.send_delayed_message(response, 500);
            } else {
                self.add_message("Error: Active conversation not found.");
            }
        } else {
            self.add_message("No active conversation. Use /conversations to see available conversations or /new to start one.");
        }
    }

    pub fn add_message(&mut self, message: &str) {
        self.messages.push(message.to_string());
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

    fn send_delayed_message(&self, message: String, delay_ms: u64) {
        if let Some(ref tx) = self.delayed_message_tx {
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                let _ = tx_clone.send(message);
            });
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

    async fn generate_fake_response(&self, message: &str, conv: &Conversation) -> String {
        // For now, use the mock service to generate responses
        if let Some(mock_service) = self.dialog_lib.mock_service() {
            mock_service.generate_fake_response(message, conv).await
        } else {
            // Fallback to simple response
            format!("{}: Thanks for your message!", conv.name)
        }
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
        let mut app = App::new_async().await;
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
        let mut app = App::new_async().await;
        app.refresh_data().await;

        // Simulate typing "@al" to search for Alice
        app.detect_at_search("Hello @al");
        assert!(app.is_searching);
        
        if !app.contacts.is_empty() {
            // We should get some suggestions if we have contacts
            app.update_search_suggestions();
            // The exact results depend on the mock data, so we'll just check the mechanism works
        }
    }

    #[tokio::test]
    async fn test_suggestion_navigation() {
        let mut app = App::new_async().await;
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
        let mut app = App::new_async().await;
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