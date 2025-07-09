use nostr_mls::prelude::*;

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

pub enum AppResult {
    Continue,
    Exit,
}