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
}

pub type Result<T> = std::result::Result<T, DialogError>;