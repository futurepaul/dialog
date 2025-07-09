use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;

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
}

impl App {
    pub fn new() -> Self {
        let mut text_area = TextArea::default();
        text_area.set_cursor_line_style(ratatui::style::Style::default());
        text_area.set_placeholder_text("Type '/' to start a command");
        
        let mut app = Self {
            mode: AppMode::Normal,
            text_area,
            connection_status: ConnectionStatus::Connected,
            active_conversation: None,
            contact_count: 0,
            pending_invites: 0,
            messages: Vec::new(),
            scroll_offset: 0,
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
                self.add_message("/add <pubkey|nip05> - Add a new contact");
                self.add_message("/new - Start a new conversation");
                self.add_message("/conversations - List active conversations");
                self.add_message("/contacts - List all contacts");
                self.add_message("/invites - View pending invitations");
                self.add_message("/keypackage - Publish your key package");
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
                self.add_message("Starting new conversation (not implemented)");
            }
            "/conversations" => {
                self.add_message("No active conversations");
            }
            "/contacts" => {
                self.add_message(&format!("You have {} contacts", self.contact_count));
            }
            "/invites" => {
                self.add_message(&format!("You have {} pending invitations", self.pending_invites));
            }
            "/keypackage" => {
                self.add_message("Publishing key package (not implemented)");
            }
            _ => {
                self.add_message(&format!("Unknown command: {}", parts[0]));
            }
        }
    }

    async fn process_message(&mut self, message: &str) {
        if self.active_conversation.is_some() {
            self.add_message(&format!("You: {}", message));
        } else {
            self.add_message("No active conversation. Use /new to start one.");
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

    pub fn get_status_text(&self) -> String {
        let input_context = match self.mode {
            AppMode::Normal => "Type '/' to start a command",
            AppMode::CommandInput => "Enter command",
            AppMode::MessageInput => "Type message and press Enter to send",
        };

        let conversation_info = match &self.active_conversation {
            Some(name) => format!("Talking to {}", name),
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
}