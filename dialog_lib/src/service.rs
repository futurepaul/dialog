use crate::types::{Contact, Conversation, ConnectionStatus, Profile, InviteListResult, MessageFetchResult, UiUpdate};
use crate::errors::Result;
use nostr_mls::prelude::*;
use std::any::Any;
use tokio::sync::mpsc;

#[async_trait::async_trait]
pub trait MlsService: Send + Sync + std::fmt::Debug {
    fn as_any(&self) -> &dyn Any;
    async fn get_contacts(&self) -> Result<Vec<Contact>>;
    async fn get_conversations(&self) -> Result<Vec<Conversation>>;
    async fn get_connection_status(&self) -> Result<ConnectionStatus>;
    async fn send_message(&self, group_id: &GroupId, content: &str) -> Result<()>;
    async fn create_conversation(&self, name: &str, participants: Vec<PublicKey>) -> Result<String>;
    async fn add_contact(&self, pubkey: &str) -> Result<()>;
    async fn switch_conversation(&self, conversation_id: &str) -> Result<()>;
    async fn get_active_conversation(&self) -> Result<Option<String>>;
    async fn get_pending_invites_count(&self) -> Result<usize>;
    async fn toggle_connection(&self) -> Result<ConnectionStatus>;
    async fn get_own_pubkey(&self) -> Result<PublicKey>;
    async fn load_profile(&self, pubkey: &PublicKey) -> Result<Option<Profile>>;
    async fn publish_profile(&self, profile: &Profile) -> Result<()>;
    async fn get_relay_url(&self) -> Result<String>;
    
    // New methods for group lifecycle
    async fn publish_key_packages(&self) -> Result<Vec<String>>; // Returns event IDs
    async fn list_pending_invites(&self) -> Result<InviteListResult>;
    async fn accept_invite(&self, group_id: &str) -> Result<()>;
    async fn fetch_and_process_group_events(&self, group_id: &GroupId) -> Result<()>;
    
    // Message fetching
    async fn fetch_messages(&self, group_id: &GroupId) -> Result<MessageFetchResult>;
    
    // Real-time message subscription
    async fn subscribe_to_groups(&self, ui_sender: mpsc::Sender<UiUpdate>) -> Result<()>;
    
    // Refresh subscriptions after group changes
    async fn refresh_subscriptions(&self) -> Result<()>;
}