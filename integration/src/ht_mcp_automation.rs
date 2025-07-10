use anyhow::{anyhow, Result};
use std::time::Duration;
use tracing::{info, warn};

/// ht-mcp automation client for dialog_tui interactions
pub struct DialogTuiAutomation {
    session_id: Option<String>,
}

impl DialogTuiAutomation {
    pub fn new() -> Self {
        Self { session_id: None }
    }

    /// Create a new ht-mcp session running dialog_tui
    /// This is a placeholder that would use the actual MCP functions available in the environment
    pub async fn create_session(&mut self, key_name: &str, relay_urls: &str) -> Result<String> {
        info!("Creating ht-mcp session for dialog_tui with key: {}", key_name);
        info!("Would run: DIALOG_RELAY_URLS={} cargo run --bin dialog_tui -- --key {}", relay_urls, key_name);
        
        // For now, create a mock session ID since we need the actual MCP environment
        // In the real implementation, this would use the ht-mcp server
        let session_id = format!("mock_session_{}", key_name);
        self.session_id = Some(session_id.clone());
        
        info!("Created mock session: {} (replace with real ht-mcp call)", session_id);
        
        // Simulate startup time
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        Ok(session_id)
    }

    /// Send keys to the active session
    pub async fn send_keys(&self, keys: &[&str]) -> Result<()> {
        let session_id = self.session_id.as_ref()
            .ok_or_else(|| anyhow!("No active session"))?;

        for key in keys {
            let output = Command::new("ht-mcp")
                .args(&["send-keys", session_id, key])
                .output()
                .map_err(|e| anyhow!("Failed to send key '{}': {}", key, e))?;

            if !output.status.success() {
                warn!("Failed to send key '{}': {}", key, 
                    String::from_utf8_lossy(&output.stderr));
            }
            
            // Small delay between keystrokes
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Take a snapshot of current terminal state
    pub async fn take_snapshot(&self) -> Result<String> {
        let session_id = self.session_id.as_ref()
            .ok_or_else(|| anyhow!("No active session"))?;

        let output = Command::new("ht-mcp")
            .args(&["take-snapshot", session_id])
            .output()
            .map_err(|e| anyhow!("Failed to take snapshot: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!("Snapshot failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Wait for specific text to appear in terminal output
    pub async fn wait_for_text(&self, expected_text: &str, timeout_secs: u64) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        loop {
            if start_time.elapsed().as_secs() > timeout_secs {
                return Err(anyhow!("Timeout waiting for text: {}", expected_text));
            }

            let snapshot = self.take_snapshot().await?;
            if snapshot.contains(expected_text) {
                info!("Found expected text: {}", expected_text);
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    /// Setup dialog_tui for testing (connect and publish key packages)
    pub async fn setup_dialog_tui(&self) -> Result<String> {
        info!("Setting up dialog_tui for testing");

        // Connect to relay
        self.send_keys(&["/connect", "Enter"]).await?;
        self.wait_for_text("Connected", 10).await?;

        // Publish key packages
        self.send_keys(&["/keypackage", "Enter"]).await?;
        self.wait_for_text("Published", 10).await?;

        // Get public key
        self.send_keys(&["/pk", "Enter"]).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        let snapshot = self.take_snapshot().await?;
        let pubkey = self.extract_pubkey_from_output(&snapshot)?;
        
        info!("Dialog_TUI setup complete, pubkey: {}", pubkey);
        Ok(pubkey)
    }

    /// Accept an invitation and join a group
    pub async fn accept_invite_and_join(&self) -> Result<()> {
        info!("Checking for invites and accepting");

        // Check for invites
        self.send_keys(&["/invites", "Enter"]).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Accept first invite (assuming it exists)
        self.send_keys(&["Enter"]).await?;
        self.wait_for_text("Successfully joined", 15).await?;

        info!("Successfully joined group");
        Ok(())
    }

    /// Send a test message to the current group
    pub async fn send_test_message(&self, message: &str) -> Result<()> {
        info!("Sending test message: {}", message);

        self.send_keys(&[message, "Enter"]).await?;
        
        // Fetch messages to see our own message
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.send_keys(&["/fetch", "Enter"]).await?;
        
        Ok(())
    }

    /// Create a group and invite members
    pub async fn create_group_and_invite(&self, group_name: &str, member_pubkey: &str) -> Result<()> {
        info!("Creating group '{}' and inviting member", group_name);

        // Add contact first
        self.send_keys(&[&format!("/add {}", member_pubkey), "Enter"]).await?;
        self.wait_for_text("Contact added", 10).await?;

        // Create group
        self.send_keys(&[&format!("/create {}", group_name), "Enter"]).await?;
        
        // Navigate and select the contact (this is interactive)
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.send_keys(&[" ", "Enter"]).await?; // Space to select, Enter to confirm
        
        self.wait_for_text(&format!("Group '{}' created successfully", group_name), 15).await?;

        // Send welcome message
        self.send_keys(&["Welcome to the group!", "Enter"]).await?;

        info!("Group '{}' created and invitation sent", group_name);
        Ok(())
    }

    /// Extract pubkey from /pk command output
    fn extract_pubkey_from_output(&self, output: &str) -> Result<String> {
        // Look for "Hex: " followed by 64 hex characters
        for line in output.lines() {
            if let Some(hex_pos) = line.find("Hex: ") {
                let hex_start = hex_pos + 5;
                if line.len() >= hex_start + 64 {
                    let pubkey = &line[hex_start..hex_start + 64];
                    if pubkey.chars().all(|c| c.is_ascii_hexdigit()) {
                        return Ok(pubkey.to_string());
                    }
                }
            }
        }
        Err(anyhow!("Could not extract pubkey from output"))
    }

    /// Close the ht-mcp session
    pub async fn close_session(&mut self) -> Result<()> {
        if let Some(session_id) = &self.session_id {
            info!("Closing ht-mcp session: {}", session_id);
            
            let output = Command::new("ht-mcp")
                .args(&["close-session", session_id])
                .output()
                .map_err(|e| anyhow!("Failed to close session: {}", e))?;

            if !output.status.success() {
                warn!("Failed to close session: {}", 
                    String::from_utf8_lossy(&output.stderr));
            }

            self.session_id = None;
        }
        Ok(())
    }
}

impl Drop for DialogTuiAutomation {
    fn drop(&mut self) {
        if self.session_id.is_some() {
            // Note: This is a blocking operation in Drop, which isn't ideal
            // In practice, sessions should be explicitly closed
            warn!("Dialog TUI automation session not properly closed");
        }
    }
}