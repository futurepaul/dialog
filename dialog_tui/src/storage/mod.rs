use std::collections::HashMap;
use std::path::PathBuf;
use nostr_sdk::{Keys, PublicKey};
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_mls::NostrMls;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use crate::error::{DialogTuiError, Result};
use crate::model::{ChatMessage, Conversation, ConversationId, Contact, ContactId};

pub struct PerPubkeyStorage {
    home_dir: PathBuf,
    current_pubkey: Option<PublicKey>,
    message_db: Option<SqlitePool>,
    nostr_mls: Option<NostrMls<NostrMlsSqliteStorage>>,
}

impl PerPubkeyStorage {
    pub fn new() -> Result<Self> {
        let home_dir = home::home_dir()
            .ok_or_else(|| DialogTuiError::Storage { 
                message: "Could not determine home directory".to_string() 
            })?
            .join(".local/share/dialog_tui");

        std::fs::create_dir_all(&home_dir)?;

        Ok(Self {
            home_dir,
            current_pubkey: None,
            message_db: None,
            nostr_mls: None,
        })
    }

    pub async fn init_for_pubkey(&mut self, keys: &Keys) -> Result<()> {
        let pubkey = keys.public_key();
        let pubkey_hex = pubkey.to_hex();
        
        // Create pubkey-specific directory
        let pubkey_dir = self.home_dir.join(&pubkey_hex);
        std::fs::create_dir_all(&pubkey_dir)?;

        // Initialize message database
        let message_db_path = pubkey_dir.join("messages.db");
        
        // Ensure the directory exists with proper permissions
        if !pubkey_dir.exists() {
            std::fs::create_dir_all(&pubkey_dir)?;
        }
        
        let connection_string = format!("sqlite:{}?mode=rwc", message_db_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await?;

        // Create tables if they don't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS contacts (
                id TEXT PRIMARY KEY,
                pubkey TEXT NOT NULL UNIQUE,
                petname TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
            "#
        ).execute(&pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                group_id BLOB,
                participants TEXT NOT NULL,
                last_message_time TEXT,
                unread_count INTEGER DEFAULT 0
            )
            "#
        ).execute(&pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                sender TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                is_own BOOLEAN NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations (id)
            )
            "#
        ).execute(&pool).await?;

        // Initialize NostrMls
        let mls_db_path = pubkey_dir.join("mls.db");
        let mls_storage = NostrMlsSqliteStorage::new(mls_db_path)?;
        let nostr_mls = NostrMls::new(mls_storage);

        self.current_pubkey = Some(pubkey);
        self.message_db = Some(pool);
        self.nostr_mls = Some(nostr_mls);

        Ok(())
    }

    pub async fn save_message(&self, message: &ChatMessage) -> Result<()> {
        let pool = self.message_db.as_ref()
            .ok_or_else(|| DialogTuiError::Storage { 
                message: "Storage not initialized".to_string() 
            })?;

        sqlx::query(
            "INSERT OR REPLACE INTO messages (id, conversation_id, sender, content, timestamp, is_own)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
        .bind(&message.id)
        .bind(&message.conversation_id)
        .bind(message.sender.to_hex())
        .bind(&message.content)
        .bind(message.timestamp.to_rfc3339())
        .bind(message.is_own)
        .execute(pool).await?;

        Ok(())
    }

    pub async fn load_messages(&self, conversation_id: &ConversationId) -> Result<Vec<ChatMessage>> {
        let pool = self.message_db.as_ref()
            .ok_or_else(|| DialogTuiError::Storage { 
                message: "Storage not initialized".to_string() 
            })?;

        let rows = sqlx::query(
            "SELECT id, conversation_id, sender, content, timestamp, is_own 
             FROM messages 
             WHERE conversation_id = ?1 
             ORDER BY timestamp ASC"
        )
        .bind(conversation_id)
        .fetch_all(pool).await?;

        let mut messages = Vec::new();
        for row in rows {
            use sqlx::Row;
            
            let timestamp_str: String = row.try_get("timestamp")?;
            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| DialogTuiError::Storage { 
                    message: format!("Invalid timestamp: {}", e) 
                })?
                .with_timezone(&chrono::Utc);

            let sender_hex: String = row.try_get("sender")?;
            let sender = PublicKey::from_hex(&sender_hex)?;

            messages.push(ChatMessage {
                id: row.try_get("id")?,
                conversation_id: row.try_get("conversation_id")?,
                sender,
                content: row.try_get("content")?,
                timestamp,
                is_own: row.try_get::<i32, _>("is_own")? != 0,
            });
        }

        Ok(messages)
    }

    pub async fn save_conversation(&self, conversation: &Conversation) -> Result<()> {
        let pool = self.message_db.as_ref()
            .ok_or_else(|| DialogTuiError::Storage { 
                message: "Storage not initialized".to_string() 
            })?;

        let participants_json = serde_json::to_string(
            &conversation.participants.iter().map(|p| p.to_hex()).collect::<Vec<_>>()
        ).map_err(|e| DialogTuiError::Storage { 
            message: format!("Failed to serialize participants: {}", e) 
        })?;

        let group_id_bytes = conversation.group_id.as_ref().map(|id| id.as_slice());
        let last_message_time = conversation.last_message_time.as_ref().map(|t| t.to_rfc3339());

        sqlx::query(
            "INSERT OR REPLACE INTO conversations (id, name, group_id, participants, last_message_time, unread_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
        .bind(&conversation.id)
        .bind(&conversation.name)
        .bind(group_id_bytes)
        .bind(participants_json)
        .bind(last_message_time)
        .bind(conversation.unread_count as i64)
        .execute(pool).await?;

        Ok(())
    }

    pub async fn load_conversations(&self) -> Result<HashMap<ConversationId, Conversation>> {
        let pool = self.message_db.as_ref()
            .ok_or_else(|| DialogTuiError::Storage { 
                message: "Storage not initialized".to_string() 
            })?;

        let rows = sqlx::query(
            "SELECT id, name, group_id, participants, last_message_time, unread_count 
             FROM conversations 
             ORDER BY last_message_time DESC"
        ).fetch_all(pool).await?;

        let mut conversations = HashMap::new();
        for row in rows {
            use sqlx::Row;
            
            let participants_json: String = row.try_get("participants")?;
            let participants_vec: Vec<String> = serde_json::from_str(&participants_json)
                .map_err(|e| DialogTuiError::Storage { 
                    message: format!("Failed to deserialize participants: {}", e) 
                })?;

            let participants: Result<Vec<PublicKey>> = participants_vec
                .into_iter()
                .map(|hex| PublicKey::from_hex(&hex).map_err(|e| e.into()))
                .collect();
            let participants = participants?;

            let group_id_bytes: Option<Vec<u8>> = row.try_get("group_id")?;
            let group_id = group_id_bytes.as_ref()
                .map(|bytes| nostr_mls::prelude::GroupId::from_slice(bytes));

            let last_message_time_str: Option<String> = row.try_get("last_message_time")?;
            let last_message_time = last_message_time_str.as_ref()
                .map(|s| chrono::DateTime::parse_from_rfc3339(s))
                .transpose()
                .map_err(|e| DialogTuiError::Storage { 
                    message: format!("Invalid timestamp: {}", e) 
                })?
                .map(|dt| dt.with_timezone(&chrono::Utc));

            let id: String = row.try_get("id")?;
            let conversation = Conversation {
                id: id.clone(),
                group_id,
                name: row.try_get("name")?,
                participants,
                last_message_time,
                unread_count: row.try_get::<i64, _>("unread_count")? as u32,
            };

            conversations.insert(id, conversation);
        }

        Ok(conversations)
    }

    pub async fn save_contact(&self, contact: &Contact) -> Result<()> {
        let pool = self.message_db.as_ref()
            .ok_or_else(|| DialogTuiError::Storage { 
                message: "Storage not initialized".to_string() 
            })?;

        sqlx::query(
            "INSERT OR REPLACE INTO contacts (id, pubkey, petname, created_at)
             VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(&contact.id)
        .bind(contact.pubkey.to_hex())
        .bind(&contact.petname)
        .bind(contact.created_at.to_rfc3339())
        .execute(pool).await?;

        Ok(())
    }

    pub async fn load_contacts(&self) -> Result<HashMap<ContactId, Contact>> {
        let pool = self.message_db.as_ref()
            .ok_or_else(|| DialogTuiError::Storage { 
                message: "Storage not initialized".to_string() 
            })?;

        let rows = sqlx::query(
            "SELECT id, pubkey, petname, created_at 
             FROM contacts 
             ORDER BY created_at DESC"
        ).fetch_all(pool).await?;

        let mut contacts = HashMap::new();
        for row in rows {
            use sqlx::Row;
            
            let pubkey_hex: String = row.try_get("pubkey")?;
            let pubkey = PublicKey::from_hex(&pubkey_hex)?;
            
            let created_at_str: String = row.try_get("created_at")?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| DialogTuiError::Storage { 
                    message: format!("Invalid timestamp: {}", e) 
                })?
                .with_timezone(&chrono::Utc);

            let id: String = row.try_get("id")?;
            let contact = Contact {
                id: id.clone(),
                pubkey,
                petname: row.try_get("petname")?,
                created_at,
            };

            contacts.insert(id, contact);
        }

        Ok(contacts)
    }

    pub fn get_nostr_mls(&self) -> Option<&NostrMls<NostrMlsSqliteStorage>> {
        self.nostr_mls.as_ref()
    }
    
    pub fn get_nostr_mls_mut(&mut self) -> Option<&mut NostrMls<NostrMlsSqliteStorage>> {
        self.nostr_mls.as_mut()
    }
    
    pub async fn clear_all_data(&mut self) -> Result<()> {
        if let Some(pool) = &self.message_db {
            sqlx::query("DELETE FROM messages").execute(pool).await?;
            sqlx::query("DELETE FROM conversations").execute(pool).await?;
            sqlx::query("DELETE FROM contacts").execute(pool).await?;
        }
        Ok(())
    }
    
    pub async fn clear_contacts(&mut self) -> Result<()> {
        if let Some(pool) = &self.message_db {
            sqlx::query("DELETE FROM contacts").execute(pool).await?;
        }
        Ok(())
    }
    
    pub async fn clear_conversations(&mut self) -> Result<()> {
        if let Some(pool) = &self.message_db {
            sqlx::query("DELETE FROM messages").execute(pool).await?;
            sqlx::query("DELETE FROM conversations").execute(pool).await?;
        }
        Ok(())
    }
    
    pub async fn reset_mls_state(&mut self, keys: &Keys) -> Result<()> {
        // Reset MLS state (this will force re-initialization)
        self.nostr_mls = None;
        self.init_for_pubkey(keys).await?;
        Ok(())
    }
}