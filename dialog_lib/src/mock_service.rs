use crate::service::MlsService;
use crate::types::{Contact, Conversation, ConnectionStatus};
use crate::errors::{DialogError, Result};
use nostr_mls::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct MockMlsService {
    contacts: Arc<RwLock<Vec<Contact>>>,
    conversations: Arc<RwLock<Vec<Conversation>>>,
    connection_status: Arc<RwLock<ConnectionStatus>>,
    active_conversation: Arc<RwLock<Option<String>>>,
    pending_invites: Arc<RwLock<usize>>,
    own_pubkey: PublicKey,
}

impl MockMlsService {
    pub fn new() -> Self {
        let service = Self {
            contacts: Arc::new(RwLock::new(Vec::new())),
            conversations: Arc::new(RwLock::new(Vec::new())),
            connection_status: Arc::new(RwLock::new(ConnectionStatus::Connected)),
            active_conversation: Arc::new(RwLock::new(None)),
            pending_invites: Arc::new(RwLock::new(2)),
            own_pubkey: Keys::generate().public_key(),
        };
        
        service
    }
    
    pub async fn new_with_data() -> Self {
        let service = Self::new();
        service.setup_fake_data().await;
        service
    }
    
    async fn setup_fake_data(&self) {
        // Generate real keys for mock data
        let alice_key = Keys::generate().public_key();
        let bob_key = Keys::generate().public_key();
        let charlie_key = Keys::generate().public_key();
        let diana_key = Keys::generate().public_key();
        
        // Add fake contacts with real keys
        let mut contacts = self.contacts.write().await;
        contacts.push(Contact {
            name: "Alice".to_string(),
            pubkey: alice_key,
            online: true,
        });
        contacts.push(Contact {
            name: "Bob".to_string(),
            pubkey: bob_key,
            online: false,
        });
        contacts.push(Contact {
            name: "Charlie".to_string(),
            pubkey: charlie_key,
            online: true,
        });
        contacts.push(Contact {
            name: "Diana".to_string(),
            pubkey: diana_key,
            online: true,
        });
        drop(contacts);

        // Generate real group IDs for mock conversations
        let alice_group_id = GroupId::from_slice(&hex::decode("1111111111111111111111111111111111111111111111111111111111111111").unwrap());
        let dev_group_id = GroupId::from_slice(&hex::decode("2222222222222222222222222222222222222222222222222222222222222222").unwrap());
        let charlie_group_id = GroupId::from_slice(&hex::decode("3333333333333333333333333333333333333333333333333333333333333333").unwrap());

        // Add fake conversations with real group IDs
        let mut conversations = self.conversations.write().await;
        conversations.push(Conversation {
            id: "conv-alice".to_string(),
            group_id: Some(alice_group_id),
            name: "Alice".to_string(),
            participants: vec![alice_key],
            last_message: Some("Hey! How's it going?".to_string()),
            unread_count: 2,
            is_group: false,
        });
        conversations.push(Conversation {
            id: "conv-group-dev".to_string(),
            group_id: Some(dev_group_id),
            name: "Development Team".to_string(),
            participants: vec![alice_key, bob_key, charlie_key],
            last_message: Some("Alice: Let's sync up tomorrow".to_string()),
            unread_count: 0,
            is_group: true,
        });
        conversations.push(Conversation {
            id: "conv-charlie".to_string(),
            group_id: Some(charlie_group_id),
            name: "Charlie".to_string(),
            participants: vec![charlie_key],
            last_message: Some("Thanks for the help earlier!".to_string()),
            unread_count: 1,
            is_group: false,
        });
    }
    
    pub async fn generate_fake_response(&self, message: &str, conv: &Conversation) -> String {
        // Use message content hash for deterministic but varied responses
        let mut hasher = DefaultHasher::new();
        message.hash(&mut hasher);
        let hash = hasher.finish() as usize;

        if conv.is_group {
            let responses = [
                "Alice: That's interesting!",
                "Bob: I agree with that.",
                "Charlie: Good point!",
                "Diana: Thanks for sharing!",
                "Alice: I hadn't thought of that.",
                "Bob: Let's discuss this more.",
                "Charlie: Makes sense to me.",
                "Diana: Can you explain more?",
            ];
            responses.get(hash % responses.len()).unwrap_or(&"Alice: Thanks!").to_string()
        } else {
            let responses = [
                "Sounds good!",
                "I see what you mean.",
                "That's interesting to hear.",
                "Thanks for letting me know!",
                "I'll think about that.",
                "Good to hear from you!",
                "Let me get back to you on that.",
                "That makes sense.",
            ];
            let response = responses.get(hash % responses.len()).unwrap_or(&"Thanks!");
            format!("{}: {}", conv.name, response)
        }
    }
    
    pub async fn find_contact_by_name(&self, name: &str) -> Option<Contact> {
        let contacts = self.contacts.read().await;
        contacts.iter()
            .find(|c| c.name.to_lowercase() == name.to_lowercase())
            .cloned()
    }
    
    pub async fn get_conversation_by_id(&self, id: &str) -> Option<Conversation> {
        let conversations = self.conversations.read().await;
        conversations.iter()
            .find(|c| c.id == id)
            .cloned()
    }
}

impl Default for MockMlsService {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MockMlsService {
    fn clone(&self) -> Self {
        Self {
            contacts: Arc::clone(&self.contacts),
            conversations: Arc::clone(&self.conversations),
            connection_status: Arc::clone(&self.connection_status),
            active_conversation: Arc::clone(&self.active_conversation),
            pending_invites: Arc::clone(&self.pending_invites),
            own_pubkey: self.own_pubkey,
        }
    }
}

#[async_trait::async_trait]
impl MlsService for MockMlsService {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    async fn get_contacts(&self) -> Result<Vec<Contact>> {
        let contacts = self.contacts.read().await;
        Ok(contacts.clone())
    }

    async fn get_conversations(&self) -> Result<Vec<Conversation>> {
        let conversations = self.conversations.read().await;
        Ok(conversations.clone())
    }

    async fn get_connection_status(&self) -> Result<ConnectionStatus> {
        let status = self.connection_status.read().await;
        Ok(status.clone())
    }

    async fn send_message(&self, _group_id: &GroupId, _content: &str) -> Result<()> {
        // Mock implementation - in real implementation would send to MLS group
        Ok(())
    }

    async fn create_conversation(&self, name: &str, participants: Vec<PublicKey>) -> Result<String> {
        let conv_id = format!("conv-{}", name.to_lowercase());
        let mut conversations = self.conversations.write().await;
        
        // Check if conversation already exists
        if conversations.iter().any(|c| c.id == conv_id) {
            return Err(DialogError::InvalidCommand(format!("Conversation with {} already exists", name)));
        }
        
        // Create mock group ID
        let group_id = GroupId::from_slice(&hex::decode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").unwrap());
        
        conversations.push(Conversation {
            id: conv_id.clone(),
            group_id: Some(group_id),
            name: name.to_string(),
            participants,
            last_message: None,
            unread_count: 0,
            is_group: false,
        });
        
        Ok(conv_id)
    }

    async fn add_contact(&self, pubkey: &str) -> Result<()> {
        // Mock implementation - in real implementation would validate and add contact
        let mut contacts = self.contacts.write().await;
        contacts.push(Contact {
            name: format!("Contact-{}", pubkey.len()),
            pubkey: Keys::generate().public_key(), // Mock key
            online: true,
        });
        Ok(())
    }

    async fn switch_conversation(&self, conversation_id: &str) -> Result<()> {
        let conversations = self.conversations.read().await;
        if conversations.iter().any(|c| c.id == conversation_id) {
            let mut active = self.active_conversation.write().await;
            *active = Some(conversation_id.to_string());
            Ok(())
        } else {
            Err(DialogError::ConversationNotFound(conversation_id.to_string()))
        }
    }

    async fn get_active_conversation(&self) -> Result<Option<String>> {
        let active = self.active_conversation.read().await;
        Ok(active.clone())
    }

    async fn get_pending_invites_count(&self) -> Result<usize> {
        let count = self.pending_invites.read().await;
        Ok(*count)
    }

    async fn toggle_connection(&self) -> Result<ConnectionStatus> {
        let mut status = self.connection_status.write().await;
        status.simulate_connection_change();
        Ok(status.clone())
    }

    async fn get_own_pubkey(&self) -> Result<PublicKey> {
        Ok(self.own_pubkey)
    }
}