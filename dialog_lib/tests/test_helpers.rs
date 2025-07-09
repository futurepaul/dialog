use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use std::net::TcpListener;
use nostr::ToBech32;

#[derive(Debug)]
pub struct EphemeralRelay {
    pub process: Child,
    pub port: u16,
    pub url: String,
}

impl EphemeralRelay {
    /// Start an ephemeral nak serve relay on an available port
    pub async fn start() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let port = find_available_port()?;
        let url = format!("ws://localhost:{}", port);
        
        // Start nak serve with verbose logging
        let mut process = Command::new("nak")
            .args(&["serve", "--verbose", "--port", &port.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start nak serve: {}. Make sure 'nak' is installed and in PATH", e))?;
        
        // Give the relay a moment to start up
        sleep(Duration::from_millis(500)).await;
        
        // Check if process is still running
        match process.try_wait() {
            Ok(Some(status)) => {
                return Err(format!("nak serve exited immediately with status: {}", status).into());
            }
            Ok(None) => {
                // Process is still running, good!
            }
            Err(e) => {
                return Err(format!("Failed to check nak serve status: {}", e).into());
            }
        }
        
        Ok(EphemeralRelay {
            process,
            port,
            url,
        })
    }
    
    /// Get the WebSocket URL for this relay
    pub fn url(&self) -> &str {
        &self.url
    }
    
    /// Get the port this relay is running on
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for EphemeralRelay {
    fn drop(&mut self) {
        // Kill the nak serve process when the relay is dropped
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Find an available port for the ephemeral relay
fn find_available_port() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
    // Try to bind to port 0 to get an available port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener); // Release the port
    Ok(port)
}

/// Test utility for creating test users with deterministic keys
pub struct TestUser {
    pub name: String,
    pub keys: nostr::Keys,
    pub pubkey_hex: String,
    pub pubkey_bech32: String,
}

impl TestUser {
    /// Create a test user with deterministic keys based on a seed
    pub fn new(name: &str, seed: u64) -> Self {
        // Create deterministic keys for reproducible tests
        let secret_key = nostr::SecretKey::from_slice(&seed.to_be_bytes().repeat(4)).unwrap();
        let keys = nostr::Keys::new(secret_key);
        let pubkey_hex = keys.public_key().to_hex();
        let pubkey_bech32 = keys.public_key().to_bech32().unwrap();
        
        TestUser {
            name: name.to_string(),
            keys,
            pubkey_hex,
            pubkey_bech32,
        }
    }
    
    /// Get the public key as hex string
    pub fn pubkey_hex(&self) -> &str {
        &self.pubkey_hex
    }
    
    /// Get the public key as bech32 string
    pub fn pubkey_bech32(&self) -> &str {
        &self.pubkey_bech32
    }
    
    /// Get the nostr Keys
    pub fn keys(&self) -> &nostr::Keys {
        &self.keys
    }
}

/// Test scenario helper for multi-user testing
pub struct TestScenario {
    pub relay: EphemeralRelay,
    pub users: Vec<TestUser>,
}

impl TestScenario {
    /// Create a test scenario with an ephemeral relay and multiple users
    pub async fn new(user_names: &[&str]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let relay = EphemeralRelay::start().await?;
        
        let users = user_names
            .iter()
            .enumerate()
            .map(|(i, name)| TestUser::new(name, i as u64 + 1))
            .collect();
        
        Ok(TestScenario { relay, users })
    }
    
    /// Get a user by name
    pub fn get_user(&self, name: &str) -> Option<&TestUser> {
        self.users.iter().find(|user| user.name == name)
    }
    
    /// Get the relay URL
    pub fn relay_url(&self) -> &str {
        self.relay.url()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ephemeral_relay_startup() {
        let relay = EphemeralRelay::start().await.expect("Failed to start ephemeral relay");
        
        // Verify the relay is running
        assert!(relay.port() > 0);
        assert!(relay.url().starts_with("ws://localhost:"));
        
        // The relay should be automatically cleaned up when dropped
    }
    
    #[tokio::test]
    async fn test_test_user_creation() {
        let alice = TestUser::new("alice", 1);
        let bob = TestUser::new("bob", 2);
        
        // Users should have different keys
        assert_ne!(alice.pubkey_hex(), bob.pubkey_hex());
        assert_ne!(alice.pubkey_bech32(), bob.pubkey_bech32());
        
        // Keys should be deterministic (same seed = same keys)
        let alice2 = TestUser::new("alice", 1);
        assert_eq!(alice.pubkey_hex(), alice2.pubkey_hex());
    }
    
    #[tokio::test]
    async fn test_scenario_setup() {
        let scenario = TestScenario::new(&["alice", "bob", "charlie"])
            .await
            .expect("Failed to create test scenario");
        
        // Should have 3 users
        assert_eq!(scenario.users.len(), 3);
        
        // Should be able to find users by name
        assert!(scenario.get_user("alice").is_some());
        assert!(scenario.get_user("bob").is_some());
        assert!(scenario.get_user("charlie").is_some());
        assert!(scenario.get_user("nonexistent").is_none());
        
        // Relay should be running
        assert!(scenario.relay_url().starts_with("ws://localhost:"));
    }
}