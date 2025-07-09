pub mod types;
pub mod errors;
pub mod service;
pub mod mock_service;
pub mod real_service;
pub mod config;

// Re-export commonly used types
pub use types::*;
pub use errors::*;
pub use service::MlsService;
pub use mock_service::MockMlsService;
pub use real_service::RealMlsService;
pub use config::{DialogConfig, ServiceMode};

// Re-export Nostr-MLS types to eliminate direct dependencies in UIs
pub use nostr_mls::prelude::{
    PublicKey, GroupId, Keys,
};

// Re-export nostr utilities
pub use nostr::nips::nip19::ToBech32;

// Re-export hex utilities
pub use hex;

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
    
    /// Create a new DialogLib instance with mock service and pre-loaded data
    pub async fn new_mock_with_data() -> Self {
        Self {
            service: Arc::new(MockMlsService::new_with_data().await),
        }
    }
    
    /// Create a new DialogLib instance with a custom service
    pub fn new_with_service(service: Arc<dyn MlsService>) -> Self {
        Self { service }
    }
    
    /// Create a new DialogLib instance based on configuration
    pub async fn from_config(config: DialogConfig) -> Result<Self> {
        match config.mode {
            ServiceMode::Mock => Ok(Self::new_mock_with_data().await),
            ServiceMode::Real => {
                // Generate keys for this instance
                let keys = nostr_mls::prelude::Keys::generate();
                
                let service: Arc<dyn MlsService> = Arc::new(
                    RealMlsService::new(keys, config.relay_url).await?
                );
                
                Ok(Self::new_with_service(service))
            }
        }
    }
    
    /// Create a new DialogLib instance from environment variables
    pub async fn from_env() -> Result<Self> {
        let config = DialogConfig::from_env();
        Self::from_config(config).await
    }
    
    /// Create a new DialogLib instance with generated keys
    pub async fn new() -> Result<Self> {
        let config = DialogConfig::builder()
            .mode(ServiceMode::Real)
            .build();
        Self::from_config(config).await
    }
    
    /// Create a new DialogLib instance with specific keys
    pub async fn new_with_keys(keys: nostr_mls::prelude::Keys) -> Result<Self> {
        let service: Arc<dyn MlsService> = Arc::new(
            RealMlsService::new(keys, "ws://localhost:8080".to_string()).await?
        );
        Ok(Self::new_with_service(service))
    }
    
    /// Create a new DialogLib instance with custom configuration
    pub async fn new_with_config(
        mode: ServiceMode,
        relay_url: impl Into<String>,
    ) -> Result<Self> {
        let config = DialogConfig::builder()
            .mode(mode)
            .relay_url(relay_url)
            .build();
        Self::from_config(config).await
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
    
    /// Get the user's own public key
    pub async fn get_own_pubkey(&self) -> Result<PublicKey> {
        self.service.get_own_pubkey().await
    }
}
