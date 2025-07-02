use anyhow::Result;
use clap::{Parser, Subcommand};
use dialog_client::{DialogClient, PublicKey};
// Note: Using whitenoise types now
// use nostr_sdk::prelude::*;
use tracing::info;
use tokio::time::{sleep, Duration};

#[derive(Parser)]
#[command(name = "dialog_cli")]
#[command(about = "A CLI tool for testing dialog client and relay functionality")]
struct Cli {
    /// Use a specific secret key (hex format) instead of generating a new one
    #[arg(long, global = true)]
    key: Option<String>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Publish a text note to the relay
    Publish {
        /// The relay URL to connect to
        #[arg(short, long, default_value = "ws://127.0.0.1:7979")]
        relay: String,
        /// The text content of the note
        message: String,
    },
    /// Fetch and display notes from the relay
    Fetch {
        /// The relay URL to connect to
        #[arg(short, long, default_value = "ws://127.0.0.1:7979")]
        relay: String,
        /// Number of notes to fetch
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Interactive test: publish then fetch notes
    Test {
        /// The relay URL to connect to
        #[arg(short, long, default_value = "ws://127.0.0.1:7979")]
        relay: String,
        /// Test message to publish
        #[arg(short, long, default_value = "Hello from dialog_cli!")]
        message: String,
    },
    /// Send an encrypted message to a recipient
    SendEncrypted {
        /// The relay URL to connect to
        #[arg(short, long, default_value = "ws://127.0.0.1:7979")]
        relay: String,
        /// Recipient's public key (hex format)
        #[arg(short = 'p', long)]
        recipient: String,
        /// The encrypted message content
        message: String,
    },
    /// Fetch and decrypt encrypted messages
    FetchEncrypted {
        /// The relay URL to connect to
        #[arg(short, long, default_value = "ws://127.0.0.1:7979")]
        relay: String,
        /// Number of messages to fetch
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Create a new group
    CreateGroup {
        /// The relay URL to connect to
        #[arg(short, long, default_value = "ws://127.0.0.1:7979")]
        relay: String,
        /// Group name
        #[arg(short, long)]
        name: String,
        /// Member public keys (comma-separated hex format)
        #[arg(short, long)]
        members: String,
    },
    /// Send a message to a group
    SendGroupMessage {
        /// The relay URL to connect to
        #[arg(short, long, default_value = "ws://127.0.0.1:7979")]
        relay: String,
        /// Group identifier
        #[arg(short, long)]
        group_id: String,
        /// The message content
        message: String,
    },
    /// Show user's public key
    ShowKey,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    let key = cli.key.as_deref();
    
    match cli.command {
        Commands::Publish { relay, message } => {
            publish_note(&relay, &message, key).await?;
        }
        Commands::Fetch { relay, limit } => {
            fetch_notes(&relay, limit, key).await?;
        }
        Commands::Test { relay, message } => {
            test_publish_and_fetch(&relay, &message, key).await?;
        }
        Commands::SendEncrypted { relay, recipient, message } => {
            send_encrypted_message(&relay, &recipient, &message, key).await?;
        }
        Commands::FetchEncrypted { relay, limit } => {
            fetch_encrypted_messages(&relay, limit, key).await?;
        }
        Commands::CreateGroup { relay, name, members } => {
            create_group(&relay, &name, &members, key).await?;
        }
        Commands::SendGroupMessage { relay, group_id, message } => {
            send_group_message(&relay, &group_id, &message, key).await?;
        }
        Commands::ShowKey => {
            show_key(key).await?;
        }
    }
    
    Ok(())
}

async fn create_client(key: Option<&str>) -> Result<DialogClient> {
    match key {
        Some(secret_key_hex) => DialogClient::new_with_key(secret_key_hex).await,
        None => DialogClient::new().await,
    }
}

async fn publish_note(relay_url: &str, message: &str, key: Option<&str>) -> Result<()> {
    info!("Creating client and connecting to relay...");
    
    let client = create_client(key).await?;
    client.connect_to_relay(relay_url).await?;
    
    if let Some(pubkey) = client.get_public_key() {
        info!("Client pubkey: {}", pubkey);
    } else {
        info!("Client pubkey: Not available");
    }
    
    // Wait a moment for connection to establish
    sleep(Duration::from_secs(1)).await;
    
    let event_id = client.publish_note(message).await?;
    println!("✅ Published note with ID: {}", event_id);
    
    Ok(())
}

async fn fetch_notes(relay_url: &str, limit: usize, key: Option<&str>) -> Result<()> {
    info!("Creating client and connecting to relay...");
    
    let client = create_client(key).await?;
    client.connect_to_relay(relay_url).await?;
    
    // Wait a moment for connection to establish
    sleep(Duration::from_secs(1)).await;
    
    let notes = client.get_notes(Some(limit)).await?;
    
    if notes.is_empty() {
        println!("📝 No notes found on relay");
    } else {
        println!("📝 Found {} notes:", notes.len());
        for note in notes {
            println!("  🗒️  [{}] {}: {}", 
                note.created_at, 
                note.pubkey.to_hex()[..8].to_string() + "...",
                note.content
            );
        }
    }
    
    Ok(())
}

async fn test_publish_and_fetch(relay_url: &str, message: &str, key: Option<&str>) -> Result<()> {
    println!("🧪 Starting publish and fetch test...");
    
    // Publish a note
    println!("1️⃣ Publishing note...");
    publish_note(relay_url, message, key).await?;
    
    // Wait a moment
    println!("⏱️ Waiting 2 seconds...");
    sleep(Duration::from_secs(2)).await;
    
    // Fetch notes
    println!("2️⃣ Fetching notes...");
    fetch_notes(relay_url, 5, key).await?;
    
    println!("✅ Test completed!");
    
    Ok(())
}

async fn send_encrypted_message(relay_url: &str, recipient_hex: &str, message: &str, key: Option<&str>) -> Result<()> {
    info!("Creating client and connecting to relay...");
    
    let client = create_client(key).await?;
    client.connect_to_relay(relay_url).await?;
    
    // Parse recipient public key
    let recipient_pubkey = PublicKey::from_hex(recipient_hex)?;
    
    if let Some(pubkey) = client.get_public_key() {
        info!("Client pubkey: {}", pubkey);
    } else {
        info!("Client pubkey: Not available");
    }
    info!("Recipient pubkey: {}", recipient_pubkey);
    
    // Wait a moment for connection to establish
    sleep(Duration::from_secs(1)).await;
    
    let event_id = client.send_encrypted_message(&recipient_pubkey, message).await?;
    println!("🔐 Sent encrypted message with ID: {}", event_id);
    
    Ok(())
}

async fn fetch_encrypted_messages(relay_url: &str, limit: usize, key: Option<&str>) -> Result<()> {
    info!("Creating client and connecting to relay...");
    
    let client = create_client(key).await?;
    client.connect_to_relay(relay_url).await?;
    
    if let Some(pubkey) = client.get_public_key() {
        info!("Client pubkey: {}", pubkey);
    } else {
        info!("Client pubkey: Not available");
    }
    
    // Wait a moment for connection to establish
    sleep(Duration::from_secs(1)).await;
    
    let messages = client.get_encrypted_messages().await?;
    
    if messages.is_empty() {
        println!("🔐 No encrypted messages found");
    } else {
        println!("🔐 Found {} encrypted messages:", messages.len());
        for msg in messages.iter().take(limit) {
            // Try to decrypt the message
            match client.decrypt_message(&msg.pubkey, &msg.content) {
                Ok(decrypted) => {
                    println!("  📨 [{}] from {}: {}", 
                        msg.created_at,
                        msg.pubkey.to_hex()[..8].to_string() + "...",
                        decrypted
                    );
                }
                Err(_) => {
                    println!("  🔒 [{}] from {}: [Failed to decrypt]", 
                        msg.created_at,
                        msg.pubkey.to_hex()[..8].to_string() + "..."
                    );
                }
            }
        }
    }
    
    Ok(())
}

async fn create_group(relay_url: &str, name: &str, members_str: &str, key: Option<&str>) -> Result<()> {
    info!("Creating client and connecting to relay...");
    
    let client = create_client(key).await?;
    client.connect_to_relay(relay_url).await?;
    
    // Parse member public keys
    let members: Result<Vec<PublicKey>, _> = members_str
        .split(',')
        .map(|hex| PublicKey::parse(hex.trim()))
        .collect();
    let members = members?;
    
    if let Some(pubkey) = client.get_public_key() {
        info!("Client pubkey: {}", pubkey);
    } else {
        info!("Client pubkey: Not available");
    }
    info!("Creating group '{}' with {} members", name, members.len());
    
    // Wait a moment for connection to establish
    sleep(Duration::from_secs(1)).await;
    
    let event_id = client.create_group(name, members).await?;
    println!("👥 Created group '{}' with ID: {}", name, event_id);
    
    Ok(())
}

async fn send_group_message(relay_url: &str, group_id: &str, message: &str, key: Option<&str>) -> Result<()> {
    info!("Creating client and connecting to relay...");
    
    let client = create_client(key).await?;
    client.connect_to_relay(relay_url).await?;
    
    if let Some(pubkey) = client.get_public_key() {
        info!("Client pubkey: {}", pubkey);
    } else {
        info!("Client pubkey: Not available");
    }
    
    // Wait a moment for connection to establish
    sleep(Duration::from_secs(1)).await;
    
    // For now, we'll send to an empty members list since we don't have group state management
    let event_id = client.send_group_message(group_id, message, &[]).await?;
    println!("👥 Sent group message to '{}' with ID: {}", group_id, event_id);
    
    Ok(())
}

async fn show_key(key: Option<&str>) -> Result<()> {
    let client = create_client(key).await?;
    if let Some(pubkey) = client.get_public_key() {
        println!("🔑 Your public key: {}", pubkey);
    } else {
        println!("🔑 Your public key: Not available");
    }
    match client.get_secret_key_hex().await {
        Ok(Some(secret)) => println!("🗝️  Your secret key: {}", secret),
        Ok(None) => println!("🗝️  Your secret key: Not available"),
        Err(e) => println!("🗝️  Your secret key: Error: {}", e),
    }
    Ok(())
}
