use crate::DialogClient as CoreDialogClient;
use anyhow::Result;
use whitenoise::{PublicKey, Event};
use nostr::EventId;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Error types for UniFFI
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ClientError {
    #[error("Invalid key: {message}")]
    InvalidKey { message: String },
    #[error("Connection error: {message}")]
    ConnectionError { message: String },
    #[error("Encryption error: {message}")]
    EncryptionError { message: String },
    #[error("Generic error: {message}")]
    Generic { message: String },
}

impl From<anyhow::Error> for ClientError {
    fn from(err: anyhow::Error) -> Self {
        ClientError::Generic {
            message: err.to_string(),
        }
    }
}

// Data structures for UniFFI
#[derive(Debug, Clone, uniffi::Record)]
pub struct NoteData {
    pub id: String,
    pub content: String,
    pub author: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct EncryptedMessage {
    pub id: String,
    pub content: String,
    pub sender: String,
    pub created_at: u64,
}

// Main UniFFI interface
#[derive(uniffi::Object)]
pub struct DialogClient {
    core: Arc<CoreDialogClient>,
    runtime: Arc<Runtime>,
}

#[uniffi::export]
impl DialogClient {
    #[uniffi::constructor]
    pub fn new() -> Result<Self, ClientError> {
        let runtime = Runtime::new().map_err(|e| ClientError::Generic {
            message: format!("Failed to create runtime: {}", e),
        })?;
        
        let core = runtime.block_on(async {
            CoreDialogClient::new().await
        }).map_err(|e| ClientError::Generic {
            message: e.to_string(),
        })?;

        Ok(DialogClient {
            core: Arc::new(core),
            runtime: Arc::new(runtime),
        })
    }

    #[uniffi::constructor]
    pub fn new_with_key(secret_key_hex: String) -> Result<Self, ClientError> {
        let runtime = Runtime::new().map_err(|e| ClientError::Generic {
            message: format!("Failed to create runtime: {}", e),
        })?;
        
        let core = runtime.block_on(async {
            CoreDialogClient::new_with_key(&secret_key_hex).await
        }).map_err(|e| ClientError::InvalidKey {
            message: e.to_string(),
        })?;

        Ok(DialogClient {
            core: Arc::new(core),
            runtime: Arc::new(runtime),
        })
    }

    pub fn connect_to_relay(&self, relay_url: String) -> Result<(), ClientError> {
        self.runtime.block_on(async {
            self.core.connect_to_relay(&relay_url).await
        }).map_err(|e| ClientError::ConnectionError {
            message: e.to_string(),
        })
    }

    pub fn publish_note(&self, content: String) -> Result<String, ClientError> {
        let event_id = self.runtime.block_on(async {
            self.core.publish_note(&content).await
        }).map_err(|e| ClientError::Generic {
            message: e.to_string(),
        })?;
        
        Ok(event_id.to_hex())
    }

    pub fn get_notes(&self, limit: Option<u32>) -> Result<Vec<NoteData>, ClientError> {
        let events = self.runtime.block_on(async {
            self.core.get_notes(limit.map(|l| l as usize)).await
        }).map_err(|e| ClientError::Generic {
            message: e.to_string(),
        })?;

        let notes = events.into_iter().map(|event| NoteData {
            id: event.id.to_hex(),
            content: event.content,
            author: event.pubkey.to_hex(),
            created_at: event.created_at.as_u64(),
        }).collect();

        Ok(notes)
    }

    pub fn get_public_key(&self) -> String {
        self.core.get_public_key()
            .map(|pk| pk.to_hex())
            .unwrap_or_else(|| "No public key available".to_string())
    }

    pub fn get_secret_key_hex(&self) -> String {
        self.runtime.block_on(async {
            self.core.get_secret_key_hex().await
                .unwrap_or(None)
                .unwrap_or_else(|| "No secret key available".to_string())
        })
    }

    // TODO: Fix type compatibility between whitenoise and nostr_sdk types
    // pub fn send_encrypted_message(&self, recipient_pubkey: String, content: String) -> Result<String, ClientError> {
    //     let pubkey = PublicKey::from_hex(&recipient_pubkey).map_err(|e| ClientError::InvalidKey {
    //         message: format!("Invalid recipient public key: {}", e),
    //     })?;

    //     let event_id = self.rt.block_on(async {
    //         self.core.send_encrypted_message(&pubkey, &content).await
    //     }).map_err(|e| ClientError::EncryptionError {
    //         message: e.to_string(),
    //     })?;

    //     Ok(event_id.to_hex())
    // }

    pub fn get_encrypted_messages(&self) -> Result<Vec<EncryptedMessage>, ClientError> {
        let events = self.runtime.block_on(async {
            self.core.get_encrypted_messages().await
        }).map_err(|e| ClientError::Generic {
            message: e.to_string(),
        })?;

        let messages = events.into_iter().map(|event| EncryptedMessage {
            id: event.id.to_hex(),
            content: event.content,
            sender: event.pubkey.to_hex(),
            created_at: event.created_at.as_u64(),
        }).collect();

        Ok(messages)
    }

    // TODO: Fix type compatibility
    // pub fn decrypt_message(&self, sender_pubkey: String, encrypted_content: String) -> Result<String, ClientError> {
    //     let pubkey = PublicKey::from_hex(&sender_pubkey).map_err(|e| ClientError::InvalidKey {
    //         message: format!("Invalid sender public key: {}", e),
    //     })?;

    //     let decrypted = self.core.decrypt_message(&pubkey, &encrypted_content).map_err(|e| ClientError::EncryptionError {
    //         message: e.to_string(),
    //     })?;

    //     Ok(decrypted)
    // }

    pub fn create_group(&self, group_name: String, member_pubkeys: Vec<String>) -> Result<String, ClientError> {
        let pubkeys: Result<Vec<whitenoise::PublicKey>, _> = member_pubkeys.iter()
            .map(|hex| whitenoise::PublicKey::from_hex(hex))
            .collect();
        
        let pubkeys = pubkeys.map_err(|e| ClientError::InvalidKey {
            message: format!("Invalid member public key: {}", e),
        })?;

        let group_id = self.runtime.block_on(async {
            self.core.create_group(&group_name, pubkeys).await
        }).map_err(|e| ClientError::Generic {
            message: e.to_string(),
        })?;

        Ok(group_id)
    }

    pub fn send_group_message(&self, group_id: String, content: String, member_pubkeys: Vec<String>) -> Result<String, ClientError> {
        let pubkeys: Result<Vec<whitenoise::PublicKey>, _> = member_pubkeys.iter()
            .map(|hex| whitenoise::PublicKey::from_hex(hex))
            .collect();
        
        let _pubkeys = pubkeys.map_err(|e| ClientError::InvalidKey {
            message: format!("Invalid member public key: {}", e),
        })?;

        let message_id = self.runtime.block_on(async {
            self.core.send_group_message(&group_id, &content, &[]).await
        }).map_err(|e| ClientError::Generic {
            message: e.to_string(),
        })?;

        Ok(message_id)
    }

    pub fn fetch_groups(&self) -> Result<Vec<String>, ClientError> {
        self.runtime.block_on(async {
            self.core.fetch_groups().await
        }).map_err(|e| ClientError::Generic {
            message: e.to_string(),
        })
    }

    pub fn fetch_group_messages(&self, group_id: String) -> Result<Vec<String>, ClientError> {
        self.runtime.block_on(async {
            self.core.fetch_group_messages(&group_id).await
        }).map_err(|e| ClientError::Generic {
            message: e.to_string(),
        })
    }

    pub fn add_members_to_group(&self, group_id: String, new_member_pubkeys: Vec<String>) -> Result<(), ClientError> {
        let pubkeys: Result<Vec<whitenoise::PublicKey>, _> = new_member_pubkeys.iter()
            .map(|hex| whitenoise::PublicKey::from_hex(hex))
            .collect();
        
        let pubkeys = pubkeys.map_err(|e| ClientError::InvalidKey {
            message: format!("Invalid member public key: {}", e),
        })?;

        self.runtime.block_on(async {
            self.core.add_members_to_group(&group_id, pubkeys).await
        }).map_err(|e| ClientError::Generic {
            message: e.to_string(),
        })
    }

    pub fn remove_members_from_group(&self, group_id: String, member_pubkeys_to_remove: Vec<String>) -> Result<(), ClientError> {
        let pubkeys: Result<Vec<whitenoise::PublicKey>, _> = member_pubkeys_to_remove.iter()
            .map(|hex| whitenoise::PublicKey::from_hex(hex))
            .collect();
        
        let pubkeys = pubkeys.map_err(|e| ClientError::InvalidKey {
            message: format!("Invalid member public key: {}", e),
        })?;

        self.runtime.block_on(async {
            self.core.remove_members_from_group(&group_id, pubkeys).await
        }).map_err(|e| ClientError::Generic {
            message: e.to_string(),
        })
    }
}

// Generate UniFFI bindings
uniffi::setup_scaffolding!();