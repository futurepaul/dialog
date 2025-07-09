use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc;
use nostr_mls::prelude::*;

pub enum AppResult {
    Continue,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    MessageInput,
    CommandInput,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
}

impl ConnectionStatus {
    pub fn simulate_connection_change(&mut self) {
        *self = match self {
            ConnectionStatus::Connected => ConnectionStatus::Disconnected,
            ConnectionStatus::Connecting => ConnectionStatus::Connected,
            ConnectionStatus::Disconnected => ConnectionStatus::Connecting,
        };
    }
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub name: String,
    pub pubkey: PublicKey,
    pub online: bool,
}

#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: String,
    pub group_id: Option<GroupId>,
    pub name: String,
    pub participants: Vec<PublicKey>,
    pub last_message: Option<String>,
    pub unread_count: usize,
    pub is_group: bool,
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
        };

        // Add welcome messages
        app.add_message("* Welcome to Dialog!");
        app.add_message("");
        app.add_message("/help for help, /status for your current setup");
        app.add_message("");
        app.add_message(&format!("cwd: {}", std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".to_string())));
        app.add_message("");

        // Add fake data
        app.setup_fake_data();

        app
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
            KeyCode::PageUp => {
                self.scroll_up();
                return AppResult::Continue;
            }
            KeyCode::PageDown => {
                self.scroll_down();
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
            } else if !current_text.starts_with('/') && self.mode == AppMode::CommandInput {
                self.mode = AppMode::MessageInput;
                self.update_placeholder();
            } else if current_text.is_empty() && self.mode != AppMode::Normal {
                self.mode = AppMode::Normal;
                self.update_placeholder();
            }
        }

        AppResult::Continue
    }

    fn update_placeholder(&mut self) {
        match self.mode {
            AppMode::Normal => self.text_area.set_placeholder_text("Type '/' to start a command"),
            AppMode::CommandInput => self.text_area.set_placeholder_text("Enter command"),
            AppMode::MessageInput => self.text_area.set_placeholder_text("Type message and press Enter to send"),
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
                self.add_message("Navigation:");
                self.add_message("  PageUp/PageDown - Scroll through messages");
                self.add_message("  Ctrl+C - Exit");
                self.add_message("  Esc - Clear input");
                self.add_message("");
            }
            "/add" => {
                if parts.len() > 1 {
                    let contact = parts[1];
                    self.add_message(&format!("Adding contact: {}", contact));
                    self.contact_count += 1;
                } else {
                    self.add_message("Usage: /add <pubkey|nip05>");
                }
            }
            "/new" => {
                if parts.len() > 1 {
                    let contact_name = parts[1];
                    if let Some(contact) = self.contacts.iter().find(|c| c.name.to_lowercase() == contact_name.to_lowercase()) {
                        let conv_id = format!("conv-{}", contact.name.to_lowercase());
                        if !self.conversations.iter().any(|c| c.id == conv_id) {
                            self.conversations.push(Conversation {
                                id: conv_id.clone(),
                                group_id: Some(GroupId::from_slice(&hex::decode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").unwrap())),
                                name: contact.name.clone(),
                                participants: vec![contact.pubkey],
                                last_message: None,
                                unread_count: 0,
                                is_group: false,
                            });
                            self.add_message(&format!("Started new conversation with {}", contact.name));
                            self.active_conversation = Some(conv_id);
                        } else {
                            self.add_message(&format!("Conversation with {} already exists", contact.name));
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
                if self.conversations.is_empty() {
                    self.add_message("No active conversations");
                } else {
                    self.add_message("Active conversations:");
                    self.add_message("");
                    
                    // Clone the conversations to avoid borrowing issues
                    let conversations = self.conversations.clone();
                    let active_conv = self.active_conversation.clone();
                    
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
                                self.contacts.iter()
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
                if self.contacts.is_empty() {
                    self.add_message("You have no contacts");
                } else {
                    self.add_message(&format!("You have {} contacts:", self.contacts.len()));
                    self.add_message("");
                    
                    // Clone the contacts to avoid borrowing issues
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
                self.add_message(&format!("You have {} pending invitations", self.pending_invites));
            }
            "/keypackage" => {
                self.add_message("Publishing key package (not implemented)");
            }
            "/status" => {
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
                self.connection_status.simulate_connection_change();
                self.add_message(&format!("Connection status changed to: {:?}", self.connection_status));
            }
            "/switch" => {
                if parts.len() > 1 {
                    if let Ok(num) = parts[1].parse::<usize>() {
                        if num > 0 && num <= self.conversations.len() {
                            // Clone the conversation data to avoid borrowing issues
                            let conv = self.conversations[num - 1].clone();
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
            // Clone the conversation data to avoid borrowing issues
            if let Some(conv) = self.conversations.iter().find(|c| c.id == *active_id).cloned() {
                // Show user message immediately - this should never be blocked
                self.add_message(&format!("You: {}", message));
                
                // Generate the response and send it with delay - this won't block the UI
                let response = self.generate_fake_response(message, &conv);
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

    fn setup_fake_data(&mut self) {
        // Generate real keys for mock data
        let alice_key = Keys::generate().public_key();
        let bob_key = Keys::generate().public_key();
        let charlie_key = Keys::generate().public_key();
        let diana_key = Keys::generate().public_key();
        
        // Add fake contacts with real keys
        self.contacts.push(Contact {
            name: "Alice".to_string(),
            pubkey: alice_key,
            online: true,
        });
        self.contacts.push(Contact {
            name: "Bob".to_string(),
            pubkey: bob_key,
            online: false,
        });
        self.contacts.push(Contact {
            name: "Charlie".to_string(),
            pubkey: charlie_key,
            online: true,
        });
        self.contacts.push(Contact {
            name: "Diana".to_string(),
            pubkey: diana_key,
            online: true,
        });

        // Generate real group IDs for mock conversations
        let alice_group_id = GroupId::from_slice(&hex::decode("1111111111111111111111111111111111111111111111111111111111111111").unwrap());
        let dev_group_id = GroupId::from_slice(&hex::decode("2222222222222222222222222222222222222222222222222222222222222222").unwrap());
        let charlie_group_id = GroupId::from_slice(&hex::decode("3333333333333333333333333333333333333333333333333333333333333333").unwrap());

        // Add fake conversations with real group IDs
        self.conversations.push(Conversation {
            id: "conv-alice".to_string(),
            group_id: Some(alice_group_id),
            name: "Alice".to_string(),
            participants: vec![alice_key],
            last_message: Some("Hey! How's it going?".to_string()),
            unread_count: 2,
            is_group: false,
        });
        self.conversations.push(Conversation {
            id: "conv-group-dev".to_string(),
            group_id: Some(dev_group_id),
            name: "Development Team".to_string(),
            participants: vec![alice_key, bob_key, charlie_key],
            last_message: Some("Alice: Let's sync up tomorrow".to_string()),
            unread_count: 0,
            is_group: true,
        });
        self.conversations.push(Conversation {
            id: "conv-charlie".to_string(),
            group_id: Some(charlie_group_id),
            name: "Charlie".to_string(),
            participants: vec![charlie_key],
            last_message: Some("Thanks for the help earlier!".to_string()),
            unread_count: 1,
            is_group: false,
        });

        // Update counts
        self.contact_count = self.contacts.len();
        self.pending_invites = 2;
    }

    fn generate_fake_response(&self, message: &str, conv: &Conversation) -> String {
        // Use message content hash for deterministic but varied responses
        let mut hasher = DefaultHasher::new();
        message.hash(&mut hasher);
        let hash = hasher.finish() as usize;

        if conv.is_group {
            let responses = [
                "Alice: That's interesting!",
                "Bob: I agree with that.",
                "Charlie: Good point!",
                "Diana: Thanks for sharing!",
                "Alice: I hadn't thought of that.",
                "Bob: Let's discuss this more.",
                "Charlie: Makes sense to me.",
                "Diana: Can you explain more?",
            ];
            responses.get(hash % responses.len()).unwrap_or(&"Alice: Thanks!").to_string()
        } else {
            let responses = [
                "Sounds good!",
                "I see what you mean.",
                "That's interesting to hear.",
                "Thanks for letting me know!",
                "I'll think about that.",
                "Good to hear from you!",
                "Let me get back to you on that.",
                "That makes sense.",
            ];
            let response = responses.get(hash % responses.len()).unwrap_or(&"Thanks!");
            format!("{}: {}", conv.name, response)
        }
    }
}