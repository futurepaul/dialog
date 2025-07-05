use std::collections::HashMap;
use nostr_sdk::PublicKey;
use nostr_mls::prelude::GroupId;
use super::message::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ActivePane {
    Contacts,
    Conversations,
    Chat,
    Input,
    PowerTools,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
}

pub type ConversationId = String;
pub type ContactId = String;

#[derive(Debug, Clone)]
pub struct Contact {
    pub id: ContactId,
    pub pubkey: PublicKey,
    pub petname: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: ConversationId,
    pub group_id: Option<GroupId>,
    pub name: String,
    pub participants: Vec<PublicKey>,
    pub last_message_time: Option<chrono::DateTime<chrono::Utc>>,
    pub unread_count: u32,
}

#[derive(Debug, Clone)]
pub struct PendingInvite {
    pub id: String,
    pub from: PublicKey,
    pub petname: String, // Will be "Unknown" if not in contacts
    pub group_name: Option<String>,
    pub event_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogMode {
    Normal,
    AddContact,
    CreateConversation,
    PublishKeypackage,
    AcceptInvite,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PowerToolsMode {
    Menu,
    DebugLog,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DialogState {
    pub mode: DialogMode,
    pub input_buffer: String,
    pub field_index: usize, // For multi-field dialogs
    pub stored_fields: Vec<String>, // For storing previous field values
}

#[derive(Debug, Clone)]
pub struct AppState {
    // UI State
    pub active_pane: ActivePane,
    pub selected_contact: Option<ContactId>,
    pub selected_conversation: Option<ConversationId>,
    pub input_buffer: String,
    pub scroll_offset: u16,
    pub dialog_state: DialogState,
    
    // Data State
    pub contacts: HashMap<ContactId, Contact>,
    pub conversations: HashMap<ConversationId, Conversation>,
    pub messages: HashMap<ConversationId, Vec<ChatMessage>>,
    pub pending_invites: Vec<PendingInvite>,
    pub selected_invite: Option<usize>,
    
    // Network State
    pub connection_status: ConnectionStatus,
    
    // UI Layout State
    pub terminal_size: (u16, u16),
    pub show_help: bool,
    pub power_tools_mode: PowerToolsMode,
    pub debug_logs: Vec<LogEntry>,
    pub power_tools_selection: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_pane: ActivePane::Contacts,
            selected_contact: None,
            selected_conversation: None,
            input_buffer: String::new(),
            scroll_offset: 0,
            dialog_state: DialogState {
                mode: DialogMode::Normal,
                input_buffer: String::new(),
                field_index: 0,
                stored_fields: Vec::new(),
            },
            contacts: HashMap::new(),
            conversations: HashMap::new(),
            messages: HashMap::new(),
            pending_invites: Vec::new(),
            selected_invite: None,
            connection_status: ConnectionStatus::Disconnected,
            terminal_size: (80, 24),
            show_help: false,
            power_tools_mode: PowerToolsMode::Menu,
            debug_logs: Vec::new(),
            power_tools_selection: 0,
        }
    }
}