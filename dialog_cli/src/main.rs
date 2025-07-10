use clap::{Arg, Command};
use dialog_lib::{DialogLib, StorageBackend, Keys, PublicKey, GroupId, hex, DialogConfig};
use dotenv::{dotenv, from_path};
use nostr_sdk::prelude::*;
use std::{env, path::PathBuf, fs};
use thiserror::Error;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Error, Debug)]
enum DialogError {
    #[error("Dialog lib error: {0}")]
    DialogLib(#[from] dialog_lib::DialogError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),
    
    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    
    #[error("Tracing error: {0}")]
    Tracing(#[from] tracing::subscriber::SetGlobalDefaultError),
    
    #[error("Key error: {0}")]
    Key(#[from] nostr_sdk::key::Error),
    
    #[error("General error: {0}")]
    General(String),
}

fn find_and_load_env() {
    // First try the standard dotenv() which looks for .env in current dir
    dotenv().ok();

    // Then walk up the directory tree looking for .env.local
    let mut current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    loop {
        let env_file = current_dir.join(".env.local");
        if env_file.exists() {
            from_path(&env_file).ok();
            break;
        }

        // Move up one directory
        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break, // Reached filesystem root
        }
    }
}

fn get_secret_key(key_arg: &str) -> Result<String, DialogError> {
    match key_arg {
        "bob" => {
            find_and_load_env();
            Ok(env::var("BOB_SK_HEX")?)
        }
        "alice" => {
            find_and_load_env();
            Ok(env::var("ALICE_SK_HEX")?)
        }
        hex_key => {
            // Validate that it looks like a hex string
            if hex_key.len() == 64 && hex_key.chars().all(|c| c.is_ascii_hexdigit()) {
                Ok(hex_key.to_string())
            } else {
                Err(DialogError::General("Key must be either 'bob', 'alice', or a 64-character hex string".into()))
            }
        }
    }
}

async fn create_dialog_lib(sk_hex: &str, relay_url: &str) -> Result<DialogLib, DialogError> {
    let keys = Keys::parse(sk_hex)?;
    let data_dir = env::current_dir()?.join(".dialog_cli_data");
    let identity_dir = data_dir.join(keys.public_key().to_hex());
    fs::create_dir_all(&identity_dir)?;
    let db_path = identity_dir.join("mls.db");
    
    let storage_backend = StorageBackend::Sqlite { path: db_path };
    
    Ok(DialogLib::new_with_storage(keys, relay_url, storage_backend).await?)
}

#[tokio::main]
async fn main() -> Result<(), DialogError> {
    // Set up tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Set up command line argument parsing using clap builder pattern
    let matches = Command::new("dialog_cli")
        .version("0.1.0")
        .about("Dialog CLI for Nostr MLS")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("publish-key")
                .about("Generates and publishes a key package for the user")
                .arg(
                    Arg::new("key")
                        .long("key")
                        .value_name("KEY")
                        .help("Secret key for identity: 'bob', 'alice', or hex string")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("create-group")
                .about("Creates a new group and invites a counterparty")
                .arg(
                    Arg::new("key")
                        .long("key")
                        .value_name("KEY")
                        .help("Secret key for your identity")
                        .required(true),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .help("Name of the group")
                        .required(true),
                )
                .arg(
                    Arg::new("counterparty")
                        .long("counterparty")
                        .help("Public key of the counterparty to invite")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("send-message")
                .about("Sends a message to a group")
                .arg(
                    Arg::new("key")
                        .long("key")
                        .value_name("KEY")
                        .help("Secret key for your identity")
                        .required(true),
                )
                .arg(
                    Arg::new("group-id")
                        .long("group-id")
                        .help("Hex-encoded ID of the group")
                        .required(true),
                )
                .arg(
                    Arg::new("message")
                        .long("message")
                        .help("Content of the message to send")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("list-invites")
                .about("Lists pending group invitations")
                .arg(
                    Arg::new("key")
                        .long("key")
                        .value_name("KEY")
                        .help("Secret key for your identity")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("accept-invite")
                .about("Accepts a pending group invitation")
                .arg(
                    Arg::new("key")
                        .long("key")
                        .value_name("KEY")
                        .help("Secret key for your identity")
                        .required(true),
                )
                .arg(
                    Arg::new("group-id")
                        .long("group-id")
                        .help("Hex-encoded ID of the group to join")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("get-pubkey")
                .about("Gets the public key from a secret key")
                .arg(
                    Arg::new("key")
                        .long("key")
                        .value_name("KEY")
                        .help("Secret key for the identity")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("get-messages")
                .about("Fetches and displays messages for a group")
                .arg(
                    Arg::new("key")
                        .long("key")
                        .value_name("KEY")
                        .help("Secret key for your identity")
                        .required(true),
                )
                .arg(
                    Arg::new("group-id")
                        .long("group-id")
                        .help("Hex-encoded Nostr group ID")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("list-groups")
                .about("Lists all groups")
                .arg(
                    Arg::new("key")
                        .long("key")
                        .value_name("KEY")
                        .help("Secret key for your identity")
                        .required(true),
                ),
        )
        .get_matches();

    // Use DialogConfig to get relay URLs, respecting environment variables
    let config = DialogConfig::from_env();
    let relay_url = config.relay_urls.first()
        .ok_or(DialogError::General("No relay URLs configured".into()))?
        .clone();

    match matches.subcommand() {
        Some(("publish-key", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            println!("Using key for: {}", key_arg);
            
            let dialog_lib = create_dialog_lib(&sk_hex, &relay_url).await?;
            
            // Connect to relay
            dialog_lib.connect().await?;
            
            // Publish key packages
            let event_ids = dialog_lib.publish_key_packages().await?;
            println!("Published {} key package(s)", event_ids.len());
            for event_id in event_ids {
                println!("Event ID: {}", event_id);
            }
        }
        Some(("create-group", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let dialog_lib = create_dialog_lib(&sk_hex, &relay_url).await?;
            println!("Using key for: {}", key_arg);

            // Connect to relay
            dialog_lib.connect().await?;

            let group_name = sub_matches.get_one::<String>("name").unwrap();
            let counterparty_pk_hex = sub_matches.get_one::<String>("counterparty").unwrap();
            let counterparty_pk = PublicKey::from_hex(counterparty_pk_hex)?;

            println!("Creating group '{}' with counterparty: {}", group_name, counterparty_pk.to_hex());
            
            let group_id = dialog_lib.create_conversation(group_name, vec![counterparty_pk]).await?;
            println!("Group created successfully. Group ID: {}", group_id);
        }
        Some(("send-message", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let dialog_lib = create_dialog_lib(&sk_hex, &relay_url).await?;
            println!("Using key for: {}", key_arg);

            // Connect to relay
            dialog_lib.connect().await?;

            let group_id_hex = sub_matches.get_one::<String>("group-id").unwrap();
            let message = sub_matches.get_one::<String>("message").unwrap();

            // Parse group ID (supporting both 32 and 64 char hex)
            let group_id = if group_id_hex.len() == 32 {
                let group_id_bytes = hex::decode(group_id_hex)
                    .map_err(|e| DialogError::General(format!("Invalid group ID: {}", e)))?;
                GroupId::from_slice(&group_id_bytes)
            } else if group_id_hex.len() == 64 {
                // Need to find the group by Nostr ID
                let conversations = dialog_lib.get_conversations().await?;
                conversations.iter()
                    .find(|c| &c.id == group_id_hex)
                    .ok_or(DialogError::General("Group not found".into()))?
                    .group_id
                    .clone()
                    .ok_or(DialogError::General("Group has no MLS group ID".into()))?
            } else {
                return Err(DialogError::General("Invalid group ID length".into()));
            };

            // Sync group state before sending
            dialog_lib.fetch_and_process_group_events(&group_id).await?;

            println!("Sending message to group...");
            dialog_lib.send_message(&group_id, message).await?;
            println!("Message sent successfully!");
        }
        Some(("list-invites", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let dialog_lib = create_dialog_lib(&sk_hex, &relay_url).await?;
            println!("Listing invites for: {}", key_arg);

            // Connect to relay
            dialog_lib.connect().await?;

            let invite_result = dialog_lib.list_pending_invites().await?;
            
            if invite_result.processing_errors.len() > 0 {
                println!("\nProcessing errors:");
                for error in &invite_result.processing_errors {
                    println!("  {}", error);
                }
            }
            
            if invite_result.invites.is_empty() {
                println!("\nNo pending invites found.");
            } else {
                println!("\nPending invites:");
                for invite in invite_result.invites {
                    println!("  Group Name: {}", invite.group_name);
                    println!("  Group ID: {}", hex::encode(invite.group_id.as_slice()));
                    println!("  Member Count: {}", invite.member_count);
                    if let Some(inviter) = invite.inviter {
                        println!("  Inviter: {}", inviter.to_hex());
                    }
                    println!("");
                }
            }
        }
        Some(("accept-invite", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let dialog_lib = create_dialog_lib(&sk_hex, &relay_url).await?;
            println!("Accepting invite for: {}", key_arg);

            // Connect to relay
            dialog_lib.connect().await?;

            let group_id_hex = sub_matches.get_one::<String>("group-id").unwrap();
            
            dialog_lib.accept_invite(group_id_hex).await?;
            println!("Successfully joined group!");
        }
        Some(("get-pubkey", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let keys = Keys::parse(&sk_hex)?;
            println!("{}", keys.public_key().to_hex());
        }
        Some(("get-messages", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let dialog_lib = create_dialog_lib(&sk_hex, &relay_url).await?;
            println!("Getting messages for: {}", key_arg);

            // Connect to relay
            dialog_lib.connect().await?;

            let group_id_hex = sub_matches.get_one::<String>("group-id").unwrap();
            
            // Parse group ID
            let group_id = if group_id_hex.len() == 32 {
                let group_id_bytes = hex::decode(group_id_hex)
                    .map_err(|e| DialogError::General(format!("Invalid group ID: {}", e)))?;
                GroupId::from_slice(&group_id_bytes)
            } else if group_id_hex.len() == 64 {
                // Need to find the group by Nostr ID
                let conversations = dialog_lib.get_conversations().await?;
                conversations.iter()
                    .find(|c| &c.id == group_id_hex)
                    .ok_or(DialogError::General("Group not found".into()))?
                    .group_id
                    .clone()
                    .ok_or(DialogError::General("Group has no MLS group ID".into()))?
            } else {
                return Err(DialogError::General("Invalid group ID length".into()));
            };

            let result = dialog_lib.fetch_messages(&group_id).await?;
            
            if result.processing_errors.len() > 0 {
                println!("\nProcessing errors:");
                for error in &result.processing_errors {
                    println!("  {}", error);
                }
            }
            
            if result.messages.is_empty() {
                println!("\nNo messages found in group.");
            } else {
                println!("\n--- Messages for group {} ---", group_id_hex);
                for message in result.messages {
                    println!("From: {}", message.sender.to_hex());
                    println!("Content: {}", message.content);
                    println!("--------------------");
                }
            }
        }
        Some(("list-groups", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let dialog_lib = create_dialog_lib(&sk_hex, &relay_url).await?;
            println!("Listing groups for: {}", key_arg);

            let conversations = dialog_lib.get_conversations().await?;
            
            if conversations.is_empty() {
                println!("No groups found.");
            } else {
                println!("\nGroups:");
                for conv in conversations {
                    println!("  Name: {}", conv.name);
                    if let Some(group_id) = &conv.group_id {
                        println!("  Group ID (MLS): {}", hex::encode(group_id.as_slice()));
                    }
                    println!("  Group ID (Nostr): {}", conv.id);
                    println!("  Participants: {}", conv.participants.len());
                    println!("");
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}