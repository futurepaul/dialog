use nostr_mls::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    MessageInput,
    CommandInput,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Nostr user profile information (Kind 0 event content)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    /// Display name (preferred name to show)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Username/handle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Profile description/bio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /// Profile picture URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    /// Banner image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    /// Website URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    /// Lightning address for zaps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lud16: Option<String>,
}

impl Profile {
    /// Get the best available name for display
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
            .or_else(|| self.name.as_deref())
    }
    
    /// Create a new empty profile
    pub fn new() -> Self {
        Self {
            display_name: None,
            name: None,
            about: None,
            picture: None,
            banner: None,
            website: None,
            lud16: None,
        }
    }
    
    /// Create a profile with just a name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            display_name: Some(name.into()),
            name: None,
            about: None,
            picture: None,
            banner: None,
            website: None,
            lud16: None,
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::new()
    }
}

pub enum AppResult {
    Continue,
    Exit,
}

/// Pending group invitation
#[derive(Debug, Clone)]
pub struct PendingInvite {
    pub group_id: GroupId,
    pub group_name: String,
    pub inviter: Option<PublicKey>,
    pub member_count: usize,
    pub timestamp: i64,
}

/// A decrypted message in a conversation
#[derive(Debug, Clone)]
pub struct Message {
    /// The sender's public key
    pub sender: PublicKey,
    /// The message content
    pub content: String,
    /// Timestamp when the message was sent (Unix timestamp in seconds)
    pub timestamp: i64,
    /// Message ID (event ID)
    pub id: Option<String>,
}

/// Result of listing pending invites, includes both invites and any processing errors
#[derive(Debug, Clone)]
pub struct InviteListResult {
    pub invites: Vec<PendingInvite>,
    pub processing_errors: Vec<String>,
}

/// Result of fetching messages, includes both messages and any processing errors
#[derive(Debug, Clone)]
pub struct MessageFetchResult {
    pub messages: Vec<Message>,
    pub processing_errors: Vec<String>,
}

/// Nostr event kinds
pub mod nostr_kinds {
    pub const METADATA: u16 = 0;
}

/// UI update events for real-time messaging
#[derive(Debug, Clone)]
pub enum UiUpdate {
    /// New message received in a group
    NewMessage { group_id: GroupId, message: Message },
    /// Group state changed (e.g., member joined/left)
    GroupStateChange { group_id: GroupId, epoch: u64 },
    /// Connection status changed
    ConnectionStatus(ConnectionStatus),
    /// New invitation received
    NewInvite(PendingInvite),
    /// Group has new messages (triggers a fetch)
    GroupHasNewMessages { group_id: GroupId },
}