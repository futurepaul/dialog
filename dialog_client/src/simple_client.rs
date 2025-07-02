use anyhow::Result;
use nostr_sdk::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

pub struct SimpleDialogClient {
    client: Option<Client>,
    keys: Keys,
}

impl SimpleDialogClient {
    pub fn new() -> Self {
        let keys = Keys::generate();
        Self {
            client: None,
            keys,
        }
    }
    
    pub fn get_public_key(&self) -> String {
        self.keys.public_key().to_string()
    }
    
    pub async fn connect_to_relay(&mut self, relay_url: &str) -> Result<()> {
        let client = Client::new(self.keys.clone());
        client.add_relay(relay_url).await?;
        client.connect().await;
        
        // Wait for connection
        sleep(Duration::from_millis(500)).await;
        
        self.client = Some(client);
        Ok(())
    }
    
    pub async fn publish_note(&self, content: &str) -> Result<String> {
        if let Some(client) = &self.client {
            let event = EventBuilder::text_note(content)
                .sign(&self.keys).await?;
            
            client.send_event(&event).await?;
            Ok(event.id.to_string())
        } else {
            Err(anyhow::anyhow!("Client not connected"))
        }
    }
    
    pub async fn get_notes(&self, limit: Option<usize>) -> Result<Vec<SimpleNote>> {
        if let Some(client) = &self.client {
            let filter = Filter::new()
                .kind(Kind::TextNote)
                .limit(limit.unwrap_or(20));
            
            let timeout = Duration::from_secs(5);
            let events = client.fetch_events(filter, timeout).await?;
            
            let notes = events.iter()
                .map(|event| SimpleNote {
                    content: event.content.clone(),
                    pubkey: event.pubkey.to_string(),
                    created_at: event.created_at.as_u64(),
                    event_id: event.id.to_string(),
                })
                .collect();
            
            Ok(notes)
        } else {
            Err(anyhow::anyhow!("Client not connected"))
        }
    }
}

#[derive(Clone)]
pub struct SimpleNote {
    pub content: String,
    pub pubkey: String,
    pub created_at: u64,
    pub event_id: String,
}