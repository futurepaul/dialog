/// Comprehensive automation coordination between ht-mcp, dialog_tui, and whitenoise
/// This module provides the orchestration layer for complex multi-client testing

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn, error};

use crate::ht_mcp_automation::DialogTuiAutomation;

/// Central coordinator for managing multiple test clients and scenarios
pub struct AutomationCoordinator {
    /// Active dialog_tui sessions (key_name -> automation)
    dialog_sessions: Arc<RwLock<HashMap<String, DialogTuiAutomation>>>,
    /// Test configuration
    config: TestConfig,
    /// Event bus for coordination messages
    event_bus: mpsc::Sender<CoordinationEvent>,
    /// Event receiver for processing coordination events
    event_receiver: Arc<RwLock<Option<mpsc::Receiver<CoordinationEvent>>>>,
}

impl AutomationCoordinator {
    /// Create new automation coordinator
    pub fn new(config: TestConfig) -> Self {
        let (sender, receiver) = mpsc::channel(100);
        
        Self {
            dialog_sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            event_bus: sender,
            event_receiver: Arc::new(RwLock::new(Some(receiver))),
        }
    }

    /// Start the coordination event processing loop
    pub async fn start_coordination_loop(&self) -> Result<()> {
        info!("Starting automation coordination event loop");
        
        let mut receiver = self.event_receiver.write().await
            .take()
            .ok_or_else(|| anyhow!("Event receiver already taken"))?;

        // Process coordination events
        while let Some(event) = receiver.recv().await {
            if let Err(e) = self.handle_coordination_event(event).await {
                error!("Failed to handle coordination event: {}", e);
            }
        }

        Ok(())
    }

    /// Handle individual coordination events
    async fn handle_coordination_event(&self, event: CoordinationEvent) -> Result<()> {
        match event {
            CoordinationEvent::SetupDialogTui { key_name, response_tx } => {
                let result = self.setup_dialog_tui_session(&key_name).await;
                let _ = response_tx.send(result);
            }
            CoordinationEvent::SendDialogMessage { key_name, message, response_tx } => {
                let result = self.send_dialog_message(&key_name, &message).await;
                let _ = response_tx.send(result);
            }
            CoordinationEvent::CreateDialogGroup { key_name, group_name, member_pubkey, response_tx } => {
                let result = self.create_dialog_group(&key_name, &group_name, &member_pubkey).await;
                let _ = response_tx.send(result);
            }
            CoordinationEvent::AcceptDialogInvite { key_name, response_tx } => {
                let result = self.accept_dialog_invite(&key_name).await;
                let _ = response_tx.send(result);
            }
            CoordinationEvent::CleanupSession { key_name, response_tx } => {
                let result = self.cleanup_session(&key_name).await;
                let _ = response_tx.send(result);
            }
        }
        Ok(())
    }

    /// Setup a new dialog_tui session
    async fn setup_dialog_tui_session(&self, key_name: &str) -> Result<String> {
        info!("Setting up dialog_tui session for key: {}", key_name);
        
        let mut dialog_automation = DialogTuiAutomation::new();
        let session_id = dialog_automation.create_session(key_name, &self.config.relay_urls).await?;
        let pubkey = dialog_automation.setup_dialog_tui().await?;
        
        // Store the session for later use
        self.dialog_sessions.write().await
            .insert(key_name.to_string(), dialog_automation);
        
        info!("Dialog_tui session ready: {} -> {}", key_name, pubkey);
        Ok(pubkey)
    }

    /// Send message from dialog_tui session
    async fn send_dialog_message(&self, key_name: &str, message: &str) -> Result<()> {
        let sessions = self.dialog_sessions.read().await;
        let session = sessions.get(key_name)
            .ok_or_else(|| anyhow!("No session found for key: {}", key_name))?;
        
        session.send_test_message(message).await?;
        info!("Sent message from {}: {}", key_name, message);
        Ok(())
    }

    /// Create group via dialog_tui
    async fn create_dialog_group(&self, key_name: &str, group_name: &str, member_pubkey: &str) -> Result<()> {
        let sessions = self.dialog_sessions.read().await;
        let session = sessions.get(key_name)
            .ok_or_else(|| anyhow!("No session found for key: {}", key_name))?;
        
        session.create_group_and_invite(group_name, member_pubkey).await?;
        info!("Created group '{}' from {} and invited {}", group_name, key_name, member_pubkey);
        Ok(())
    }

    /// Accept invitation via dialog_tui
    async fn accept_dialog_invite(&self, key_name: &str) -> Result<()> {
        let sessions = self.dialog_sessions.read().await;
        let session = sessions.get(key_name)
            .ok_or_else(|| anyhow!("No session found for key: {}", key_name))?;
        
        session.accept_invite_and_join().await?;
        info!("Accepted invite for {}", key_name);
        Ok(())
    }

    /// Cleanup session
    async fn cleanup_session(&self, key_name: &str) -> Result<()> {
        let mut sessions = self.dialog_sessions.write().await;
        if let Some(mut session) = sessions.remove(key_name) {
            session.close_session().await?;
            info!("Cleaned up session for {}", key_name);
        }
        Ok(())
    }

    /// Public API methods for external coordination

    /// Setup dialog_tui and return pubkey (for whitenoise integration)
    pub async fn setup_dialog_for_whitenoise(&self, key_name: &str) -> Result<String> {
        let (response_tx, mut response_rx) = mpsc::channel(1);
        
        self.event_bus.send(CoordinationEvent::SetupDialogTui {
            key_name: key_name.to_string(),
            response_tx,
        }).await?;
        
        response_rx.recv().await
            .ok_or_else(|| anyhow!("No response received"))?
    }

    /// Have dialog_tui accept whitenoise invitation
    pub async fn signal_dialog_accept_invite(&self, key_name: &str) -> Result<()> {
        let (response_tx, mut response_rx) = mpsc::channel(1);
        
        self.event_bus.send(CoordinationEvent::AcceptDialogInvite {
            key_name: key_name.to_string(),
            response_tx,
        }).await?;
        
        response_rx.recv().await
            .ok_or_else(|| anyhow!("No response received"))?
    }

    /// Have dialog_tui send messages
    pub async fn send_dialog_messages(&self, key_name: &str, messages: Vec<String>) -> Result<()> {
        for message in messages {
            let (response_tx, mut response_rx) = mpsc::channel(1);
            
            self.event_bus.send(CoordinationEvent::SendDialogMessage {
                key_name: key_name.to_string(),
                message,
                response_tx,
            }).await?;
            
            response_rx.recv().await
                .ok_or_else(|| anyhow!("No response received"))??;
                
            // Small delay between messages
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok(())
    }

    /// Have dialog_tui create group and invite whitenoise
    pub async fn dialog_create_and_invite(&self, key_name: &str, group_name: &str, whitenoise_pubkey: &str) -> Result<()> {
        let (response_tx, mut response_rx) = mpsc::channel(1);
        
        self.event_bus.send(CoordinationEvent::CreateDialogGroup {
            key_name: key_name.to_string(),
            group_name: group_name.to_string(),
            member_pubkey: whitenoise_pubkey.to_string(),
            response_tx,
        }).await?;
        
        response_rx.recv().await
            .ok_or_else(|| anyhow!("No response received"))?
    }

    /// Cleanup all sessions
    pub async fn cleanup_all_sessions(&self) -> Result<()> {
        let sessions: Vec<String> = self.dialog_sessions.read().await.keys().cloned().collect();
        
        for key_name in sessions {
            if let Err(e) = self.cleanup_session(&key_name).await {
                warn!("Failed to cleanup session {}: {}", key_name, e);
            }
        }
        
        info!("Cleaned up all dialog_tui sessions");
        Ok(())
    }
}

/// Coordination events for the event bus
#[derive(Debug)]
enum CoordinationEvent {
    SetupDialogTui {
        key_name: String,
        response_tx: mpsc::Sender<Result<String>>,
    },
    SendDialogMessage {
        key_name: String,
        message: String,
        response_tx: mpsc::Sender<Result<()>>,
    },
    CreateDialogGroup {
        key_name: String,
        group_name: String,
        member_pubkey: String,
        response_tx: mpsc::Sender<Result<()>>,
    },
    AcceptDialogInvite {
        key_name: String,
        response_tx: mpsc::Sender<Result<()>>,
    },
    CleanupSession {
        key_name: String,
        response_tx: mpsc::Sender<Result<()>>,
    },
}

/// Test configuration for automation coordinator
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub relay_urls: String,
    pub backup_relay: String,
    pub default_timeout_secs: u64,
    pub message_delay_ms: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            relay_urls: "ws://localhost:8080,ws://localhost:7777".to_string(),
            backup_relay: "ws://localhost:10547".to_string(),
            default_timeout_secs: 30,
            message_delay_ms: 500,
        }
    }
}

/// High-level coordination interface for whitenoise integration tests
pub struct WhitenoiseTestCoordinator {
    coordinator: AutomationCoordinator,
}

impl WhitenoiseTestCoordinator {
    /// Create new whitenoise test coordinator
    pub fn new() -> Self {
        Self {
            coordinator: AutomationCoordinator::new(TestConfig::default()),
        }
    }

    /// Start the coordinator (should be called once at test start)
    pub async fn start(&self) -> Result<()> {
        // Start coordination loop in background
        let coordinator = AutomationCoordinator {
            dialog_sessions: Arc::clone(&self.coordinator.dialog_sessions),
            config: self.coordinator.config.clone(),
            event_bus: self.coordinator.event_bus.clone(),
            event_receiver: Arc::clone(&self.coordinator.event_receiver),
        };
        tokio::spawn(async move {
            if let Err(e) = coordinator.start_coordination_loop().await {
                error!("Coordination loop failed: {}", e);
            }
        });
        
        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// Complete test scenario: whitenoise creates group, dialog joins, messaging
    pub async fn test_whitenoise_creates_dialog_joins(&self) -> Result<()> {
        info!("=== COORDINATED TEST: Whitenoise creates, Dialog joins ===");
        
        // Step 1: Setup dialog_tui
        let dialog_pubkey = self.coordinator.setup_dialog_for_whitenoise("alice").await?;
        info!("Dialog ready with pubkey: {}", dialog_pubkey);
        
        // Step 2: Whitenoise would create group here
        // whitenoise.create_group(..., vec![dialog_pubkey], ...).await?;
        info!("Whitenoise should create group with dialog_tui member");
        
        // Step 3: Dialog accepts invitation
        tokio::time::sleep(Duration::from_secs(5)).await; // Give whitenoise time
        self.coordinator.signal_dialog_accept_invite("alice").await?;
        
        // Step 4: Exchange messages
        self.coordinator.send_dialog_messages("alice", vec![
            "Hello from dialog_tui!".to_string(),
            "Testing interoperability".to_string(),
        ]).await?;
        
        info!("=== COORDINATED TEST COMPLETED ===");
        Ok(())
    }

    /// Complete test scenario: dialog creates group, whitenoise joins, messaging
    pub async fn test_dialog_creates_whitenoise_joins(&self, whitenoise_pubkey: &str) -> Result<()> {
        info!("=== COORDINATED TEST: Dialog creates, Whitenoise joins ===");
        
        // Step 1: Setup dialog_tui
        let _dialog_pubkey = self.coordinator.setup_dialog_for_whitenoise("bob").await?;
        
        // Step 2: Dialog creates group and invites whitenoise
        self.coordinator.dialog_create_and_invite("bob", "TestGroup", whitenoise_pubkey).await?;
        
        // Step 3: Whitenoise would accept invitation here
        // whitenoise.accept_welcome(...).await?;
        info!("Whitenoise should accept dialog's invitation");
        
        // Step 4: Exchange messages after whitenoise joins
        tokio::time::sleep(Duration::from_secs(8)).await; // Give whitenoise time
        self.coordinator.send_dialog_messages("bob", vec![
            "Welcome to the group!".to_string(),
            "From dialog_tui creator".to_string(),
        ]).await?;
        
        info!("=== REVERSE COORDINATED TEST COMPLETED ===");
        Ok(())
    }

    /// Cleanup all test resources
    pub async fn cleanup(&self) -> Result<()> {
        self.coordinator.cleanup_all_sessions().await
    }
}