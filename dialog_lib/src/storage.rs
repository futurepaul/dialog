use nostr_mls::prelude::*;
use nostr_mls::groups::{GroupResult, UpdateGroupResult};
use nostr_mls::messages::MessageProcessingResult;
use nostr_mls_storage::groups::types as group_types;
use nostr_mls_storage::messages::types as message_types;
use nostr_mls_storage::welcomes::types as welcome_types;
use openmls::prelude::KeyPackage;
use nostr_mls_memory_storage::NostrMlsMemoryStorage;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum StorageBackend {
    Memory,
    Sqlite { path: PathBuf },
}

impl Default for StorageBackend {
    fn default() -> Self {
        StorageBackend::Memory
    }
}

pub enum NostrMlsStorage {
    Memory(Arc<RwLock<NostrMls<NostrMlsMemoryStorage>>>),
    Sqlite(Arc<tokio::sync::Mutex<NostrMls<NostrMlsSqliteStorage>>>),
}

impl std::fmt::Debug for NostrMlsStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Memory(_) => write!(f, "NostrMlsStorage::Memory"),
            Self::Sqlite(_) => write!(f, "NostrMlsStorage::Sqlite"),
        }
    }
}

// Macro to implement delegating methods to both storage types
macro_rules! delegate_nostr_mls {
    ($self:expr, $method:ident, $($arg:expr),*) => {
        match $self {
            NostrMlsStorage::Memory(mls) => {
                let mls = mls.read().await;
                mls.$method($($arg),*)
            },
            NostrMlsStorage::Sqlite(mls) => {
                let mls = mls.lock().await;
                mls.$method($($arg),*)
            },
        }
    };
}

impl NostrMlsStorage {
    pub async fn new(backend: StorageBackend) -> Result<Self, crate::errors::DialogError> {
        match backend {
            StorageBackend::Memory => {
                let storage = NostrMlsMemoryStorage::default();
                let nostr_mls = NostrMls::new(storage);
                Ok(NostrMlsStorage::Memory(Arc::new(RwLock::new(nostr_mls))))
            }
            StorageBackend::Sqlite { path } => {
                // Ensure parent directory exists
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let storage = NostrMlsSqliteStorage::new(path)
                    .map_err(|e| crate::errors::DialogError::Storage(e.to_string()))?;
                let nostr_mls = NostrMls::new(storage);
                Ok(NostrMlsStorage::Sqlite(Arc::new(tokio::sync::Mutex::new(nostr_mls))))
            }
        }
    }

    // Delegate all NostrMls methods
    pub async fn get_groups(&self) -> Result<Vec<group_types::Group>, nostr_mls::Error> {
        delegate_nostr_mls!(self, get_groups,)
    }

    pub async fn create_key_package_for_event(
        &self,
        pubkey: &PublicKey,
        relays: impl IntoIterator<Item = RelayUrl>,
    ) -> Result<(String, Vec<Tag>), nostr_mls::Error> {
        match self {
            NostrMlsStorage::Memory(mls) => {
                let mls = mls.read().await;
                let (content, tags) = mls.create_key_package_for_event(pubkey, relays)?;
                Ok((content, tags.to_vec()))
            },
            NostrMlsStorage::Sqlite(mls) => {
                let mls = mls.lock().await;
                let (content, tags) = mls.create_key_package_for_event(pubkey, relays)?;
                Ok((content, tags.to_vec()))
            },
        }
    }

    pub async fn parse_key_package(&self, event: &Event) -> Result<KeyPackage, nostr_mls::Error> {
        delegate_nostr_mls!(self, parse_key_package, event)
    }

    pub async fn create_group(
        &self,
        pubkey: &PublicKey,
        key_packages: Vec<Event>,
        admins: Vec<PublicKey>,
        config: NostrGroupConfigData,
    ) -> Result<GroupResult, nostr_mls::Error> {
        delegate_nostr_mls!(self, create_group, pubkey, key_packages, admins, config)
    }

    pub async fn process_message(&self, event: &Event) -> Result<MessageProcessingResult, nostr_mls::Error> {
        delegate_nostr_mls!(self, process_message, event)
    }

    pub async fn process_welcome(&self, gift_wrap_id: &EventId, rumor: &UnsignedEvent) -> Result<(), nostr_mls::Error> {
        match self {
            NostrMlsStorage::Memory(mls) => {
                let mls = mls.read().await;
                mls.process_welcome(gift_wrap_id, rumor).map(|_| ())
            },
            NostrMlsStorage::Sqlite(mls) => {
                let mls = mls.lock().await;
                mls.process_welcome(gift_wrap_id, rumor).map(|_| ())
            },
        }
    }

    pub async fn get_pending_welcomes(&self) -> Result<Vec<welcome_types::Welcome>, nostr_mls::Error> {
        delegate_nostr_mls!(self, get_pending_welcomes,)
    }

    pub async fn accept_welcome(&self, welcome: &welcome_types::Welcome) -> Result<(), nostr_mls::Error> {
        delegate_nostr_mls!(self, accept_welcome, welcome)
    }

    pub async fn create_message(&self, group_id: &GroupId, rumor: UnsignedEvent) -> Result<Event, nostr_mls::Error> {
        delegate_nostr_mls!(self, create_message, group_id, rumor)
    }

    pub async fn get_messages(&self, group_id: &GroupId) -> Result<Vec<message_types::Message>, nostr_mls::Error> {
        delegate_nostr_mls!(self, get_messages, group_id)
    }

    pub async fn get_members(&self, group_id: &GroupId) -> Result<std::collections::BTreeSet<PublicKey>, nostr_mls::Error> {
        delegate_nostr_mls!(self, get_members, group_id)
    }

    pub async fn add_members(&self, group_id: &GroupId, key_packages: Vec<Event>) -> Result<UpdateGroupResult, nostr_mls::Error> {
        delegate_nostr_mls!(self, add_members, group_id, &key_packages)
    }

    pub async fn remove_members(&self, group_id: &GroupId, members: Vec<PublicKey>) -> Result<UpdateGroupResult, nostr_mls::Error> {
        delegate_nostr_mls!(self, remove_members, group_id, &members)
    }
}