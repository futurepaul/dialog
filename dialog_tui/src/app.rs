use std::time::Duration;
use std::panic;
use std::fs::OpenOptions;
use std::io::Write;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use tokio::sync::mpsc;
use tokio::time;
use nostr_sdk::{Keys, Client, Filter, Kind, EventBuilder, RelayUrl, prelude::*, nips::nip59};
use nostr_mls::groups::NostrGroupConfigData;
use nostr_mls::messages::MessageProcessingResult;

use crate::error::{DialogTuiError, Result};
use crate::model::{AppState, Msg, Cmd};
use crate::storage::PerPubkeyStorage;
use crate::update;
use crate::ui;

pub struct App {
    state: AppState,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    event_tx: mpsc::UnboundedSender<Msg>,
    event_rx: mpsc::UnboundedReceiver<Msg>,
    storage: PerPubkeyStorage,
    keys: Option<Keys>,
    client: Option<Client>,
    log_file: Option<std::fs::File>,
}

impl App {
    pub async fn new() -> Result<Self> {
        // Setup terminal with better error handling
        enable_raw_mode().map_err(|e| {
            DialogTuiError::Ui { 
                message: format!("Failed to enable raw mode: {}. Make sure you're running in a proper terminal.", e) 
            }
        })?;
        
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| {
            DialogTuiError::Ui { 
                message: format!("Failed to enter alternate screen: {}. Make sure you're running in a proper terminal.", e) 
            }
        })?;
        
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).map_err(|e| {
            DialogTuiError::Ui { 
                message: format!("Failed to create terminal: {}. Make sure you're running in a proper terminal.", e) 
            }
        })?;

        // Setup event channels
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Initialize storage
        let storage = PerPubkeyStorage::new()?;
        
        // Initialize log file
        let log_file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open("dialog_tui_debug.log") {
            Ok(file) => {
                println!("Debug logs will be written to: dialog_tui_debug.log");
                Some(file)
            }
            Err(e) => {
                eprintln!("Warning: Could not create log file: {}", e);
                None
            }
        };

        Ok(Self {
            state: AppState::default(),
            terminal,
            event_tx,
            event_rx,
            storage,
            keys: None,
            client: None,
            log_file,
        })
    }

    pub async fn init_with_key(&mut self, private_key: Option<String>) -> Result<()> {
        let sk_hex = private_key.ok_or_else(|| DialogTuiError::InvalidInput {
            message: "Private key is required".to_string()
        })?;

        let keys = Keys::parse(&sk_hex)?;
        
        // Initialize storage for this pubkey
        self.storage.init_for_pubkey(&keys).await?;
        
        // Log session start
        self.add_log_entry("INFO", &format!("=== NEW SESSION STARTED === Key: {}", keys.public_key()));
        
        // Load existing conversations and contacts
        let conversations = self.storage.load_conversations().await?;
        self.state.conversations = conversations;
        
        let contacts = self.storage.load_contacts().await?;
        self.state.contacts = contacts;

        // Setup nostr client
        let client = Client::new(keys.clone());
        client.add_relay("ws://localhost:8080").await?;
        
        self.keys = Some(keys);
        self.client = Some(client);
        
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        // Start event listeners
        self.start_input_handler();
        self.start_ticker();
        self.start_message_fetcher();

        // Connect to relay
        if let Some(client) = &self.client {
            client.connect().await;
            self.state.connection_status = crate::model::state::ConnectionStatus::Connected;
            
            // Fetch pending invites on startup
            if let Err(e) = self.execute_command(Cmd::FetchPendingInvites).await {
                tracing::warn!("Failed to fetch pending invites on startup: {}", e);
            }
            
            // Add startup log entry
            self.add_log_entry("INFO", "Dialog TUI started successfully");
        }

        loop {
            // Draw UI
            self.terminal.draw(|frame| ui::render(&self.state, frame))?;

            // Handle events
            match self.event_rx.recv().await {
                Some(msg) => {
                    if matches!(msg, Msg::Quit) {
                        break;
                    }

                    let (new_state, cmd) = update::update(&self.state, msg);
                    self.state = new_state;
                    self.execute_command(cmd).await?;
                }
                None => break,
            }
        }

        Ok(())
    }

    fn start_input_handler(&self) {
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            loop {
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            if tx.send(Msg::KeyPress(key)).is_err() {
                                break;
                            }
                        }
                        Ok(Event::Resize(w, h)) => {
                            if tx.send(Msg::TerminalResized(w, h)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }

    fn start_ticker(&self) {
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                if tx.send(Msg::Tick).is_err() {
                    break;
                }
                // Also send toast expiration check
                if tx.send(Msg::ExpireToasts).is_err() {
                    break;
                }
            }
        });
    }

    fn start_message_fetcher(&self) {
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(5)); // Fetch messages every 5 seconds
            loop {
                interval.tick().await;
                if tx.send(Msg::FetchNewMessages).is_err() {
                    break;
                }
            }
        });
    }

    async fn execute_command(&mut self, cmd: Cmd) -> Result<()> {
        match cmd {
            Cmd::None => Ok(()),
            Cmd::Batch(cmds) => {
                for cmd in cmds {
                    Box::pin(self.execute_command(cmd)).await?;
                }
                Ok(())
            }
            Cmd::Exit => {
                self.event_tx.send(Msg::Quit)
                    .map_err(|e| DialogTuiError::Send(e.to_string()))?;
                Ok(())
            }
            Cmd::SaveMessage(message) => {
                self.storage.save_message(&message).await?;
                Ok(())
            }
            Cmd::LoadConversationHistory(conversation_id) => {
                let messages = self.storage.load_messages(&conversation_id).await?;
                self.state.messages.insert(conversation_id, messages);
                Ok(())
            }
            Cmd::SendMessage(content, conversation_id) => {
                tracing::info!("SendMessage command initiated for conversation: {}", conversation_id);
                self.add_log_entry("INFO", &format!("Attempting to send message to conversation: {}", conversation_id));
                
                if let (Some(client), Some(keys)) = (&self.client, &self.keys) {
                    tracing::debug!("Client and keys available for sending message");
                    
                    // Extract conversation details before borrowing storage mutably
                    let (conversation_name, group_id) = if let Some(conversation) = self.state.conversations.get(&conversation_id) {
                        tracing::debug!("Found conversation: {}", conversation.name);
                        
                        if let Some(group_id) = &conversation.group_id {
                            tracing::debug!("Conversation has group_id: {}", hex::encode(group_id.as_slice()));
                            (conversation.name.clone(), Some(group_id.clone()))
                        } else {
                            (conversation.name.clone(), None)
                        }
                    } else {
                        (String::new(), None)
                    };
                    
                    if conversation_name.is_empty() {
                        let error_msg = format!("Conversation not found: {}", conversation_id);
                        tracing::error!("{}", error_msg);
                        self.add_log_entry("ERROR", &error_msg);
                    } else if let Some(group_id) = group_id {
                        if let Some(nostr_mls) = self.storage.get_nostr_mls_mut() {
                            tracing::info!("Sending message to group: {}", conversation_id);
                            let mut log_messages = Vec::new(); // Collect log messages to add later
                                
                                // First, sync the group state by fetching any pending messages
                                let groups = nostr_mls.get_groups()
                                    .map_err(|e| DialogTuiError::Network { 
                                        message: format!("Failed to get groups: {}", e) 
                                    })?;
                                    
                                log_messages.push(("DEBUG".to_string(), format!("Found {} total MLS groups in storage", groups.len())));
                                for (i, group) in groups.iter().enumerate() {
                                    log_messages.push(("DEBUG".to_string(), format!("Group {}: id={}, epoch={}", i, hex::encode(group.mls_group_id.as_slice()), group.epoch)));
                                }
                                
                                if let Some(stored_group) = groups.iter().find(|g| g.mls_group_id == group_id) {
                                    tracing::debug!("Found stored group at epoch: {}", stored_group.epoch);
                                    log_messages.push(("DEBUG".to_string(), format!("Found stored group at epoch: {}", stored_group.epoch)));
                                    let nostr_group_id_hex = hex::encode(&stored_group.nostr_group_id);
                                    
                                    // Fetch and process any pending MLS events
                                    let filter = Filter::new()
                                        .kind(Kind::MlsGroupMessage)
                                        .custom_tag(
                                            nostr_sdk::SingleLetterTag::lowercase(nostr_sdk::Alphabet::H), 
                                            nostr_group_id_hex
                                        );
                                    
                                    let events = client.fetch_events(filter, Duration::from_secs(10)).await
                                        .map_err(|e| DialogTuiError::Network { 
                                            message: format!("Failed to fetch MLS events: {}", e) 
                                        })?;
                                    
                                    tracing::info!("Found {} MLS group events before sending", events.len());
                                    for event in events {
                                        if let Err(e) = nostr_mls.process_message(&event) {
                                            tracing::warn!("Failed to process MLS event {}: {}", event.id, e);
                                        }
                                    }
                                    
                                    // Create the message as a TextNote rumor
                                    let rumor = EventBuilder::new(Kind::TextNote, content.clone())
                                        .build(keys.public_key());
                                    
                                    // Create MLS message
                                    tracing::debug!("Creating MLS message for group_id: {}", hex::encode(group_id.as_slice()));
                                    log_messages.push(("DEBUG".to_string(), format!("Creating MLS message for group_id: {}", hex::encode(group_id.as_slice()))));
                                    
                                    // DEEP DEBUG: Let's verify the group state before attempting to create message
                                    let groups_before_create = nostr_mls.get_groups()
                                        .map_err(|e| DialogTuiError::Network { 
                                            message: format!("Failed to get groups before create_message: {}", e) 
                                        })?;
                                    log_messages.push(("DEBUG".to_string(), format!("Groups available before create_message: {}", groups_before_create.len())));
                                    
                                    let target_group = groups_before_create.iter().find(|g| g.mls_group_id == group_id);
                                    if let Some(group) = target_group {
                                        log_messages.push(("DEBUG".to_string(), format!("Target group found - epoch: {}", group.epoch)));
                                        log_messages.push(("ERROR".to_string(), "CRITICAL BUG: MLS library has group metadata but create_message fails!".to_string()));
                                        log_messages.push(("ERROR".to_string(), "This indicates the group's cryptographic state is corrupted or missing".to_string()));
                                        log_messages.push(("ERROR".to_string(), "Possible solutions: 1) Re-fetch welcome message 2) Re-join group 3) Delete corrupted group".to_string()));
                                        log_messages.push(("ERROR".to_string(), "WORKAROUND: Use Power Tools > Reset All State to clear corrupted groups".to_string()));
                                        
                                        // Try to get more detailed group information if available
                                        log_messages.push(("DEBUG".to_string(), format!("Group details - name: {:?}, description: {:?}", group.name, group.description)));
                                        
                                        // For now, we'll continue to attempt create_message to get the exact error
                                    } else {
                                        log_messages.push(("ERROR".to_string(), "Target group NOT FOUND in get_groups() right before create_message!".to_string()));
                                        // Add log entry after dropping mutable borrow
                                        for (level, msg) in log_messages {
                                            self.add_log_entry(&level, &msg);
                                        }
                                        self.add_log_entry("ERROR", "CRITICAL: Group exists in conversation storage but not in MLS storage!");
                                        return Err(DialogTuiError::Network { 
                                            message: "Group storage inconsistency detected".to_string() 
                                        });
                                    }
                                    
                                    // Wrap in panic protection
                                    let message_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                                        nostr_mls.create_message(&group_id, rumor.clone())
                                    }));
                                    
                                    let message_event = match message_result {
                                        Ok(Ok(event)) => {
                                            log_messages.push(("INFO".to_string(), "Successfully created MLS message".to_string()));
                                            event
                                        }
                                        Ok(Err(e)) => {
                                            let error_msg = format!("Failed to create MLS message: {}", e);
                                            tracing::error!("{}", error_msg);
                                            // Add log entry after dropping mutable borrow
                                            for (level, msg) in log_messages {
                                                self.add_log_entry(&level, &msg);
                                            }
                                            self.add_log_entry("ERROR", &error_msg);
                                            return Err(DialogTuiError::Network { message: error_msg });
                                        }
                                        Err(panic_info) => {
                                            let error_msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                                                format!("MLS create_message panicked: {}", s)
                                            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                                                format!("MLS create_message panicked: {}", s)
                                            } else {
                                                "MLS create_message panicked with unknown error".to_string()
                                            };
                                            tracing::error!("{}", error_msg);
                                            // Add log entry after dropping mutable borrow
                                            for (level, msg) in log_messages {
                                                self.add_log_entry(&level, &msg);
                                            }
                                            self.add_log_entry("ERROR", &error_msg);
                                            return Err(DialogTuiError::Network { message: error_msg });
                                        }
                                    };
                                    
                                    // Send the message
                                    let output = client.send_event(&message_event).await
                                        .map_err(|e| DialogTuiError::Network { 
                                            message: format!("Failed to send message: {}", e) 
                                        })?;
                                    
                                    tracing::info!("Sent message: {}", output.id());
                                    
                                    // Process the message locally to maintain MLS state
                                    let message_id = output.id().to_string();
                                    let sender_pubkey = keys.public_key();
                                    
                                    if let Err(e) = nostr_mls.process_message(&message_event) {
                                        tracing::error!("Failed to process own message locally: {}", e);
                                        // Store error for later logging
                                        let error_msg = format!("Failed to process own message locally: {}", e);
                                        self.add_log_entry("ERROR", &error_msg);
                                    } else {
                                        tracing::debug!("Successfully processed own message locally");
                                    }
                                    
                                    // Add to local state immediately for UI feedback
                                    let chat_message = crate::model::ChatMessage {
                                        id: message_id.clone(),
                                        conversation_id: conversation_id.clone(),
                                        sender: sender_pubkey,
                                        content,
                                        timestamp: chrono::Utc::now(),
                                        is_own: true,
                                    };
                                    
                                    self.state.messages
                                        .entry(conversation_id.clone())
                                        .or_insert_with(Vec::new)
                                        .push(chat_message.clone());
                                    
                                    // Save to storage
                                    self.storage.save_message(&chat_message).await?;
                                    
                                    // Mark the message as processed
                                    self.state.processed_message_ids.insert(message_id);
                                    
                                    log_messages.push(("INFO".to_string(), format!("Successfully sent message to {}", conversation_name)));
                                } else {
                                    let error_msg = format!("Group not found in MLS storage for conversation: {}. Found {} total groups but none match group_id: {}", 
                                        conversation_id, groups.len(), hex::encode(group_id.as_slice()));
                                    tracing::error!("{}", error_msg);
                                    log_messages.push(("ERROR".to_string(), error_msg.clone()));
                                    
                                    // Try to recover by re-fetching welcome messages
                                    log_messages.push(("INFO".to_string(), "Attempting to recover by re-fetching welcome messages...".to_string()));
                                    
                                    // client and keys are already available from the outer scope
                                    let filter = Filter::new()
                                        .kind(Kind::GiftWrap)
                                        .pubkey(keys.public_key());
                                    
                                    if let Ok(events) = client.fetch_events(filter, Duration::from_secs(5)).await {
                                        log_messages.push(("DEBUG".to_string(), format!("Found {} gift wrap events during recovery", events.len())));
                                        
                                        for event in events {
                                            if let Ok(unwrapped_gift) = nip59::extract_rumor(keys, &event).await {
                                                if unwrapped_gift.rumor.kind == Kind::MlsWelcome {
                                                    log_messages.push(("DEBUG".to_string(), format!("Processing welcome message during recovery: {}", event.id)));
                                                    if let Err(e) = nostr_mls.process_welcome(&event.id, &unwrapped_gift.rumor) {
                                                        log_messages.push(("WARN".to_string(), format!("Failed to process welcome during recovery: {}", e)));
                                                    } else {
                                                        log_messages.push(("INFO".to_string(), "Successfully processed welcome message during recovery".to_string()));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            
                            // Add all collected log messages after dropping the mutable borrow
                            for (level, msg) in log_messages {
                                self.add_log_entry(&level, &msg);
                            }
                        } else {
                            let error_msg = "NostrMLS not initialized";
                            tracing::error!("{}", error_msg);
                            self.add_log_entry("ERROR", error_msg);
                        }
                    } else {
                        let error_msg = format!("Conversation {} has no group_id (not an MLS conversation)", conversation_id);
                        tracing::error!("{}", error_msg);
                        self.add_log_entry("ERROR", &error_msg);
                    }
                } else {
                    let error_msg = "Client or keys not available for sending message";
                    tracing::error!("{}", error_msg);
                    self.add_log_entry("ERROR", error_msg);
                }
                Ok(())
            }
            Cmd::ConnectWebSocket => {
                // TODO: Implement WebSocket connection
                Ok(())
            }
            Cmd::CreateMlsGroup(contact_id) => {
                tracing::info!("Starting CreateMlsGroup command for contact_id: {}", contact_id);
                self.add_log_entry("INFO", &format!("Starting CreateMlsGroup command for contact_id: {}", contact_id));
                if let Some(contact) = self.state.contacts.get(&contact_id) {
                    tracing::info!("Found contact: {}", contact.petname);
                    if let (Some(client), Some(keys)) = (&self.client, &self.keys) {
                        tracing::info!("Client and keys available, proceeding with group creation");
                        tracing::info!("Creating MLS group with contact: {}", contact.petname);
                        
                        // First, fetch the contact's key package
                        let filter = Filter::new()
                            .kind(Kind::MlsKeyPackage)
                            .author(contact.pubkey);
                        
                        let events = client
                            .fetch_events(filter, Duration::from_secs(10))
                            .await
                            .map_err(|e| DialogTuiError::Network { 
                                message: format!("Failed to fetch key packages: {}", e) 
                            })?;
                        
                        if let Some(key_package_event) = events.first() {
                            tracing::info!("Found key package for contact: {}", contact.petname);
                            
                            // Get mutable reference to nostr_mls
                            if let Some(nostr_mls) = self.storage.get_nostr_mls_mut() {
                                // Parse and validate the key package
                                let _counterparty_key_package = nostr_mls
                                    .parse_key_package(key_package_event)
                                    .map_err(|e| DialogTuiError::Network { 
                                        message: format!("Failed to parse key package: {}", e) 
                                    })?;
                                
                                // Create group configuration
                                let admins = vec![keys.public_key(), contact.pubkey];
                                let relay_url = RelayUrl::parse("ws://localhost:8080")
                                    .map_err(|e| DialogTuiError::Network { 
                                        message: format!("Invalid relay URL: {}", e) 
                                    })?;
                                
                                let config = NostrGroupConfigData::new(
                                    contact.petname.clone(),
                                    format!("Chat with {}", contact.petname),
                                    None,
                                    None,
                                    vec![relay_url],
                                );
                                
                                // Create the MLS group
                                let group_create_result = nostr_mls
                                    .create_group(
                                        &keys.public_key(),
                                        vec![key_package_event.clone()],
                                        admins,
                                        config,
                                    )
                                    .map_err(|e| DialogTuiError::Network { 
                                        message: format!("Failed to create MLS group: {}", e) 
                                    })?;
                                
                                let mls_group_id = group_create_result.group.mls_group_id.clone();
                                let _nostr_group_id = group_create_result.group.nostr_group_id.clone();
                                
                                tracing::info!(
                                    "Created MLS group: {} (epoch: {})", 
                                    hex::encode(mls_group_id.as_slice()),
                                    group_create_result.group.epoch
                                );
                                
                                // Send welcome messages
                                for rumor in group_create_result.welcome_rumors {
                                    let gift_wrap_event = EventBuilder::gift_wrap(
                                        keys,
                                        &contact.pubkey,
                                        rumor,
                                        None
                                    )
                                    .await
                                    .map_err(|e| DialogTuiError::Network { 
                                        message: format!("Failed to create gift wrap: {}", e) 
                                    })?;
                                    
                                    client.send_event(&gift_wrap_event).await
                                        .map_err(|e| DialogTuiError::Network { 
                                            message: format!("Failed to send welcome message: {}", e) 
                                        })?;
                                    
                                    tracing::info!("Sent welcome message to {}", contact.petname);
                                }
                                
                                // Create conversation with MLS group ID
                                let conversation_id = hex::encode(mls_group_id.as_slice());
                                let conversation = crate::model::state::Conversation {
                                    id: conversation_id.clone(),
                                    group_id: Some(mls_group_id.clone()),
                                    name: contact.petname.clone(),
                                    participants: vec![contact.pubkey],
                                    last_message_time: None,
                                    unread_count: 0,
                                };
                                
                                // Save and add to state
                                self.storage.save_conversation(&conversation).await?;
                                self.state.conversations.insert(conversation_id.clone(), conversation);
                                self.state.selected_conversation = Some(conversation_id);
                                self.state.active_pane = crate::model::ActivePane::Chat;
                                
                                tracing::info!("Created conversation with MLS group ID: {}", hex::encode(mls_group_id.as_slice()));
                            } else {
                                tracing::error!("NostrMls not initialized");
                            }
                        } else {
                            tracing::warn!("No key package found for contact: {}. They need to publish a keypackage first.", contact.petname);
                            self.add_log_entry("WARN", &format!("No key package found for contact: {}. They need to publish a keypackage first.", contact.petname));
                        }
                    } else {
                        tracing::error!("Client or keys not available - client: {}, keys: {}", 
                            self.client.is_some(), self.keys.is_some());
                        self.add_log_entry("ERROR", "Client or keys not available");
                    }
                } else {
                    tracing::error!("Contact not found for contact_id: {}", contact_id);
                    self.add_log_entry("ERROR", &format!("Contact not found for contact_id: {}", contact_id));
                }
                Ok(())
            }
            Cmd::SaveContact(contact) => {
                self.storage.save_contact(&contact).await?;
                self.event_tx.send(Msg::ContactAdded(contact))
                    .map_err(|e| DialogTuiError::Send(e.to_string()))?;
                Ok(())
            }
            Cmd::LoadContacts => {
                let contacts = self.storage.load_contacts().await?;
                self.state.contacts = contacts;
                Ok(())
            }
            Cmd::SaveConversation(conversation) => {
                self.storage.save_conversation(&conversation).await?;
                Ok(())
            }
            Cmd::LoadConversations => {
                let conversations = self.storage.load_conversations().await?;
                self.state.conversations = conversations;
                Ok(())
            }
            Cmd::PublishKeypackageToRelay => {
                if let (Some(client), Some(keys)) = (&self.client, &self.keys) {
                    if let Some(nostr_mls) = self.storage.get_nostr_mls() {
                        tracing::info!("Publishing keypackage for {}", keys.public_key().to_hex());
                        
                        let relay_url = RelayUrl::parse("ws://localhost:8080")
                            .map_err(|e| DialogTuiError::Network { 
                                message: format!("Invalid relay URL: {}", e) 
                            })?;
                        
                        let (key_package_encoded, tags) = nostr_mls
                            .create_key_package_for_event(&keys.public_key(), [relay_url])
                            .map_err(|e| DialogTuiError::Network { 
                                message: format!("Failed to create key package: {}", e) 
                            })?;
                        
                        let key_package_event = EventBuilder::new(Kind::MlsKeyPackage, key_package_encoded)
                            .tags(tags)
                            .sign_with_keys(keys)
                            .map_err(|e| DialogTuiError::Network { 
                                message: format!("Failed to sign key package event: {}", e) 
                            })?;
                        
                        let output = client.send_event(&key_package_event).await
                            .map_err(|e| DialogTuiError::Network { 
                                message: format!("Failed to publish key package: {}", e) 
                            })?;
                        
                        tracing::info!("Published keypackage: {}", output.id());
                        self.add_log_entry("INFO", &format!("Published keypackage: {}", output.id()));
                    }
                }
                Ok(())
            }
            Cmd::FetchPendingInvites => {
                if let (Some(client), Some(keys)) = (&self.client, &self.keys) {
                    tracing::info!("Fetching pending invites for {}", keys.public_key().to_hex());
                    
                    // Fetch gift-wrapped events for this user
                    let filter = Filter::new()
                        .kind(Kind::GiftWrap)
                        .pubkey(keys.public_key());
                    
                    let events = client.fetch_events(filter, Duration::from_secs(10)).await
                        .map_err(|e| DialogTuiError::Network { 
                            message: format!("Failed to fetch gift wrap events: {}", e) 
                        })?;
                    
                    tracing::info!("Found {} gift wrap events", events.len());
                    
                    for event in events {
                        // Try to decrypt the gift wrap using NIP-59
                        if let Ok(unwrapped_gift) = nip59::extract_rumor(keys, &event).await {
                            let rumor = unwrapped_gift.rumor;
                            tracing::info!("Unwrapped gift wrap event: {}", rumor.kind);
                            
                            // Check if this is a welcome message
                            if rumor.kind == Kind::MlsWelcome {
                                // Extract invite information
                                let from_pubkey = unwrapped_gift.sender;
                                let petname = self.state.contacts.values()
                                    .find(|c| c.pubkey == from_pubkey)
                                    .map(|c| c.petname.clone())
                                    .unwrap_or_else(|| "Unknown".to_string());
                                
                                let invite = crate::model::state::PendingInvite {
                                    id: event.id.to_string(),
                                    from: from_pubkey,
                                    petname,
                                    group_name: None, // Could extract from tags if available
                                    event_id: event.id.to_string(),
                                    timestamp: chrono::DateTime::from_timestamp(event.created_at.as_u64() as i64, 0)
                                        .unwrap_or_else(|| chrono::Utc::now()),
                                };
                                
                                // Add to pending invites if not already there
                                if !self.state.pending_invites.iter().any(|i| i.event_id == event.id.to_string()) {
                                    self.state.pending_invites.push(invite);
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            Cmd::AcceptPendingInvite(invite_index) => {
                if let (Some(client), Some(keys)) = (&self.client, &self.keys) {
                    if let Some(invite) = self.state.pending_invites.get(invite_index) {
                        tracing::info!("Accepting invite from {}", invite.petname);
                        
                        // Fetch the original gift wrap event
                        let filter = Filter::new()
                            .kind(Kind::GiftWrap)
                            .id(nostr_sdk::EventId::from_hex(&invite.event_id).map_err(|e| {
                                DialogTuiError::Network { 
                                    message: format!("Invalid event ID: {}", e) 
                                }
                            })?);
                        
                        let events = client.fetch_events(filter, Duration::from_secs(10)).await
                            .map_err(|e| DialogTuiError::Network { 
                                message: format!("Failed to fetch invite event: {}", e) 
                            })?;
                        
                        if let Some(event) = events.first() {
                            if let Ok(unwrapped_gift) = nip59::extract_rumor(keys, event).await {
                                // Store values we need after mutable borrow
                                let invite_petname = invite.petname.clone();
                                let invite_from = invite.from;
                                
                                // Collect messages to save outside the mutable borrow
                                let mut messages_to_save = Vec::new();
                                let mut conversation_created = false;
                                let mut log_messages = Vec::new();
                                
                                if let Some(nostr_mls) = self.storage.get_nostr_mls_mut() {
                                    // Process the welcome message
                                    match nostr_mls.process_welcome(&event.id, &unwrapped_gift.rumor) {
                                        Ok(_) => {
                                            tracing::info!("Successfully processed welcome from {}", invite_petname);
                                            log_messages.push(("INFO".to_string(), format!("Processed welcome message from {}", invite_petname)));
                                            
                                            // Get the groups and create conversation data
                                            let conversation_data = if let Ok(groups) = nostr_mls.get_groups() {
                                                if let Some(latest_group) = groups.last() {
                                                    let conversation_id = hex::encode(latest_group.mls_group_id.as_slice());
                                                    tracing::info!("Joined group at epoch: {}", latest_group.epoch);
                                                    log_messages.push(("DEBUG".to_string(), format!("New group created: id={}, epoch={}", conversation_id, latest_group.epoch)));
                                                    
                                                    // CRITICAL: Test if the new group can actually create messages
                                                    let test_rumor = EventBuilder::new(Kind::TextNote, "test".to_string())
                                                        .build(keys.public_key());
                                                    
                                                    match nostr_mls.create_message(&latest_group.mls_group_id, test_rumor) {
                                                        Ok(_) => {
                                                            log_messages.push(("INFO".to_string(), "✅ Group validation PASSED - can create messages".to_string()));
                                                            tracing::info!("Group validation passed - group is functional");
                                                        }
                                                        Err(e) => {
                                                            log_messages.push(("ERROR".to_string(), format!("❌ Group validation FAILED: {}", e)));
                                                            log_messages.push(("ERROR".to_string(), "The welcome message created a broken group!".to_string()));
                                                            tracing::error!("Group validation failed immediately after welcome: {}", e);
                                                            // Add collected log messages before returning error
                                                            for (level, msg) in log_messages {
                                                                self.add_log_entry(&level, &msg);
                                                            }
                                                            // Don't create conversation for broken group
                                                            return Err(DialogTuiError::Network { 
                                                                message: format!("Invite acceptance created broken group: {}", e) 
                                                            });
                                                        }
                                                    }
                                                    
                                                    Some((
                                                        conversation_id.clone(),
                                                        latest_group.mls_group_id.clone(),
                                                        hex::encode(&latest_group.nostr_group_id),
                                                        conversation_id,
                                                    ))
                                                } else {
                                                    tracing::warn!("No groups found after processing welcome");
                                                    log_messages.push(("WARN".to_string(), "No groups found after processing welcome".to_string()));
                                                    None
                                                }
                                            } else {
                                                tracing::error!("Failed to get groups after processing welcome");
                                                log_messages.push(("ERROR".to_string(), "Failed to get groups after processing welcome".to_string()));
                                                None
                                            };
                                            
                                            // Process existing messages for the group
                                            if let Some((conversation_id, _group_id, nostr_group_id_hex, _)) = &conversation_data {
                                                tracing::info!("Fetching existing messages for newly joined group...");
                                                
                                                let filter = Filter::new()
                                                    .kind(Kind::MlsGroupMessage)
                                                    .custom_tag(
                                                        nostr_sdk::SingleLetterTag::lowercase(nostr_sdk::Alphabet::H), 
                                                        nostr_group_id_hex.clone()
                                                    );
                                                
                                                if let Ok(existing_events) = client.fetch_events(filter, Duration::from_secs(10)).await {
                                                    tracing::info!("Found {} existing MLS messages for group", existing_events.len());
                                                    
                                                    let mut new_messages: Vec<(crate::model::ChatMessage, String)> = Vec::new();
                                                    
                                                    for existing_event in existing_events {
                                                        let event_id = existing_event.id.to_string();
                                                        
                                                        // Skip if already processed
                                                        if self.state.processed_message_ids.contains(&event_id) {
                                                            continue;
                                                        }
                                                        
                                                        // Mark as processed
                                                        self.state.processed_message_ids.insert(event_id.clone());
                                                        
                                                        // Try to process the message
                                                        match nostr_mls.process_message(&existing_event) {
                                                            Ok(nostr_mls::messages::MessageProcessingResult::ApplicationMessage(message)) => {
                                                                tracing::info!("Decrypted existing message: {}", message.content);
                                                                
                                                                let chat_message = crate::model::ChatMessage {
                                                                    id: event_id,
                                                                    conversation_id: conversation_id.clone(),
                                                                    sender: message.pubkey,
                                                                    content: message.content,
                                                                    timestamp: chrono::DateTime::from_timestamp(existing_event.created_at.as_u64() as i64, 0)
                                                                        .unwrap_or_else(|| chrono::Utc::now()),
                                                                    is_own: message.pubkey == keys.public_key(),
                                                                };
                                                                
                                                                messages_to_save.push(chat_message);
                                                            }
                                                            Ok(_) => {
                                                                tracing::debug!("Processed non-application MLS message: {}", event_id);
                                                            }
                                                            Err(e) => {
                                                                tracing::warn!("Failed to process existing MLS message {}: {}", event_id, e);
                                                            }
                                                        }
                                                    }
                                                    
                                                }
                                            }
                                            
                                            // Mark that we successfully processed the invite
                                            conversation_created = true;
                                        }
                                        Err(e) => {
                                            let error_msg = format!("Failed to process welcome from {}: {}", invite_petname, e);
                                            tracing::error!("{}", error_msg);
                                        }
                                    }
                                } // End of mutable borrow scope
                                
                                // Add any collected log messages
                                for (level, msg) in log_messages {
                                    self.add_log_entry(&level, &msg);
                                }
                                
                                // Now handle all the operations that require storage access
                                if conversation_created {
                                    // Add messages to state
                                    for chat_message in &messages_to_save {
                                        self.state.messages
                                            .entry(chat_message.conversation_id.clone())
                                            .or_insert_with(Vec::new)
                                            .push(chat_message.clone());
                                    }
                                    
                                    // Save messages to storage
                                    for chat_message in messages_to_save {
                                        let _ = self.storage.save_message(&chat_message).await;
                                    }
                                    
                                    // Create and save conversation
                                    if let Some(nostr_mls) = self.storage.get_nostr_mls() {
                                        if let Ok(groups) = nostr_mls.get_groups() {
                                            if let Some(latest_group) = groups.last() {
                                                let conversation_id = hex::encode(latest_group.mls_group_id.as_slice());
                                                let conversation = crate::model::state::Conversation {
                                                    id: conversation_id.clone(),
                                                    group_id: Some(latest_group.mls_group_id.clone()),
                                                    name: invite_petname.clone(),
                                                    participants: vec![invite_from],
                                                    last_message_time: None,
                                                    unread_count: 0,
                                                };
                                                
                                                // Save and add to state
                                                self.storage.save_conversation(&conversation).await?;
                                                self.state.conversations.insert(conversation_id.clone(), conversation);
                                                
                                                tracing::info!("Created conversation for accepted invite: {}", conversation_id);
                                                self.add_log_entry("INFO", &format!("Successfully joined group invited by {}", invite_petname));
                                            }
                                        }
                                    }
                                    
                                    // Remove from pending invites
                                    self.state.pending_invites.remove(invite_index);
                                } else {
                                    let error_msg = "NostrMLS not initialized for accepting invite";
                                    tracing::error!("{}", error_msg);
                                    self.add_log_entry("ERROR", error_msg);
                                }
                            } else {
                                let error_msg = format!("Failed to decrypt gift wrap for invite from {}", invite.petname);
                                tracing::error!("{}", error_msg);
                                self.add_log_entry("ERROR", &error_msg);
                            }
                        } else {
                            let error_msg = format!("Failed to fetch invite event: {}", invite.event_id);
                            tracing::error!("{}", error_msg);
                            self.add_log_entry("ERROR", &error_msg);
                        }
                    } else {
                        let error_msg = format!("Invalid invite index: {}", invite_index);
                        tracing::error!("{}", error_msg);
                        self.add_log_entry("ERROR", &error_msg);
                    }
                } else {
                    let error_msg = "Client or keys not available for accepting invite";
                    tracing::error!("{}", error_msg);
                    self.add_log_entry("ERROR", error_msg);
                }
                Ok(())
            }
            Cmd::FetchNewMessages => {
                if let (Some(client), Some(keys)) = (&self.client, &self.keys) {
                    // First, get the groups (without holding a mutable borrow)
                    let groups = if let Some(nostr_mls) = self.storage.get_nostr_mls() {
                        nostr_mls.get_groups().unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    
                    let mut new_messages = Vec::new();
                    
                    for group in groups {
                        let nostr_group_id_hex = hex::encode(&group.nostr_group_id);
                        let conversation_id = hex::encode(group.mls_group_id.as_slice());
                        
                        // Fetch MLS group messages for this group
                        let filter = Filter::new()
                            .kind(Kind::MlsGroupMessage)
                            .custom_tag(
                                nostr_sdk::SingleLetterTag::lowercase(nostr_sdk::Alphabet::H), 
                                nostr_group_id_hex
                            );
                        
                        let events = client.fetch_events(filter, Duration::from_secs(5)).await
                            .map_err(|e| DialogTuiError::Network { 
                                message: format!("Failed to fetch MLS messages: {}", e) 
                            })?;
                        
                        for event in events {
                            let event_id = event.id.to_string();
                            
                            // Check if we've already processed this message (either successfully or unsuccessfully)
                            if self.state.processed_message_ids.contains(&event_id) {
                                tracing::debug!("Skipping already processed message: {}", event_id);
                                continue;
                            }
                            
                            // Mark as processed to prevent future reprocessing
                            self.state.processed_message_ids.insert(event_id.clone());
                            
                            // Also check if it already exists in our messages
                            let message_exists = self.state.messages
                                .get(&conversation_id)
                                .map(|msgs| msgs.iter().any(|m| m.id == event_id))
                                .unwrap_or(false);
                            
                            if !message_exists {
                                // Process the MLS message to decrypt it (using a fresh mutable borrow)
                                if let Some(nostr_mls) = self.storage.get_nostr_mls_mut() {
                                    match nostr_mls.process_message(&event) {
                                        Ok(MessageProcessingResult::ApplicationMessage(message)) => {
                                            tracing::info!("Decrypted application message from group {}: {}", conversation_id, message.content);
                                            
                                            // Create chat message from decrypted content
                                            let chat_message = crate::model::ChatMessage {
                                                id: event.id.to_string(),
                                                conversation_id: conversation_id.clone(),
                                                sender: message.pubkey,
                                                content: message.content.clone(),
                                                timestamp: chrono::DateTime::from_timestamp(event.created_at.as_u64() as i64, 0)
                                                    .unwrap_or_else(|| chrono::Utc::now()),
                                                is_own: message.pubkey == keys.public_key(),
                                            };
                                            
                                            new_messages.push((chat_message, message.content));
                                        }
                                        Ok(MessageProcessingResult::Proposal(_)) => {
                                            tracing::debug!("Processed MLS proposal message: {}", event.id);
                                        }
                                        Ok(MessageProcessingResult::Commit) => {
                                            tracing::debug!("Processed MLS commit message: {}", event.id);
                                        }
                                        Ok(MessageProcessingResult::ExternalJoinProposal) => {
                                            tracing::debug!("Processed MLS external join proposal: {}", event.id);
                                        }
                                        Ok(MessageProcessingResult::Unprocessable) => {
                                            tracing::debug!("Unprocessable MLS message (likely own message or already processed): {}", event.id);
                                        }
                                        Err(e) => {
                                            tracing::warn!("Failed to process MLS message {}: {}", event.id, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Now add messages to state and save them (no mutable borrow conflicts)
                    for (chat_message, content) in new_messages {
                        let conversation_id = chat_message.conversation_id.clone();
                        
                        // Add to state
                        self.state.messages
                            .entry(conversation_id.clone())
                            .or_insert_with(Vec::new)
                            .push(chat_message.clone());
                        
                        // Update conversation last message time
                        if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                            conversation.last_message_time = Some(chat_message.timestamp);
                            if !chat_message.is_own && Some(&conversation_id) != self.state.selected_conversation.as_ref() {
                                conversation.unread_count += 1;
                            }
                        }
                        
                        // Save to storage
                        self.storage.save_message(&chat_message).await?;
                        
                        // Add log entry now that we don't have any borrows
                        self.add_log_entry("INFO", &format!("Received message: {}", content));
                    }
                }
                Ok(())
            }
            Cmd::ResetAllState => {
                tracing::warn!("Resetting all application state");
                
                // Clear all in-memory state
                self.state.contacts.clear();
                self.state.conversations.clear();
                self.state.messages.clear();
                self.state.pending_invites.clear();
                self.state.selected_contact = None;
                self.state.selected_conversation = None;
                self.state.selected_invite = None;
                
                // Clear storage (this is destructive!)
                let _ = self.storage.clear_all_data().await;
                
                // Reset MLS state (this will force re-initialization)
                if let Some(keys) = &self.keys {
                    let _ = self.storage.reset_mls_state(keys).await;
                }
                
                self.add_log_entry("WARN", "All state has been reset!");
                Ok(())
            }
            Cmd::DeleteAllContacts => {
                tracing::warn!("Deleting all contacts");
                
                self.state.contacts.clear();
                self.state.selected_contact = None;
                
                let _ = self.storage.clear_contacts().await;
                
                self.add_log_entry("WARN", "All contacts deleted");
                Ok(())
            }
            Cmd::DeleteAllConversations => {
                tracing::warn!("Deleting all conversations");
                
                self.state.conversations.clear();
                self.state.messages.clear();
                self.state.selected_conversation = None;
                
                let _ = self.storage.clear_conversations().await;
                
                self.add_log_entry("WARN", "All conversations deleted");
                Ok(())
            }
            Cmd::RescanRelays => {
                tracing::info!("Rescanning relays");
                
                if let Some(client) = &self.client {
                    // Disconnect and reconnect
                    let _ = client.disconnect().await;
                    client.connect().await;
                    
                    self.state.connection_status = crate::model::state::ConnectionStatus::Connected;
                    
                    // Fetch both invites and messages using Box::pin for recursion
                    let _ = Box::pin(self.execute_command(Cmd::FetchPendingInvites)).await;
                    let _ = Box::pin(self.execute_command(Cmd::FetchNewMessages)).await;
                }
                
                self.add_log_entry("INFO", "Rescanned relays and refetched data");
                Ok(())
            }
            Cmd::RepublishKeypackage => {
                tracing::info!("Republishing keypackage");
                self.add_log_entry("INFO", "Republishing keypackage...");
                Box::pin(self.execute_command(Cmd::PublishKeypackageToRelay)).await
            }
        }
    }
    
    fn add_log_entry(&mut self, level: &str, message: &str) {
        let entry = crate::model::LogEntry {
            timestamp: chrono::Utc::now(),
            level: level.to_string(),
            message: message.to_string(),
        };
        
        // Write to file first (persistent logging)
        if let Some(ref mut file) = self.log_file {
            let log_line = format!("{} [{}] {}\n", 
                entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"), 
                entry.level, 
                entry.message
            );
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush(); // Ensure it's written immediately
        }
        
        self.state.debug_logs.push(entry.clone());
        
        // Keep only the last 1000 log entries in memory
        if self.state.debug_logs.len() > 1000 {
            self.state.debug_logs.drain(0..self.state.debug_logs.len() - 1000);
        }
        
        // Also send to tracing for file logging
        match level {
            "ERROR" => tracing::error!("{}", message),
            "WARN" => tracing::warn!("{}", message),
            "INFO" => tracing::info!("{}", message),
            "DEBUG" => tracing::debug!("{}", message),
            _ => tracing::trace!("{}", message),
        }
        
        // Add toast notifications for warnings and errors
        if level == "WARN" || level == "ERROR" {
            self.add_toast_notification(level, message);
        }
    }
    
    fn add_toast_notification(&mut self, level: &str, message: &str) {
        let duration = match level {
            "ERROR" => 8, // Show errors for 8 seconds
            "WARN" => 5,  // Show warnings for 5 seconds  
            _ => 3,       // Other notifications for 3 seconds
        };
        
        let toast = crate::model::ToastNotification {
            message: message.to_string(),
            level: level.to_string(),
            timestamp: chrono::Utc::now(),
            duration_secs: duration,
        };
        
        self.state.toast_notifications.push(toast);
        
        // Keep only the last 5 toast notifications
        if self.state.toast_notifications.len() > 5 {
            self.state.toast_notifications.drain(0..self.state.toast_notifications.len() - 5);
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen
        );
    }
}

