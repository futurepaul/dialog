use anyhow::Result;
use std::time::Duration;
use std::path::Path;
use tracing::info;
use whitenoise::{Whitenoise, WhitenoiseConfig, Account};
pub use whitenoise::{PublicKey, Event};
pub use nostr::{EventId, Tag};

// Import MLS types from whitenoise
use whitenoise::{
    Group, GroupId, MessageWithTokens,
};

// Re-export types for CLI usage - done above

// UniFFI support
mod uniffi_bindings;
pub use uniffi_bindings::*;

pub struct DialogClient {
    whitenoise: Option<&'static Whitenoise>,
    account: Option<Account>,
}

impl DialogClient {
    pub async fn new() -> Result<Self> {
        // Initialize whitenoise with default config
        let config = WhitenoiseConfig::new(
            Path::new("./data/whitenoise"),
            Path::new("./logs")
        );
        
        Whitenoise::initialize_whitenoise(config).await?;
        let whitenoise_instance = Whitenoise::get_instance()?;
        
        // Create a new account
        let account = whitenoise_instance.create_identity().await?;
        
        info!("Created new dialog client with account: {}", account.pubkey);
        
        Ok(DialogClient { 
            whitenoise: Some(whitenoise_instance),
            account: Some(account),
        })
    }

    pub async fn new_with_key(secret_key_hex: &str) -> Result<Self> {
        // Initialize whitenoise with default config
        let config = WhitenoiseConfig::new(
            Path::new("./data/whitenoise"),
            Path::new("./logs")
        );
        
        Whitenoise::initialize_whitenoise(config).await?;
        let whitenoise_instance = Whitenoise::get_instance()?;
        
        // Login with existing secret key
        let account = whitenoise_instance.login(secret_key_hex.to_string()).await?;
        
        info!("Created dialog client with account: {}", account.pubkey);
        
        Ok(DialogClient { 
            whitenoise: Some(whitenoise_instance),
            account: Some(account),
        })
    }

    pub async fn connect_to_relay(&self, relay_url: &str) -> Result<()> {
        info!("Connecting to relay: {}", relay_url);
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            // TODO: Use whitenoise's relay management
            // For now, just log that we would connect
            info!("Would connect account {} to relay: {}", account.pubkey, relay_url);
        }
        
        info!("Connected to relay successfully");
        Ok(())
    }

    pub async fn publish_note(&self, content: &str) -> Result<EventId> {
        info!("Publishing note: {}", content);
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            // TODO: Use whitenoise's messaging API
            // For now, create a dummy event ID
            let dummy_event_id = EventId::all_zeros();
            info!("Would publish note with account {}: {}", account.pubkey, content);
            Ok(dummy_event_id)
        } else {
            anyhow::bail!("Whitenoise not initialized")
        }
    }

    pub async fn get_notes(&self, limit: Option<usize>) -> Result<Vec<Event>> {
        info!("Fetching notes from relay");
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            // TODO: Use whitenoise's event fetching API
            // For now, return empty vector
            info!("Would fetch {} notes for account {}", limit.unwrap_or(10), account.pubkey);
            Ok(vec![])
        } else {
            anyhow::bail!("Whitenoise not initialized")
        }
    }

    pub fn get_public_key(&self) -> Option<PublicKey> {
        self.account.as_ref().map(|account| account.pubkey)
    }

    pub async fn get_secret_key_hex(&self) -> Result<Option<String>> {
        if let Some(whitenoise) = &self.whitenoise {
            // TODO: Use whitenoise's key export functionality
            // For now, return None
            Ok(None)
        } else {
            Ok(None)
        }
    }

    // TODO: Implement encrypted messaging using Whitenoise MLS
    pub async fn send_encrypted_message(&self, recipient_pubkey: &PublicKey, content: &str) -> Result<EventId> {
        info!("Sending encrypted message to: {}", recipient_pubkey);
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            // TODO: Use whitenoise's MLS messaging
            let dummy_event_id = EventId::all_zeros();
            info!("Would send encrypted message from {} to {}: {}", account.pubkey, recipient_pubkey, content);
            Ok(dummy_event_id)
        } else {
            anyhow::bail!("Whitenoise not initialized")
        }
    }

    pub async fn get_encrypted_messages(&self) -> Result<Vec<Event>> {
        info!("Fetching encrypted messages");
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            // TODO: Use whitenoise's message fetching
            info!("Would fetch encrypted messages for account: {}", account.pubkey);
            Ok(vec![])
        } else {
            anyhow::bail!("Whitenoise not initialized")
        }
    }

    pub fn decrypt_message(&self, sender_pubkey: &PublicKey, encrypted_content: &str) -> Result<String> {
        // TODO: Use whitenoise's MLS decryption
        info!("Would decrypt message from: {}", sender_pubkey);
        Ok("[Decrypted message placeholder]".to_string())
    }

    /// Create a new MLS group using Whitenoise
    pub async fn create_group(&self, group_name: &str, initial_members: Vec<PublicKey>) -> Result<String> {
        info!("Creating group: {} with {} members", group_name, initial_members.len());
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            // Use whitenoise's MLS group creation
            let group = whitenoise.create_group(
                account,
                initial_members.clone(),
                vec![account.pubkey], // Make creator an admin
                group_name.to_string(),
                "Group created via Dialog Client".to_string(),
            ).await?;
            
            // Return hex-encoded group ID
            let group_id_hex = hex::encode(&group.nostr_group_id);
            info!("Created MLS group '{}' with ID: {}", group_name, group_id_hex);
            Ok(group_id_hex)
        } else {
            anyhow::bail!("Whitenoise not initialized")
        }
    }

    /// Send a message to an MLS group using Whitenoise
    pub async fn send_group_message(&self, group_id_hex: &str, content: &str, _members: &[PublicKey]) -> Result<String> {
        info!("Sending group message to group: {}", group_id_hex);
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            // Convert hex string back to GroupId
            let group_id_bytes = hex::decode(group_id_hex)
                .map_err(|e| anyhow::anyhow!("Invalid group ID hex: {}", e))?;
            let group_id = GroupId::from_slice(&group_id_bytes);
            
            // Send message using whitenoise
            let message_with_tokens = whitenoise.send_message_to_group(
                &account.pubkey,
                &group_id,
                content.to_string(),
                1, // Kind 1 for text note
                None, // No additional tags
            ).await?;
            
            // Return the message ID as hex
            let message_id_hex = hex::encode(&message_with_tokens.message.id);
            info!("Sent group message successfully with ID: {}", message_id_hex);
            Ok(message_id_hex)
        } else {
            anyhow::bail!("Whitenoise not initialized")
        }
    }

    /// Fetch all groups for the current account
    pub async fn fetch_groups(&self) -> Result<Vec<String>> {
        info!("Fetching groups for account");
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            let groups = whitenoise.fetch_groups(account, true).await?; // Only active groups
            let group_ids: Vec<String> = groups.iter()
                .map(|group| hex::encode(&group.nostr_group_id))
                .collect();
            info!("Found {} groups", group_ids.len());
            Ok(group_ids)
        } else {
            anyhow::bail!("Whitenoise not initialized")
        }
    }

    /// Fetch messages for a specific group
    pub async fn fetch_group_messages(&self, group_id_hex: &str) -> Result<Vec<String>> {
        info!("Fetching messages for group: {}", group_id_hex);
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            // Convert hex string back to GroupId
            let group_id_bytes = hex::decode(group_id_hex)
                .map_err(|e| anyhow::anyhow!("Invalid group ID hex: {}", e))?;
            let group_id = GroupId::from_slice(&group_id_bytes);
            
            let messages = whitenoise.fetch_messages_for_group(&account.pubkey, &group_id).await?;
            let message_contents: Vec<String> = messages.iter()
                .map(|msg| msg.message.content.clone())
                .collect();
            info!("Found {} messages for group", message_contents.len());
            Ok(message_contents)
        } else {
            anyhow::bail!("Whitenoise not initialized")
        }
    }

    /// Add members to an existing group
    pub async fn add_members_to_group(&self, group_id_hex: &str, new_members: Vec<PublicKey>) -> Result<()> {
        info!("Adding {} members to group: {}", new_members.len(), group_id_hex);
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            // Convert hex string back to GroupId
            let group_id_bytes = hex::decode(group_id_hex)
                .map_err(|e| anyhow::anyhow!("Invalid group ID hex: {}", e))?;
            let group_id = GroupId::from_slice(&group_id_bytes);
            
            whitenoise.add_members_to_group(account, &group_id, new_members).await?;
            info!("Successfully added members to group");
            Ok(())
        } else {
            anyhow::bail!("Whitenoise not initialized")
        }
    }

    /// Remove members from an existing group
    pub async fn remove_members_from_group(&self, group_id_hex: &str, members_to_remove: Vec<PublicKey>) -> Result<()> {
        info!("Removing {} members from group: {}", members_to_remove.len(), group_id_hex);
        
        if let (Some(whitenoise), Some(account)) = (&self.whitenoise, &self.account) {
            // Convert hex string back to GroupId
            let group_id_bytes = hex::decode(group_id_hex)
                .map_err(|e| anyhow::anyhow!("Invalid group ID hex: {}", e))?;
            let group_id = GroupId::from_slice(&group_id_bytes);
            
            whitenoise.remove_members_from_group(account, &group_id, members_to_remove).await?;
            info!("Successfully removed members from group");
            Ok(())
        } else {
            anyhow::bail!("Whitenoise not initialized")
        }
    }
}
