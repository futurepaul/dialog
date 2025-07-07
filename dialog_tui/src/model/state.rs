use std::collections::{HashMap, HashSet};
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

#[derive(Debug, Clone, PartialEq)]
pub struct Conversation {
    pub id: ConversationId,
    pub group_id: Option<GroupId>,
    pub name: String,
    pub participants: Vec<PublicKey>,
    pub last_message_time: Option<chrono::DateTime<chrono::Utc>>,
    pub unread_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingInvite {
    pub id: String,
    pub from: PublicKey,
    pub petname: String, // Will be "Unknown" if not in contacts
    pub group_name: Option<String>,
    pub event_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionState {
    None,
    Contact(ContactId),
    Conversation(ConversationId),
    Invite(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContactField {
    Pubkey,
    Petname,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogState {
    None,
    AddContact {
        current_field: ContactField,
        pubkey: String,
        petname: String,
    },
    CreateConversation {
        selected_contact_index: usize,
    },
    PublishKeypackage,
    AcceptInvite {
        selected_invite_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PowerToolAction {
    ResetAllState,
    DeleteAllContacts,
    DeleteAllConversations,
    RescanRelays,
    RepublishKeypackage,
    ViewDebugLog,
    FetchMessages,
    FetchInvites,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PowerToolsState {
    Menu { selected_action: PowerToolAction },
    DebugLog { scroll_offset: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationListItem {
    Invite { index: usize, invite: PendingInvite },
    Separator,
    Conversation { id: ConversationId, conversation: Conversation },
}

// Legacy enums for backwards compatibility during migration
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
pub struct ToastNotification {
    pub message: String,
    pub level: String, // "WARN", "ERROR", "INFO"
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_secs: u64, // How long to show the toast
}

#[derive(Debug, Clone)]
pub struct LegacyDialogState {
    pub mode: DialogMode,
    pub input_buffer: String,
    pub field_index: usize, // For multi-field dialogs
    pub stored_fields: Vec<String>, // For storing previous field values
}

#[derive(Debug, Clone)]
pub struct AppState {
    // UI State
    pub active_pane: ActivePane,
    pub selection_state: SelectionState,
    pub input_buffer: String,
    pub scroll_offset: u16,
    pub dialog_state: LegacyDialogState, // Keep the old name for backwards compatibility
    pub power_tools_state: PowerToolsState,
    
    // New state management
    pub new_dialog_state: DialogState,
    
    // Legacy state for backwards compatibility during migration
    pub selected_contact: Option<ContactId>,
    pub selected_conversation: Option<ConversationId>,
    
    // Data State
    pub contacts: HashMap<ContactId, Contact>,
    pub conversations: HashMap<ConversationId, Conversation>,
    pub messages: HashMap<ConversationId, Vec<ChatMessage>>,
    pub pending_invites: Vec<PendingInvite>,
    pub selected_invite: Option<usize>,
    pub processed_message_ids: HashSet<String>, // Track processed message IDs to prevent reprocessing
    
    // Network State
    pub connection_status: ConnectionStatus,
    
    // UI Layout State
    pub terminal_size: (u16, u16),
    pub show_help: bool,
    pub power_tools_mode: PowerToolsMode,
    pub debug_logs: Vec<LogEntry>,
    pub power_tools_selection: usize,
    pub toast_notifications: Vec<ToastNotification>,
}

impl DialogState {
    pub fn next_field(self) -> Self {
        match self {
            DialogState::AddContact { current_field: ContactField::Pubkey, pubkey, petname } => {
                DialogState::AddContact { 
                    current_field: ContactField::Petname, 
                    pubkey, 
                    petname 
                }
            }
            DialogState::AddContact { current_field: ContactField::Petname, pubkey, petname } => {
                // Stay on the same field - caller should handle submission
                DialogState::AddContact { 
                    current_field: ContactField::Petname, 
                    pubkey, 
                    petname 
                }
            }
            other => other,
        }
    }
    
    pub fn get_current_input(&self) -> String {
        match self {
            DialogState::AddContact { current_field: ContactField::Pubkey, pubkey, .. } => pubkey.clone(),
            DialogState::AddContact { current_field: ContactField::Petname, petname, .. } => petname.clone(),
            _ => String::new(),
        }
    }
    
    pub fn update_current_input(self, input: String) -> Self {
        match self {
            DialogState::AddContact { current_field: ContactField::Pubkey, petname, .. } => {
                DialogState::AddContact { 
                    current_field: ContactField::Pubkey, 
                    pubkey: input, 
                    petname 
                }
            }
            DialogState::AddContact { current_field: ContactField::Petname, pubkey, .. } => {
                DialogState::AddContact { 
                    current_field: ContactField::Petname, 
                    pubkey, 
                    petname: input 
                }
            }
            other => other,
        }
    }
}

impl PowerToolsState {
    pub fn get_selected_action(&self) -> Option<PowerToolAction> {
        match self {
            PowerToolsState::Menu { selected_action } => Some(selected_action.clone()),
            PowerToolsState::DebugLog { .. } => None,
        }
    }
    
    pub fn next_action(self) -> Self {
        match self {
            PowerToolsState::Menu { selected_action } => {
                let next_action = match selected_action {
                    PowerToolAction::ResetAllState => PowerToolAction::DeleteAllContacts,
                    PowerToolAction::DeleteAllContacts => PowerToolAction::DeleteAllConversations,
                    PowerToolAction::DeleteAllConversations => PowerToolAction::RescanRelays,
                    PowerToolAction::RescanRelays => PowerToolAction::RepublishKeypackage,
                    PowerToolAction::RepublishKeypackage => PowerToolAction::ViewDebugLog,
                    PowerToolAction::ViewDebugLog => PowerToolAction::FetchMessages,
                    PowerToolAction::FetchMessages => PowerToolAction::FetchInvites,
                    PowerToolAction::FetchInvites => PowerToolAction::ResetAllState,
                };
                PowerToolsState::Menu { selected_action: next_action }
            }
            other => other,
        }
    }
    
    pub fn prev_action(self) -> Self {
        match self {
            PowerToolsState::Menu { selected_action } => {
                let prev_action = match selected_action {
                    PowerToolAction::ResetAllState => PowerToolAction::FetchInvites,
                    PowerToolAction::DeleteAllContacts => PowerToolAction::ResetAllState,
                    PowerToolAction::DeleteAllConversations => PowerToolAction::DeleteAllContacts,
                    PowerToolAction::RescanRelays => PowerToolAction::DeleteAllConversations,
                    PowerToolAction::RepublishKeypackage => PowerToolAction::RescanRelays,
                    PowerToolAction::ViewDebugLog => PowerToolAction::RepublishKeypackage,
                    PowerToolAction::FetchMessages => PowerToolAction::ViewDebugLog,
                    PowerToolAction::FetchInvites => PowerToolAction::FetchMessages,
                };
                PowerToolsState::Menu { selected_action: prev_action }
            }
            other => other,
        }
    }
}

impl AppState {
    pub fn get_conversation_list_items(&self) -> Vec<ConversationListItem> {
        let mut items = Vec::new();
        
        // Add pending invites
        for (index, invite) in self.pending_invites.iter().enumerate() {
            items.push(ConversationListItem::Invite { 
                index, 
                invite: invite.clone() 
            });
        }
        
        // Add separator if we have both invites and conversations
        if !self.pending_invites.is_empty() && !self.conversations.is_empty() {
            items.push(ConversationListItem::Separator);
        }
        
        // Add conversations
        for (id, conversation) in &self.conversations {
            items.push(ConversationListItem::Conversation { 
                id: id.clone(), 
                conversation: conversation.clone() 
            });
        }
        
        items
    }
    
    pub fn find_conversation_index(&self, conversation_id: &ConversationId) -> usize {
        let items = self.get_conversation_list_items();
        items.iter()
            .position(|item| matches!(item, ConversationListItem::Conversation { id, .. } if id == conversation_id))
            .unwrap_or(0)
    }
    
    pub fn find_invite_index(&self, invite_index: usize) -> usize {
        let items = self.get_conversation_list_items();
        items.iter()
            .position(|item| matches!(item, ConversationListItem::Invite { index, .. } if *index == invite_index))
            .unwrap_or(0)
    }
    
    /// New enum-based navigation - eliminates complex if/else chains
    pub fn navigate_conversations_down(&self) -> SelectionState {
        let items = self.get_conversation_list_items();
        if items.is_empty() {
            return SelectionState::None;
        }
        
        let current_index = match &self.selection_state {
            SelectionState::Invite(i) => self.find_invite_index(*i),
            SelectionState::Conversation(id) => self.find_conversation_index(id),
            _ => 0,
        };
        
        let next_index = (current_index + 1) % items.len();
        
        // Skip separators
        let next_index = if matches!(items.get(next_index), Some(ConversationListItem::Separator)) {
            (next_index + 1) % items.len()
        } else {
            next_index
        };
        
        match &items[next_index] {
            ConversationListItem::Invite { index, .. } => SelectionState::Invite(*index),
            ConversationListItem::Conversation { id, .. } => SelectionState::Conversation(id.clone()),
            ConversationListItem::Separator => SelectionState::None, // Shouldn't happen after skipping
        }
    }
    
    pub fn navigate_conversations_up(&self) -> SelectionState {
        let items = self.get_conversation_list_items();
        if items.is_empty() {
            return SelectionState::None;
        }
        
        let current_index = match &self.selection_state {
            SelectionState::Invite(i) => self.find_invite_index(*i),
            SelectionState::Conversation(id) => self.find_conversation_index(id),
            _ => items.len() - 1,
        };
        
        let prev_index = if current_index == 0 { 
            items.len() - 1 
        } else { 
            current_index - 1 
        };
        
        // Skip separators
        let prev_index = if matches!(items.get(prev_index), Some(ConversationListItem::Separator)) {
            if prev_index == 0 { 
                items.len() - 1 
            } else { 
                prev_index - 1 
            }
        } else {
            prev_index
        };
        
        match &items[prev_index] {
            ConversationListItem::Invite { index, .. } => SelectionState::Invite(*index),
            ConversationListItem::Conversation { id, .. } => SelectionState::Conversation(id.clone()),
            ConversationListItem::Separator => SelectionState::None, // Shouldn't happen after skipping
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_pane: ActivePane::Contacts,
            selection_state: SelectionState::None,
            input_buffer: String::new(),
            scroll_offset: 0,
            dialog_state: LegacyDialogState {
                mode: DialogMode::Normal,
                input_buffer: String::new(),
                field_index: 0,
                stored_fields: Vec::new(),
            },
            power_tools_state: PowerToolsState::Menu { selected_action: PowerToolAction::ResetAllState },
            new_dialog_state: DialogState::None,
            selected_contact: None,
            selected_conversation: None,
            contacts: HashMap::new(),
            conversations: HashMap::new(),
            messages: HashMap::new(),
            pending_invites: Vec::new(),
            selected_invite: None,
            processed_message_ids: HashSet::new(),
            connection_status: ConnectionStatus::Disconnected,
            terminal_size: (80, 24),
            show_help: false,
            power_tools_mode: PowerToolsMode::Menu,
            debug_logs: Vec::new(),
            power_tools_selection: 0,
            toast_notifications: Vec::new(),
        }
    }
}