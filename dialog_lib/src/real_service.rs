use crate::service::MlsService;
use crate::types::{Contact, Conversation, ConnectionStatus};
use crate::errors::{Result, DialogError};
use async_trait::async_trait;
use nostr_mls::prelude::*;
use nostr_mls_memory_storage::NostrMlsMemoryStorage;
use nostr_sdk::prelude::*;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Real MLS service implementation using memory storage
type NostrMlsInstance = NostrMls<NostrMlsMemoryStorage>;

/// Real MLS service implementation using actual Nostr-MLS operations
#[derive(Debug)]
pub struct RealMlsService {
    /// The NostrMls instance for MLS operations
    nostr_mls: Arc<RwLock<NostrMlsInstance>>,
    /// Nostr client for relay communication
    client: Arc<RwLock<Client>>,
    /// Identity keys for this user
    keys: Keys,
    /// Relay URL for communication
    relay_url: String,
    /// Current connection status
    connection_status: Arc<RwLock<ConnectionStatus>>,
}

impl RealMlsService {
    /// Create a new RealMlsService with memory storage
    pub async fn new(keys: Keys, relay_url: String) -> Result<Self> {
        let storage = NostrMlsMemoryStorage::default();
        let nostr_mls = NostrMls::new(storage);
        
        let client = Client::new(keys.clone());
        
        // Add relay 
        client
            .add_relay(&relay_url)
            .await
            .map_err(|e| DialogError::General(Box::new(e)))?;
        
        Ok(Self {
            nostr_mls: Arc::new(RwLock::new(nostr_mls)),
            client: Arc::new(RwLock::new(client)),
            keys,
            relay_url,
            connection_status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
        })
    }

    /// Connect to the relay
    pub async fn connect(&self) -> Result<()> {
        let client = self.client.read().await;
        client.connect().await;
        
        // Update connection status
        let mut status = self.connection_status.write().await;
        *status = ConnectionStatus::Connected;
        
        Ok(())
    }

    /// Disconnect from the relay
    pub async fn disconnect(&self) -> Result<()> {
        let client = self.client.read().await;
        client.disconnect().await;
        
        // Update connection status
        let mut status = self.connection_status.write().await;
        *status = ConnectionStatus::Disconnected;
        
        Ok(())
    }

    /// Generate and publish a key package to the relay
    pub async fn publish_key_package(&self) -> Result<()> {
        let client = self.client.read().await;
        let nostr_mls = self.nostr_mls.read().await;
        
        let relay_url = RelayUrl::parse(&self.relay_url)
            .map_err(|e| DialogError::General(Box::new(e)))?;

        let (key_package_encoded, tags) = nostr_mls
            .create_key_package_for_event(&self.keys.public_key(), [relay_url])
            .map_err(|e| DialogError::General(Box::new(e)))?;

        let key_package_event = EventBuilder::new(Kind::MlsKeyPackage, key_package_encoded)
            .tags(tags)
            .sign_with_keys(&self.keys)
            .map_err(|e| DialogError::General(Box::new(e)))?;

        client
            .send_event(&key_package_event)
            .await
            .map_err(|e| DialogError::General(Box::new(e)))?;

        Ok(())
    }

    /// Find a group by its ID (supports both MLS Group ID and Nostr Group ID)
    async fn find_group_by_id(&self, group_id_hex: &str) -> Result<group_types::Group> {
        let nostr_mls = self.nostr_mls.read().await;
        
        let groups = nostr_mls.get_groups()
            .map_err(|e| DialogError::General(Box::new(e)))?;
        
        // Try as MLS Group ID first (32 hex chars)
        if group_id_hex.len() == 32 {
            if let Ok(group_id_bytes) = hex::decode(group_id_hex) {
                let mls_group_id = GroupId::from_slice(&group_id_bytes);
                for group in &groups {
                    if group.mls_group_id == mls_group_id {
                        return Ok(group.clone());
                    }
                }
            }
        }
        
        // Try as Nostr Group ID (64 hex chars)
        if group_id_hex.len() == 64 {
            if let Ok(nostr_group_id_bytes) = hex::decode(group_id_hex) {
                for group in &groups {
                    if group.nostr_group_id.as_slice() == nostr_group_id_bytes.as_slice() {
                        return Ok(group.clone());
                    }
                }
            }
        }
        
        Err(DialogError::General("Group not found".into()))
    }
}

#[async_trait]
impl MlsService for RealMlsService {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn get_contacts(&self) -> Result<Vec<Contact>> {
        // TODO: Implement real contact storage
        // For now, return empty contacts - this will be implemented when we add contact management
        Ok(vec![])
    }

    async fn get_conversations(&self) -> Result<Vec<Conversation>> {
        let nostr_mls = self.nostr_mls.read().await;
        
        let groups = nostr_mls.get_groups()
            .map_err(|e| DialogError::General(Box::new(e)))?;

        let mut conversations = Vec::new();
        for group in groups {
            let conversation = Conversation {
                id: hex::encode(group.mls_group_id.as_slice()),
                group_id: Some(group.mls_group_id.clone()),
                name: group.name.clone(),
                participants: vec![], // TODO: Extract participants from group
                last_message: None,   // TODO: Get last message from storage
                unread_count: 0,      // TODO: Implement unread tracking
                is_group: true,
            };
            conversations.push(conversation);
        }

        Ok(conversations)
    }

    async fn get_connection_status(&self) -> Result<ConnectionStatus> {
        let status = self.connection_status.read().await;
        Ok(*status)
    }

    async fn send_message(&self, group_id: &GroupId, content: &str) -> Result<()> {
        let client = self.client.read().await;
        let nostr_mls = self.nostr_mls.read().await;

        // Create message rumor
        let rumor = EventBuilder::new(Kind::TextNote, content).build(self.keys.public_key());

        // Create MLS message
        let message_event = nostr_mls.create_message(group_id, rumor)?;
        
        // Process locally for state sync (required in MLS)
        nostr_mls.process_message(&message_event)?;

        // Send to relay
        client
            .send_event(&message_event)
            .await
            .map_err(|e| DialogError::General(Box::new(e)))?;

        Ok(())
    }

    async fn create_conversation(&self, _name: &str, _participants: Vec<PublicKey>) -> Result<String> {
        // TODO: Implement real group creation
        // This is a complex operation that involves:
        // 1. Fetching key packages for participants from relay
        // 2. Creating MLS group with those key packages
        // 3. Publishing welcome messages to participants
        // For now, return error to indicate not implemented
        Err(DialogError::General("Group creation not yet implemented in RealMlsService".into()))
    }

    async fn add_contact(&self, _pubkey: &str) -> Result<()> {
        // TODO: Implement real contact management
        Err(DialogError::General("Contact management not yet implemented in RealMlsService".into()))
    }

    async fn switch_conversation(&self, _conversation_id: &str) -> Result<()> {
        // TODO: Implement conversation switching with real state management
        Ok(())
    }

    async fn get_active_conversation(&self) -> Result<Option<String>> {
        // TODO: Implement real active conversation tracking
        Ok(None)
    }

    async fn get_pending_invites_count(&self) -> Result<usize> {
        let nostr_mls = self.nostr_mls.read().await;
        
        let pending_welcomes = nostr_mls.get_pending_welcomes()
            .map_err(|e| DialogError::General(Box::new(e)))?;
        
        Ok(pending_welcomes.len())
    }

    async fn toggle_connection(&self) -> Result<ConnectionStatus> {
        let current_status = {
            let status = self.connection_status.read().await;
            *status
        };

        match current_status {
            ConnectionStatus::Connected => {
                self.disconnect().await?;
                Ok(ConnectionStatus::Disconnected)
            }
            ConnectionStatus::Disconnected => {
                self.connect().await?;
                Ok(ConnectionStatus::Connected)
            }
            ConnectionStatus::Connecting => {
                // If we're in the middle of connecting, just return current status
                Ok(ConnectionStatus::Connecting)
            }
        }
    }
}