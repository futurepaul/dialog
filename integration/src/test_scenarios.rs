use anyhow::Result;
use std::time::Duration;
use tracing::info;

use crate::ht_mcp_automation::DialogTuiAutomation;

/// Test scenario configurations and coordination
pub struct TestScenarios {
    relay_urls: String,
    backup_relay: String,
}

impl TestScenarios {
    pub fn new() -> Self {
        Self {
            relay_urls: "ws://localhost:8080,ws://localhost:7777".to_string(),
            backup_relay: "ws://localhost:10547".to_string(),
        }
    }

    /// Complete end-to-end test: Whitenoise creates, dialog_tui joins, bi-directional chat
    pub async fn run_complete_interop_test(&self) -> Result<()> {
        info!("=== RUNNING COMPLETE INTEROPERABILITY TEST ===");
        
        // Phase 1: Setup dialog_tui for whitenoise to invite
        let dialog_pubkey = self.setup_dialog_for_whitenoise_invite().await?;
        
        // Phase 2: Whitenoise coordination (this would be called by whitenoise)
        self.coordinate_whitenoise_group_creation(&dialog_pubkey).await?;
        
        // Phase 3: Dialog accepts and begins chatting
        self.coordinate_dialog_acceptance_and_chat().await?;
        
        // Phase 4: Reverse test - dialog creates, whitenoise joins
        self.run_reverse_interop_test().await?;
        
        info!("=== COMPLETE INTEROPERABILITY TEST FINISHED ===");
        Ok(())
    }

    /// Phase 1: Setup dialog_tui to be invited by whitenoise
    async fn setup_dialog_for_whitenoise_invite(&self) -> Result<String> {
        info!("Phase 1: Setting up dialog_tui for whitenoise invitation");
        
        let mut dialog_automation = DialogTuiAutomation::new();
        let _session_id = dialog_automation.create_session("alice_dialog", &self.relay_urls).await?;
        
        let dialog_pubkey = dialog_automation.setup_dialog_tui().await?;
        info!("Dialog_tui ready for invitation with pubkey: {}", dialog_pubkey);
        
        // Keep session alive for whitenoise to use
        std::mem::forget(dialog_automation); // Prevent cleanup
        
        Ok(dialog_pubkey)
    }

    /// Phase 2: Coordinate whitenoise group creation (called by whitenoise integration test)
    async fn coordinate_whitenoise_group_creation(&self, dialog_pubkey: &str) -> Result<()> {
        info!("Phase 2: Coordinating whitenoise group creation");
        info!("Whitenoise should create group and invite: {}", dialog_pubkey);
        
        // This is where whitenoise would:
        // 1. Create group with dialog_pubkey as member
        // 2. Send welcome messages
        // 3. Setup group subscriptions
        
        // Simulate whitenoise processing time
        tokio::time::sleep(Duration::from_secs(8)).await;
        
        Ok(())
    }

    /// Phase 3: Dialog accepts invitation and starts chatting
    async fn coordinate_dialog_acceptance_and_chat(&self) -> Result<()> {
        info!("Phase 3: Dialog accepting invitation and starting chat");
        
        // Create new automation session (previous was forgotten)
        let mut dialog_automation = DialogTuiAutomation::new();
        let _session_id = dialog_automation.create_session("alice_dialog", &self.relay_urls).await?;
        
        // Re-setup (connect and key packages)
        dialog_automation.setup_dialog_tui().await?;
        
        // Accept the invitation from whitenoise
        dialog_automation.accept_invite_and_join().await?;
        
        // Start conversation
        for i in 1..=3 {
            dialog_automation.send_test_message(&format!("Dialog message {} to whitenoise", i)).await?;
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        
        // Fetch to see any whitenoise responses
        dialog_automation.send_keys(&["/fetch", "Enter"]).await?;
        
        dialog_automation.close_session().await?;
        info!("Phase 3 completed: Dialog participation finished");
        
        Ok(())
    }

    /// Phase 4: Reverse test - dialog creates group, invites whitenoise
    async fn run_reverse_interop_test(&self) -> Result<()> {
        info!("Phase 4: Reverse interop test - dialog creates, whitenoise joins");
        
        let mut dialog_automation = DialogTuiAutomation::new();
        let _session_id = dialog_automation.create_session("bob_dialog", &self.relay_urls).await?;
        
        let dialog_pubkey = dialog_automation.setup_dialog_tui().await?;
        
        // Simulate getting whitenoise pubkey (would come from whitenoise coordination)
        let whitenoise_pubkey = "simulated_whitenoise_pubkey_for_reverse_test";
        
        // Create group and invite whitenoise
        dialog_automation.create_group_and_invite("ReverseTestGroup", whitenoise_pubkey).await?;
        
        // Wait for whitenoise to accept and respond
        info!("Waiting for whitenoise to accept invitation...");
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        // Send some messages
        dialog_automation.send_test_message("Welcome whitenoise to dialog's group!").await?;
        
        // Check for whitenoise responses
        tokio::time::sleep(Duration::from_secs(5)).await;
        dialog_automation.send_keys(&["/fetch", "Enter"]).await?;
        
        dialog_automation.close_session().await?;
        info!("Phase 4 completed: Reverse interop test finished");
        
        Ok(())
    }

    /// Stress test: Multiple message exchanges
    pub async fn run_stress_test(&self) -> Result<()> {
        info!("=== RUNNING STRESS TEST ===");
        
        let mut dialog_automation = DialogTuiAutomation::new();
        let _session_id = dialog_automation.create_session("stress_tester", &self.relay_urls).await?;
        
        dialog_automation.setup_dialog_tui().await?;
        
        // Assume group already exists for stress testing
        info!("Starting stress test message burst");
        
        // Send rapid messages
        for i in 1..=20 {
            dialog_automation.send_test_message(&format!("Stress test message {}/20", i)).await?;
            
            if i % 5 == 0 {
                // Periodic fetch to check responses
                dialog_automation.send_keys(&["/fetch", "Enter"]).await?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            } else {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
        
        // Final message check
        dialog_automation.send_keys(&["/fetch", "Enter"]).await?;
        
        dialog_automation.close_session().await?;
        info!("=== STRESS TEST COMPLETED ===");
        
        Ok(())
    }

    /// Test error recovery scenarios
    pub async fn run_error_recovery_test(&self) -> Result<()> {
        info!("=== RUNNING ERROR RECOVERY TEST ===");
        
        let mut dialog_automation = DialogTuiAutomation::new();
        let _session_id = dialog_automation.create_session("error_tester", &self.relay_urls).await?;
        
        dialog_automation.setup_dialog_tui().await?;
        
        // Test reconnection by disconnecting and reconnecting
        info!("Testing reconnection scenario");
        dialog_automation.send_keys(&["/disconnect", "Enter"]).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        dialog_automation.send_keys(&["/connect", "Enter"]).await?;
        dialog_automation.wait_for_text("Connected", 10).await?;
        
        // Test message sending after reconnection
        dialog_automation.send_test_message("Message after reconnection").await?;
        
        // Test key package refresh
        dialog_automation.send_keys(&["/refresh-keys", "Enter"]).await?;
        dialog_automation.wait_for_text("Refreshed", 10).await?;
        
        dialog_automation.close_session().await?;
        info!("=== ERROR RECOVERY TEST COMPLETED ===");
        
        Ok(())
    }
}

/// Helper for coordinating between whitenoise integration tests and dialog_tui automation
pub struct InteropCoordinator;

impl InteropCoordinator {
    /// Get a ready dialog_tui pubkey for whitenoise to invite
    pub async fn prepare_dialog_for_whitenoise(key_name: &str, relay_urls: &str) -> Result<String> {
        let mut dialog_automation = DialogTuiAutomation::new();
        let _session_id = dialog_automation.create_session(key_name, relay_urls).await?;
        let pubkey = dialog_automation.setup_dialog_tui().await?;
        
        // Keep session alive - whitenoise will coordinate with it
        std::mem::forget(dialog_automation);
        
        Ok(pubkey)
    }
    
    /// Signal dialog_tui to accept invitation (used by whitenoise tests)
    pub async fn signal_dialog_to_accept_invite(key_name: &str, relay_urls: &str) -> Result<()> {
        // Create new session for the specific key
        let mut dialog_automation = DialogTuiAutomation::new();
        let _session_id = dialog_automation.create_session(key_name, relay_urls).await?;
        
        // Setup and accept invite
        dialog_automation.setup_dialog_tui().await?;
        dialog_automation.accept_invite_and_join().await?;
        
        // Send confirmation message
        dialog_automation.send_test_message("Hello from dialog_tui - joined successfully!").await?;
        
        // Keep session alive for continued testing
        std::mem::forget(dialog_automation);
        
        Ok(())
    }
    
    /// Have dialog_tui send messages in existing group
    pub async fn send_dialog_messages(key_name: &str, relay_urls: &str, messages: Vec<String>) -> Result<()> {
        let mut dialog_automation = DialogTuiAutomation::new();
        let _session_id = dialog_automation.create_session(key_name, relay_urls).await?;
        
        dialog_automation.setup_dialog_tui().await?;
        
        for message in messages {
            dialog_automation.send_test_message(&message).await?;
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        
        dialog_automation.close_session().await?;
        Ok(())
    }
}