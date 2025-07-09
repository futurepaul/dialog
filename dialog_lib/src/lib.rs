pub mod types;
pub mod errors;
pub mod service;
pub mod mock_service;

// Re-export commonly used types
pub use types::*;
pub use errors::*;
pub use service::MlsService;
pub use mock_service::MockMlsService;

use std::sync::Arc;

/// Main interface for the dialog library
#[derive(Debug)]
pub struct DialogLib {
    service: Arc<dyn MlsService>,
}

impl DialogLib {
    /// Create a new DialogLib instance with mock service
    pub fn new_mock() -> Self {
        Self {
            service: Arc::new(MockMlsService::new()),
        }
    }
    
    /// Create a new DialogLib instance with a custom service
    pub fn new(service: Arc<dyn MlsService>) -> Self {
        Self { service }
    }
    
    /// Get all contacts
    pub async fn get_contacts(&self) -> Result<Vec<Contact>> {
        self.service.get_contacts().await
    }
    
    /// Get all conversations
    pub async fn get_conversations(&self) -> Result<Vec<Conversation>> {
        self.service.get_conversations().await
    }
    
    /// Get connection status
    pub async fn get_connection_status(&self) -> Result<ConnectionStatus> {
        self.service.get_connection_status().await
    }
    
    /// Send a message to a conversation
    pub async fn send_message(&self, group_id: &nostr_mls::prelude::GroupId, content: &str) -> Result<()> {
        self.service.send_message(group_id, content).await
    }
    
    /// Create a new conversation
    pub async fn create_conversation(&self, name: &str, participants: Vec<nostr_mls::prelude::PublicKey>) -> Result<String> {
        self.service.create_conversation(name, participants).await
    }
    
    /// Add a contact
    pub async fn add_contact(&self, pubkey: &str) -> Result<()> {
        self.service.add_contact(pubkey).await
    }
    
    /// Switch to a conversation
    pub async fn switch_conversation(&self, conversation_id: &str) -> Result<()> {
        self.service.switch_conversation(conversation_id).await
    }
    
    /// Get the active conversation ID
    pub async fn get_active_conversation(&self) -> Result<Option<String>> {
        self.service.get_active_conversation().await
    }
    
    /// Get the number of pending invites
    pub async fn get_pending_invites_count(&self) -> Result<usize> {
        self.service.get_pending_invites_count().await
    }
    
    /// Toggle connection status
    pub async fn toggle_connection(&self) -> Result<ConnectionStatus> {
        self.service.toggle_connection().await
    }
    
    /// Get access to the underlying service (for advanced operations)
    pub fn service(&self) -> &Arc<dyn MlsService> {
        &self.service
    }
    
    /// Get access to the mock service (if using mock implementation)
    pub fn mock_service(&self) -> Option<&MockMlsService> {
        self.service.as_any().downcast_ref::<MockMlsService>()
    }
}
