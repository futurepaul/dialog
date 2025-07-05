use thiserror::Error;

#[derive(Error, Debug)]
pub enum DialogTuiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Nostr MLS error: {0}")]
    NostrMls(#[from] nostr_mls::Error),

    #[error("Nostr SDK error: {0}")]
    NostrSdk(#[from] nostr_sdk::client::Error),

    #[error("Nostr key error: {0}")]
    NostrKey(#[from] nostr_sdk::key::Error),

    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),

    #[error("Hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("Tracing error: {0}")]
    Tracing(#[from] tracing::subscriber::SetGlobalDefaultError),

    #[error("MLS storage error: {0}")]
    MlsStorage(#[from] nostr_mls_sqlite_storage::error::Error),

    #[error("Send error: {0}")]
    Send(String),

    #[error("Storage error: {message}")]
    Storage { message: String },

    #[error("UI error: {message}")]
    Ui { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
}

pub type Result<T> = std::result::Result<T, DialogTuiError>;