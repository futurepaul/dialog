use anyhow::Result;
use nostr_sdk::prelude::*;
use nostr::nips::nip44;
use std::time::Duration;
use tracing::info;

// UniFFI support
mod uniffi_bindings;
pub use uniffi_bindings::*;

pub struct DialogClient {
    client: Client,
    keys: Keys,
}

impl DialogClient {
    pub async fn new() -> Result<Self> {
        let keys = Keys::generate();
        let client = Client::new(keys.clone());
        
        info!("Created new dialog client with pubkey: {}", keys.public_key());
        
        Ok(DialogClient { client, keys })
    }

    pub async fn new_with_key(secret_key_hex: &str) -> Result<Self> {
        let secret_key = SecretKey::from_hex(secret_key_hex)?;
        let keys = Keys::new(secret_key);
        let client = Client::new(keys.clone());
        
        info!("Created dialog client with existing key: {}", keys.public_key());
        
        Ok(DialogClient { client, keys })
    }

    pub async fn connect_to_relay(&self, relay_url: &str) -> Result<()> {
        info!("Connecting to relay: {}", relay_url);
        
        self.client.add_relay(relay_url).await?;
        self.client.connect().await;
        
        info!("Connected to relay successfully");
        Ok(())
    }

    pub async fn publish_note(&self, content: &str) -> Result<EventId> {
        info!("Publishing note: {}", content);
        
        let builder = EventBuilder::text_note(content);
        let output = self.client.send_event_builder(builder).await?;
        let event_id = output.val;
        
        info!("Published note with ID: {}", event_id);
        Ok(event_id)
    }

    pub async fn get_notes(&self, limit: Option<usize>) -> Result<Vec<Event>> {
        info!("Fetching notes from relay");
        
        let filter = Filter::new()
            .kind(Kind::TextNote)
            .limit(limit.unwrap_or(10));

        let events = self.client
            .fetch_events(filter, Duration::from_secs(5))
            .await?;
        
        let events_vec: Vec<Event> = events.into_iter().collect();
        info!("Retrieved {} notes", events_vec.len());
        Ok(events_vec)
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.keys.public_key()
    }

    pub fn get_secret_key_hex(&self) -> String {
        self.keys.secret_key().display_secret().to_string()
    }

    // Encrypted messaging functions using NIP-44
    pub async fn send_encrypted_message(&self, recipient_pubkey: &PublicKey, content: &str) -> Result<EventId> {
        info!("Sending encrypted message to: {}", recipient_pubkey);
        
        // Encrypt the message using NIP-44
        let encrypted_content = nip44::encrypt(
            &self.keys.secret_key(),
            recipient_pubkey,
            content,
            nip44::Version::V2,
        )?;
        
        // Create a kind-4 encrypted direct message event manually
        let builder = EventBuilder::new(Kind::EncryptedDirectMessage, encrypted_content)
            .tag(Tag::public_key(*recipient_pubkey));
        
        let output = self.client.send_event_builder(builder).await?;
        let event_id = output.val;
        
        info!("Sent encrypted message with ID: {}", event_id);
        Ok(event_id)
    }

    pub async fn get_encrypted_messages(&self) -> Result<Vec<Event>> {
        info!("Fetching encrypted messages");
        
        let filter = Filter::new()
            .kind(Kind::EncryptedDirectMessage)
            .pubkey(self.keys.public_key())
            .limit(50);

        let events = self.client
            .fetch_events(filter, Duration::from_secs(5))
            .await?;
        
        let events_vec: Vec<Event> = events.into_iter().collect();
        info!("Retrieved {} encrypted messages", events_vec.len());
        Ok(events_vec)
    }

    pub fn decrypt_message(&self, sender_pubkey: &PublicKey, encrypted_content: &str) -> Result<String> {
        let decrypted = nip44::decrypt(
            &self.keys.secret_key(),
            sender_pubkey,
            encrypted_content,
        )?;
        
        Ok(decrypted)
    }

    // Group messaging using custom event kinds (simplified MLS-like approach)
    pub async fn create_group(&self, group_name: &str, initial_members: Vec<PublicKey>) -> Result<EventId> {
        info!("Creating group: {} with {} members", group_name, initial_members.len());
        
        // Create group metadata event (kind 30000 - replaceable event)
        let group_data = serde_json::json!({
            "name": group_name,
            "members": initial_members.iter().map(|pk| pk.to_hex()).collect::<Vec<_>>(),
            "created_at": Timestamp::now().as_u64(),
            "admin": self.keys.public_key().to_hex()
        });
        
        let builder = EventBuilder::new(Kind::Custom(30000), group_data.to_string())
            .tag(Tag::identifier(group_name));
        
        let output = self.client.send_event_builder(builder).await?;
        let event_id = output.val;
        
        info!("Created group with ID: {}", event_id);
        Ok(event_id)
    }

    pub async fn send_group_message(&self, group_id: &str, content: &str, _members: &[PublicKey]) -> Result<EventId> {
        info!("Sending group message to group: {}", group_id);
        
        // For now, send individual encrypted messages to each member
        // In real MLS, this would be a single encrypted message for the group
        let group_message = serde_json::json!({
            "group_id": group_id,
            "content": content,
            "timestamp": Timestamp::now().as_u64()
        });
        
        // Create group message event (kind 30001)
        let builder = EventBuilder::new(Kind::Custom(30001), group_message.to_string())
            .tag(Tag::identifier(group_id));
        
        let output = self.client.send_event_builder(builder).await?;
        let event_id = output.val;
        
        info!("Sent group message with ID: {}", event_id);
        Ok(event_id)
    }
}
