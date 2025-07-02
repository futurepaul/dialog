use anyhow::Result;
use nostr_sdk::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a test keypair (no keychain)
    let keys = Keys::generate();
    println!("Test Client Public Key: {}", keys.public_key());
    
    // Connect to local relay
    let client = Client::new(&keys);
    client.add_relay("ws://127.0.0.1:7979", None).await?;
    client.connect().await;
    
    // Wait a moment for connection
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Get CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <message>", args[0]);
        return Ok(());
    }
    
    let message = args[1..].join(" ");
    
    // Publish a note
    println!("Publishing note: {}", message);
    let event = EventBuilder::new_text_note(&message, &[])
        .to_event(&keys)?;
    
    client.send_event(event.clone()).await?;
    println!("Published event: {}", event.id);
    
    // Wait a moment then fetch recent notes
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    println!("\nFetching recent notes...");
    let filter = Filter::new()
        .kind(Kind::TextNote)
        .limit(10);
    
    let events = client.get_events_of(vec![filter], Some(Duration::from_secs(5))).await?;
    
    println!("Found {} notes:", events.len());
    for event in events.iter().take(5) {
        println!("  {}: {} (by {})", 
            event.created_at, 
            event.content,
            event.pubkey.to_string()[..16].to_string() + "..."
        );
    }
    
    Ok(())
}