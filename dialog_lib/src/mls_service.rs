use crate::service::MlsService;
use crate::types::{Contact, Conversation, ConnectionStatus, Profile, PendingInvite, Message, InviteListResult, MessageFetchResult, UiUpdate};
use crate::errors::{Result, DialogError};
use async_trait::async_trait;
use nostr_mls::prelude::*;
use nostr_mls_memory_storage::NostrMlsMemoryStorage;
use nostr_sdk::prelude::*;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

/// Real MLS service implementation using memory storage
type NostrMlsInstance = NostrMls<NostrMlsMemoryStorage>;

/// Message cache entry with timestamp for ordering
#[derive(Debug, Clone)]
struct CachedMessage {
    message: Message,
    event_id: EventId,
}

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
    /// Runtime storage for contacts (pubkey -> Contact)
    contacts: Arc<RwLock<HashMap<PublicKey, Contact>>>,
    /// Runtime cache for profiles (pubkey -> Profile)
    profiles: Arc<RwLock<HashMap<PublicKey, Profile>>>,
    /// Message cache (group_id -> messages)
    message_cache: Arc<RwLock<HashMap<GroupId, Vec<CachedMessage>>>>,
    /// Last sync timestamp for each group
    last_sync: Arc<RwLock<HashMap<GroupId, i64>>>,
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
            contacts: Arc::new(RwLock::new(HashMap::new())),
            profiles: Arc::new(RwLock::new(HashMap::new())),
            message_cache: Arc::new(RwLock::new(HashMap::new())),
            last_sync: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Connect to the relay
    pub async fn connect(&self) -> Result<()> {
        let client = self.client.read().await;
        
        // Update status to connecting
        {
            let mut status = self.connection_status.write().await;
            *status = ConnectionStatus::Connecting;
        }
        
        // Try to connect to the relay
        client.connect().await;
        
        // Wait a brief moment for connection to establish (reduced from 1000ms)
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        // Test the connection by trying to fetch some events (reduced timeouts)
        let test_result = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            client.fetch_events(Filter::new().limit(1), std::time::Duration::from_secs(1))
        ).await;
        
        match test_result {
            Ok(Ok(_)) => {
                // Connection successful
                let mut status = self.connection_status.write().await;
                *status = ConnectionStatus::Connected;
                Ok(())
            }
            Ok(Err(e)) => {
                // Connection failed
                let mut status = self.connection_status.write().await;
                *status = ConnectionStatus::Disconnected;
                Err(DialogError::General(format!("Failed to connect to relay: {}", e).into()))
            }
            Err(_) => {
                // Timeout
                let mut status = self.connection_status.write().await;
                *status = ConnectionStatus::Disconnected;
                Err(DialogError::General("Connection timeout - relay may not be running".into()))
            }
        }
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
        // Return all contacts from runtime storage
        let contacts = self.contacts.read().await;
        Ok(contacts.values().cloned().collect())
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
        // CRITICAL: Fetch and process any MLS evolution events before sending
        // This ensures our group state is synchronized with other members
        self.fetch_and_process_group_events(group_id).await?;

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

    async fn create_conversation(&self, name: &str, participants: Vec<PublicKey>) -> Result<String> {
        let client = self.client.read().await;
        let nostr_mls = self.nostr_mls.read().await;

        // Ensure we're connected
        let status = self.connection_status.read().await;
        if *status != ConnectionStatus::Connected {
            return Err(DialogError::General("Not connected to relay".into()));
        }

        // Validate we have participants
        if participants.is_empty() {
            return Err(DialogError::General("Cannot create group without participants".into()));
        }

        // Collect key package events for all participants
        let mut key_package_events = Vec::new();
        
        for participant in &participants {
            // Fetch key packages for this participant
            let filter = Filter::new()
                .kind(Kind::MlsKeyPackage)
                .author(*participant);
            
            let events = client
                .fetch_events(filter, std::time::Duration::from_secs(5))
                .await
                .map_err(|e| DialogError::General(format!("Failed to fetch key packages: {}", e).into()))?;
            
            if let Some(key_package_event) = events.first() {
                // Validate the key package
                nostr_mls.parse_key_package(key_package_event)
                    .map_err(|e| DialogError::General(format!("Invalid key package from {}: {}", participant.to_hex(), e).into()))?;
                
                key_package_events.push(key_package_event.clone());
            } else {
                return Err(DialogError::General(
                    format!("No key package found for participant: {}", participant.to_hex()).into()
                ));
            }
        }

        // Set up group configuration
        let admins = vec![self.keys.public_key()];  // Creator is admin, can add participants as admins later
        let relay_url = RelayUrl::parse(&self.relay_url)
            .map_err(|e| DialogError::General(format!("Invalid relay URL: {}", e).into()))?;
        
        let config = NostrGroupConfigData::new(
            name.to_string(),
            String::new(),  // Empty description for now
            None,           // No picture
            None,           // No pinned messages
            vec![relay_url],
        );

        // Create the group
        let group_create_result = nostr_mls
            .create_group(
                &self.keys.public_key(),
                key_package_events,
                admins,
                config,
            )
            .map_err(|e| DialogError::General(format!("Failed to create group: {}", e).into()))?;

        // Send welcome messages to participants
        // Each welcome rumor corresponds to a specific participant in the same order
        if group_create_result.welcome_rumors.len() != participants.len() {
            return Err(DialogError::General(
                format!("Welcome rumor count mismatch: {} rumors for {} participants", 
                    group_create_result.welcome_rumors.len(), 
                    participants.len()
                ).into()
            ));
        }
        
        for (i, rumor) in group_create_result.welcome_rumors.into_iter().enumerate() {
            let participant = &participants[i];
            let gift_wrap_event = EventBuilder::gift_wrap(&self.keys, participant, rumor, None)
                .await
                .map_err(|e| DialogError::General(format!("Failed to create gift wrap for {}: {}", participant.to_hex(), e).into()))?;
            
            client
                .send_event(&gift_wrap_event)
                .await
                .map_err(|e| DialogError::General(format!("Failed to send welcome to {}: {}", participant.to_hex(), e).into()))?;
        }

        // Return the group ID as hex string
        Ok(hex::encode(group_create_result.group.mls_group_id.as_slice()))
    }

    async fn add_contact(&self, pubkey: &str) -> Result<()> {
        // Validate input is not empty
        if pubkey.trim().is_empty() {
            return Err(DialogError::General("Pubkey cannot be empty".into()));
        }

        let pubkey = pubkey.trim();

        // Parse the pubkey string - could be bech32 (npub1...) or hex
        let public_key = if pubkey.starts_with("npub1") {
            // Parse bech32 format
            PublicKey::from_bech32(pubkey)
                .map_err(|e| DialogError::General(format!("Invalid bech32 pubkey: {}", e).into()))?
        } else {
            // Try to parse as hex
            PublicKey::from_hex(pubkey)
                .map_err(|e| DialogError::General(format!("Invalid hex pubkey: {}", e).into()))?
        };

        // Check if we're trying to add ourselves
        if public_key == self.keys.public_key() {
            return Err(DialogError::General("Cannot add yourself as a contact".into()));
        }

        // Check if contact already exists
        {
            let contacts = self.contacts.read().await;
            if contacts.contains_key(&public_key) {
                return Err(DialogError::General("Contact already exists".into()));
            }
        }

        // Check if we're connected before trying to load profile
        let connection_status = {
            let status = self.connection_status.read().await;
            *status
        };
        
        let name = if connection_status != ConnectionStatus::Connected {
            // Not connected, use truncated pubkey (warning handled at UI layer)
            if pubkey.starts_with("npub1") {
                format!("{}... (offline)", &pubkey[0..12])
            } else {
                format!("{}... (offline)", &pubkey[0..8])
            }
        } else {
            // Try to load the profile for this contact to get their real name
            match self.load_profile(&public_key).await {
                Ok(Some(profile)) => {
                    // Use the best available name from the profile
                    if let Some(display_name) = profile.display_name() {
                        display_name.to_string()
                    } else {
                        // Profile exists but no name, use truncated pubkey
                        if pubkey.starts_with("npub1") {
                            format!("{}... (no name)", &pubkey[0..12])
                        } else {
                            format!("{}... (no name)", &pubkey[0..8])
                        }
                    }
                },
                Ok(None) => {
                    // No profile found on relay
                    if pubkey.starts_with("npub1") {
                        format!("{}... (no profile)", &pubkey[0..12])
                    } else {
                        format!("{}... (no profile)", &pubkey[0..8])
                    }
                },
                Err(_) => {
                    // Error loading profile, use truncated pubkey
                    if pubkey.starts_with("npub1") {
                        format!("{}... (load error)", &pubkey[0..12])
                    } else {
                        format!("{}... (load error)", &pubkey[0..8])
                    }
                }
            }
        };

        let contact = Contact {
            name,
            pubkey: public_key,
            online: false, // Default to offline since we don't have presence info yet
        };

        // Store the contact in our runtime storage
        let mut contacts = self.contacts.write().await;
        contacts.insert(public_key, contact);

        Ok(())
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

    async fn get_own_pubkey(&self) -> Result<PublicKey> {
        Ok(self.keys.public_key())
    }

    async fn load_profile(&self, pubkey: &PublicKey) -> Result<Option<Profile>> {
        // Check cache first
        {
            let profiles = self.profiles.read().await;
            if let Some(profile) = profiles.get(pubkey) {
                return Ok(Some(profile.clone()));
            }
        }

        // Profile not in cache, fetch from relay
        let client = self.client.read().await;
        
        // Create a filter for Kind 0 (metadata) events from this pubkey
        let filter = Filter::new()
            .kind(Kind::Metadata)
            .author(*pubkey)
            .limit(1); // We only need the most recent one

        // Query the relay for the profile event
        let events = client
            .fetch_events(filter, std::time::Duration::from_secs(5))
            .await
            .map_err(|e| DialogError::General(format!("Failed to query profile: {}", e).into()))?;

        // Find the most recent metadata event
        if let Some(event) = events.first() {
            // Parse the JSON content as a Profile
            match serde_json::from_str::<Profile>(&event.content) {
                Ok(profile) => {
                    // Cache the profile for future use
                    {
                        let mut profiles = self.profiles.write().await;
                        profiles.insert(*pubkey, profile.clone());
                    }
                    Ok(Some(profile))
                },
                Err(_) => {
                    // If we can't parse the profile, don't fail completely
                    Ok(None)
                }
            }
        } else {
            // No profile found on relay
            Ok(None)
        }
    }

    async fn publish_profile(&self, profile: &Profile) -> Result<()> {
        let client = self.client.read().await;
        
        // Serialize the profile to JSON
        let content = serde_json::to_string(profile)
            .map_err(|e| DialogError::General(format!("Failed to serialize profile: {}", e).into()))?;

        // Create a Kind 0 (metadata) event
        let event_builder = EventBuilder::new(Kind::Metadata, content);
        let signed_event = client
            .sign_event_builder(event_builder)
            .await
            .map_err(|e| DialogError::General(format!("Failed to sign profile event: {}", e).into()))?;

        // Publish the event to the relay
        client
            .send_event(&signed_event)
            .await
            .map_err(|e| DialogError::General(format!("Failed to publish profile: {}", e).into()))?;

        Ok(())
    }

    async fn get_relay_url(&self) -> Result<String> {
        Ok(self.relay_url.clone())
    }

    async fn publish_key_packages(&self) -> Result<Vec<String>> {
        let client = self.client.read().await;
        let nostr_mls = self.nostr_mls.read().await;

        // Ensure we're connected
        let status = self.connection_status.read().await;
        if *status != ConnectionStatus::Connected {
            return Err(DialogError::General("Not connected to relay".into()));
        }

        // EPHEMERAL MODE: We publish fresh key packages on every startup
        // because we use memory storage and lose HPKE private keys on restart.
        // This means:
        // - Old key packages on relay become "orphaned" (we can't decrypt welcomes to them)
        // - We should ideally delete old packages, but Nostr doesn't guarantee deletion
        // - For now, we just publish fresh ones and document the event IDs for observability
        
        let mut event_ids = Vec::new();
        
        // Create multiple key packages (typically 10-20 for redundancy)
        // MLS best practice: publish multiple to avoid race conditions
        let num_packages = 5; // Reduced for testing, increase for production
        
        for _ in 0..num_packages {
            // Create a key package for the event
            let relay_url = RelayUrl::parse(&self.relay_url)
                .map_err(|e| DialogError::General(format!("Invalid relay URL: {}", e).into()))?;
            let relay_urls = vec![relay_url];
            let (key_package_encoded, tags) = nostr_mls
                .create_key_package_for_event(&self.keys.public_key(), relay_urls)?;

            // Build the key package event
            let key_package_event = EventBuilder::new(Kind::MlsKeyPackage, key_package_encoded)
                .tags(tags)
                .sign_with_keys(&self.keys)
                .map_err(|e| DialogError::General(format!("Failed to sign key package: {}", e).into()))?;

            // Publish the key package event
            let event_id = client
                .send_event(&key_package_event)
                .await
                .map_err(|e| DialogError::General(format!("Failed to publish key package: {}", e).into()))?;
            
            event_ids.push(event_id.to_hex());
        }

        Ok(event_ids)
    }

    async fn list_pending_invites(&self) -> Result<InviteListResult> {
        let client = self.client.read().await;
        let nostr_mls = self.nostr_mls.read().await;

        // Ensure we're connected
        let status = self.connection_status.read().await;
        if *status != ConnectionStatus::Connected {
            return Err(DialogError::General("Not connected to relay".into()));
        }

        // Fetch gift-wrapped events for this user
        let filter = Filter::new()
            .kind(Kind::GiftWrap)
            .pubkey(self.keys.public_key());
        
        let events = client
            .fetch_events(filter, std::time::Duration::from_secs(5))
            .await
            .map_err(|e| DialogError::General(format!("Failed to fetch gift wraps: {}", e).into()))?;

        // Collect processing errors to return to UI
        let mut processing_errors = Vec::new();

        // Process gift-wrapped events to extract welcome messages
        for event in events {
            // Try to extract rumor from gift wrap using NIP-59
            match nip59::extract_rumor(&self.keys, &event).await {
                Ok(unwrapped_gift) => {
                    // Process the welcome rumor
                    if let Err(e) = nostr_mls.process_welcome(&event.id, &unwrapped_gift.rumor) {
                        // Collect error for UI display
                        processing_errors.push(format!(
                            "⚠️  Failed to process welcome from {}: {}", 
                            unwrapped_gift.sender.to_hex()[0..16].to_string(),
                            e
                        ));
                    }
                }
                Err(e) => {
                    // Error unwrapping gift wrap - might not be for us
                    processing_errors.push(format!(
                        "⚠️  Failed to unwrap gift from event {}: {}",
                        event.id.to_hex()[0..16].to_string(),
                        e
                    ));
                }
            }
        }

        // Get pending welcomes from storage
        let pending_welcomes = nostr_mls.get_pending_welcomes()?;
        
        // Convert to our PendingInvite type
        let invites = pending_welcomes.into_iter().map(|welcome| {
            PendingInvite {
                group_id: welcome.mls_group_id,
                group_name: welcome.group_name,
                inviter: None, // TODO: Extract inviter from welcome data if available
                member_count: welcome.member_count as usize,
                timestamp: chrono::Utc::now().timestamp(), // TODO: Get actual timestamp from event
            }
        }).collect();

        Ok(InviteListResult {
            invites,
            processing_errors,
        })
    }

    async fn accept_invite(&self, group_id: &str) -> Result<()> {
        let nostr_mls = self.nostr_mls.read().await;

        // Parse the group ID from hex string
        let group_id_bytes = hex::decode(group_id)
            .map_err(|e| DialogError::General(format!("Invalid group ID: {}", e).into()))?;
        let group_id = GroupId::from_slice(&group_id_bytes);

        // Get pending welcomes
        let pending_welcomes = nostr_mls.get_pending_welcomes()?;
        
        // Find the matching welcome
        if let Some(welcome) = pending_welcomes.iter().find(|w| w.mls_group_id == group_id) {
            nostr_mls.accept_welcome(welcome)?;
            Ok(())
        } else {
            Err(DialogError::General(format!("No pending invite found for group ID: {}", hex::encode(group_id.as_slice())).into()))
        }
    }

    async fn fetch_and_process_group_events(&self, group_id: &GroupId) -> Result<()> {
        let client = self.client.read().await;
        let nostr_mls = self.nostr_mls.read().await;

        // Get the stored group to find its Nostr group ID
        let groups = nostr_mls.get_groups()?;
        let stored_group = groups
            .iter()
            .find(|g| &g.mls_group_id == group_id)
            .ok_or_else(|| DialogError::General("Group not found".into()))?;

        // Filter for MLS group messages tagged with this group's Nostr Group ID
        let nostr_group_id_hex = hex::encode(&stored_group.nostr_group_id);
        let filter = Filter::new()
            .kind(Kind::MlsGroupMessage)
            .custom_tag(nostr_sdk::SingleLetterTag::lowercase(nostr_sdk::Alphabet::H), nostr_group_id_hex);

        // Fetch events from relay
        let events = client
            .fetch_events(filter, std::time::Duration::from_secs(5))
            .await
            .map_err(|e| DialogError::General(format!("Failed to fetch group events: {}", e).into()))?;

        // Process each event to update MLS state
        for event in events {
            if let Err(_) = nostr_mls.process_message(&event) {
                // Silently ignore processing errors - the event might be malformed
                // or for a different epoch/state
            }
        }

        Ok(())
    }

    async fn subscribe_to_groups(&self, ui_sender: mpsc::Sender<UiUpdate>) -> Result<()> {
        let client = self.client.read().await;
        let nostr_mls = self.nostr_mls.read().await;

        // Get all groups to subscribe to
        let groups = nostr_mls.get_groups()?;
        let mut filters = Vec::new();

        // Create filters for all group messages
        for group in &groups {
            let filter = Filter::new()
                .kind(Kind::MlsGroupMessage)
                .custom_tag(
                    SingleLetterTag::lowercase(Alphabet::H),
                    hex::encode(&group.nostr_group_id),
                );
            filters.push(filter);
        }

        // Also subscribe to gift wraps for invites
        let giftwrap_filter = Filter::new()
            .kind(Kind::GiftWrap)
            .pubkey(self.keys.public_key());
        filters.push(giftwrap_filter);

        // Create subscription
        let subscription_id = SubscriptionId::new("dialog_messages");
        for filter in filters {
            client
                .subscribe_with_id(subscription_id.clone(), filter, None)
                .await
                .map_err(|e| DialogError::General(format!("Failed to create subscription: {}", e).into()))?;
        }

        // Spawn a task to handle incoming events
        let client_clone = self.client.clone();
        let nostr_mls_clone = self.nostr_mls.clone();
        let keys_clone = self.keys.clone();
        let message_cache_clone = self.message_cache.clone();
        
        tokio::spawn(async move {
            loop {
                // Handle events from subscription
                let mut notifications = client_clone.read().await.notifications();
                
                while let Ok(notification) = notifications.recv().await {
                    if let RelayPoolNotification::Event { subscription_id: sub_id, event, .. } = notification {
                        if sub_id == subscription_id {
                            // Process the event based on its kind
                            match event.kind {
                                Kind::MlsGroupMessage => {
                                    // Process MLS message
                                    let nostr_mls = nostr_mls_clone.read().await;
                                    if let Ok(_) = nostr_mls.process_message(&event) {
                                        // Extract group ID from tags
                                        if let Some(tag) = event.tags.iter().find(|t| {
                                            t.as_slice().len() >= 2 && t.as_slice()[0] == "h"
                                        }) {
                                            if let Ok(nostr_group_id_bytes) = hex::decode(&tag.as_slice()[1]) {
                                                // Find the matching group
                                                if let Ok(groups) = nostr_mls.get_groups() {
                                                    if let Some(group) = groups.iter().find(|g| {
                                                        g.nostr_group_id.as_slice() == nostr_group_id_bytes.as_slice()
                                                    }) {
                                                        // Get the decrypted message
                                                        if let Ok(messages) = nostr_mls.get_messages(&group.mls_group_id) {
                                                            // Find the latest message
                                                            if let Some(msg) = messages.last() {
                                                                let message = Message {
                                                                    sender: msg.pubkey,
                                                                    content: msg.content.clone(),
                                                                    timestamp: event.created_at.as_u64() as i64,
                                                                    id: Some(event.id.to_hex()),
                                                                };
                                                                
                                                                // Cache the message
                                                                let mut cache = message_cache_clone.write().await;
                                                                let cached_messages = cache.entry(group.mls_group_id.clone()).or_insert_with(Vec::new);
                                                                cached_messages.push(CachedMessage {
                                                                    message: message.clone(),
                                                                    event_id: event.id,
                                                                });
                                                                
                                                                // Send UI update
                                                                let _ = ui_sender.send(UiUpdate::NewMessage {
                                                                    group_id: group.mls_group_id.clone(),
                                                                    message,
                                                                }).await;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Kind::GiftWrap => {
                                    // Process potential invite
                                    if let Ok(unwrapped_gift) = nip59::extract_rumor(&keys_clone, &event).await {
                                        let nostr_mls = nostr_mls_clone.read().await;
                                        if let Ok(_) = nostr_mls.process_welcome(&event.id, &unwrapped_gift.rumor) {
                                            // Get the new pending welcome
                                            if let Ok(welcomes) = nostr_mls.get_pending_welcomes() {
                                                if let Some(welcome) = welcomes.last() {
                                                    let invite = PendingInvite {
                                                        group_id: welcome.mls_group_id.clone(),
                                                        group_name: welcome.group_name.clone(),
                                                        inviter: Some(unwrapped_gift.sender),
                                                        member_count: welcome.member_count as usize,
                                                        timestamp: event.created_at.as_u64() as i64,
                                                    };
                                                    
                                                    // Send UI update
                                                    let _ = ui_sender.send(UiUpdate::NewInvite(invite)).await;
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn fetch_messages(&self, group_id: &GroupId) -> Result<MessageFetchResult> {
        let client = self.client.read().await;
        let nostr_mls = self.nostr_mls.read().await;

        // Collect processing errors
        let mut processing_errors = Vec::new();

        // First fetch and process any new group events to ensure we're up to date
        if let Err(e) = self.fetch_and_process_group_events(group_id).await {
            processing_errors.push(format!("⚠️  Failed to sync group state: {}", e));
        }

        // Get the stored group to find its Nostr group ID
        let groups = nostr_mls.get_groups()?;
        let stored_group = groups
            .iter()
            .find(|g| &g.mls_group_id == group_id)
            .ok_or_else(|| DialogError::General("Group not found".into()))?;

        // Filter for MLS group messages tagged with this group's Nostr Group ID
        let nostr_group_id_hex = hex::encode(&stored_group.nostr_group_id);
        let filter = Filter::new()
            .kind(Kind::MlsGroupMessage)
            .custom_tag(nostr_sdk::SingleLetterTag::lowercase(nostr_sdk::Alphabet::H), nostr_group_id_hex);

        // Fetch message events from relay
        let events = client
            .fetch_events(filter, std::time::Duration::from_secs(5))
            .await
            .map_err(|e| DialogError::General(format!("Failed to fetch messages: {}", e).into()))?;

        // Check if we have cached messages for this group
        let mut message_cache = self.message_cache.write().await;
        let cached_messages = message_cache.entry(group_id.clone()).or_insert_with(Vec::new);
        
        // Track which events we've already processed
        let processed_event_ids: std::collections::HashSet<_> = cached_messages
            .iter()
            .map(|cm| cm.event_id)
            .collect();

        // Process each event to extract messages
        for event in events {
            // Skip if we've already processed this event
            if processed_event_ids.contains(&event.id) {
                continue;
            }

            // Process the message to decrypt it
            if let Err(e) = nostr_mls.process_message(&event) {
                // Collect error for UI display
                processing_errors.push(format!(
                    "⚠️  Failed to process message {}: {}",
                    event.id.to_hex()[0..16].to_string(),
                    e
                ));
                continue;
            }

            // Get the timestamp from the event
            let timestamp = event.created_at.as_u64() as i64;
            
            // Try to get the decrypted message from storage
            let stored_messages = nostr_mls.get_messages(&stored_group.mls_group_id)?;
            
            // Find the message that corresponds to this event (by matching content/timestamp)
            // This is a bit hacky but necessary since nostr-mls doesn't expose event IDs
            if let Some(msg) = stored_messages.iter().find(|m| {
                // Find a message that we haven't cached yet
                !cached_messages.iter().any(|cm| 
                    cm.message.sender == m.pubkey && 
                    cm.message.content == m.content
                )
            }) {
                let message = Message {
                    sender: msg.pubkey,
                    content: msg.content.clone(),
                    timestamp,
                    id: Some(event.id.to_hex()),
                };
                
                cached_messages.push(CachedMessage {
                    message: message.clone(),
                    event_id: event.id,
                });
            }
        }

        // Update last sync time
        {
            let mut last_sync = self.last_sync.write().await;
            last_sync.insert(group_id.clone(), chrono::Utc::now().timestamp());
        }

        // Return all cached messages for this group
        let mut messages: Vec<Message> = cached_messages
            .iter()
            .map(|cm| cm.message.clone())
            .collect();

        // Sort messages by timestamp (oldest first)
        messages.sort_by_key(|m| m.timestamp);

        Ok(MessageFetchResult {
            messages,
            processing_errors,
        })
    }
}