use crate::types::{Contact, Conversation, ConnectionStatus};
use crate::errors::Result;
use nostr_mls::prelude::*;
use std::any::Any;

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
}