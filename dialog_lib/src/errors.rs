use thiserror::Error;

#[derive(Error, Debug)]
pub enum DialogError {
    #[error("Contact not found: {0}")]
    ContactNotFound(String),
    
    #[error("Conversation not found: {0}")]
    ConversationNotFound(String),
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("Invalid pubkey format: {0}")]
    InvalidPubkey(String),
    
    #[error("MLS operation failed: {0}")]
    MlsError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("General error: {0}")]
    General(#[from] Box<dyn std::error::Error + Send + Sync>),
    
    #[error("Nostr MLS error: {0}")]
    NostrMls(#[from] nostr_mls::Error),
    
    #[error("Nostr SDK error: {0}")]
    NostrSdk(#[from] nostr_sdk::client::Error),
}

pub type Result<T> = std::result::Result<T, DialogError>;