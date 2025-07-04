use clap::{Arg, Command};
use dotenv::{dotenv, from_path};
use nostr_mls::prelude::*;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_mls_memory_storage::NostrMlsMemoryStorage;
use nostr_sdk::{prelude::*, nips::nip59};
use nostr_mls::groups::NostrGroupConfigData;
use std::{env, path::PathBuf, fs};
use thiserror::Error;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;



/// Generate a new identity and return the keys, NostrMls instance, and temp directory
/// We use a different temp directory for each identity because OpenMLS doesn't have a concept of partitioning storage for different identities.
/// Because of this, we need to create diffrent databases for each identity.
fn generate_identity_with_key(
    sk_hex: &str,
) -> Result<(Keys, NostrMls<NostrMlsSqliteStorage>), Box<dyn std::error::Error>> {
    let keys = Keys::parse(sk_hex)?;
    let data_dir = env::current_dir()?.join(".dialog_cli_data");
    let identity_dir = data_dir.join(keys.public_key().to_hex());
    fs::create_dir_all(&identity_dir)?;
    let db_path = identity_dir.join("mls.db");
    let nostr_mls = NostrMls::new(NostrMlsSqliteStorage::new(db_path).unwrap());
    Ok((keys, nostr_mls))
}

/// Generate a new identity with memory storage
fn generate_identity_with_memory(
    sk_hex: &str,
) -> Result<(Keys, NostrMls<NostrMlsMemoryStorage>), Box<dyn std::error::Error>> {
    let keys = Keys::parse(sk_hex)?;
    let nostr_mls = NostrMls::new(NostrMlsMemoryStorage::default());
    Ok((keys, nostr_mls))
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

fn get_secret_key(key_arg: &str) -> Result<String, Box<dyn std::error::Error>> {
    match key_arg {
        "bob" => {
            find_and_load_env();
            env::var("BOB_SK_HEX")
                .map_err(|_| "BOB_SK_HEX not found in environment variables".into())
        }
        "alice" => {
            find_and_load_env();
            env::var("ALICE_SK_HEX")
                .map_err(|_| "ALICE_SK_HEX not found in environment variables".into())
        }
        hex_key => {
            // Validate that it looks like a hex string
            if hex_key.len() == 64 && hex_key.chars().all(|c| c.is_ascii_hexdigit()) {
                Ok(hex_key.to_string())
            } else {
                Err("Key must be either 'bob', 'alice', or a 64-character hex string".into())
            }
        }
    }
}

fn find_group_by_id(
    nostr_mls: &NostrMls<NostrMlsSqliteStorage>,
    group_id_hex: &str,
) -> Result<group_types::Group, DialogError> {
    let groups = nostr_mls.get_groups()?;
    
    // Try as MLS Group ID first (32 hex chars)
    if group_id_hex.len() == 32 {
        if let Ok(group_id_bytes) = ::hex::decode(group_id_hex) {
            let mls_group_id = GroupId::from_slice(&group_id_bytes);
            for group in &groups {
                if group.mls_group_id == mls_group_id {
                    return Ok(group.clone());
                }
            }
        }
    }
    
    // Try as Nostr Group ID (64 hex chars)
    if group_id_hex.len() == 64 {
        if let Ok(nostr_group_id_bytes) = ::hex::decode(group_id_hex) {
            for group in &groups {
                if group.nostr_group_id.as_slice() == nostr_group_id_bytes.as_slice() {
                    return Ok(group.clone());
                }
            }
        }
    }
    
    Err(DialogError::General("Group not found".into()))
}

#[derive(Error, Debug)]
enum DialogError {
    #[error("Nostr MLS error: {0}")]
    NostrMls(#[from] nostr_mls::Error),

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
    General(#[from] Box<dyn std::error::Error>),
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
                )
                .arg(
                    Arg::new("memory-storage")
                        .long("memory-storage")
                        .help("Use memory storage instead of SQLite (for testing)")
                        .action(clap::ArgAction::SetTrue),
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
        .subcommand(Command::new("list").about("Lists all MLS key packages from the relay"))
        .subcommand(
            Command::new("create-group-and-send")
                .about("Creates a new group, invites a counterparty, and immediately sends a message")
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
                )
                .arg(
                    Arg::new("message")
                        .long("message")
                        .help("Message to send immediately after group creation")
                        .required(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("publish-key", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let use_memory = sub_matches.get_flag("memory-storage");
            
            let relay_url = RelayUrl::parse("ws://localhost:8080").unwrap();

            if use_memory {
                println!("Using memory storage for: {}", key_arg);
                let (keys, nostr_mls) = generate_identity_with_memory(&sk_hex)?;
                
                let (key_package_encoded, tags) =
                    nostr_mls.create_key_package_for_event(&keys.public_key(), [relay_url.clone()])?;

                let key_package_event = EventBuilder::new(Kind::MlsKeyPackage, key_package_encoded)
                    .tags(tags)
                    .sign_with_keys(&keys)
                    .map_err(|e| DialogError::General(Box::new(e)))?;
                println!("Key package event: {:?}", key_package_event);

                let client = Client::new(keys.clone());
                client
                    .add_relay(relay_url.to_string())
                    .await
                    .map_err(|e| DialogError::General(Box::new(e)))?;
                client.connect().await;

                println!("Publishing key package event...");
                let output = client
                    .send_event(&key_package_event)
                    .await
                    .map_err(|e| DialogError::General(Box::new(e)))?;
                println!(
                    "Event ID: {}",
                    output.id().to_bech32().map_err(|e| DialogError::General(Box::new(e)))?
                );
                println!("Sent to: {:?}", output.success);
                println!("Not sent to: {:?}", output.failed);
            } else {
                println!("Using SQLite storage for: {}", key_arg);
                let (keys, nostr_mls) = generate_identity_with_key(&sk_hex)?;
                
                let (key_package_encoded, tags) =
                    nostr_mls.create_key_package_for_event(&keys.public_key(), [relay_url.clone()])?;

                let key_package_event = EventBuilder::new(Kind::MlsKeyPackage, key_package_encoded)
                    .tags(tags)
                    .sign_with_keys(&keys)
                    .map_err(|e| DialogError::General(Box::new(e)))?;
                println!("Key package event: {:?}", key_package_event);

                let client = Client::new(keys.clone());
                client
                    .add_relay(relay_url.to_string())
                    .await
                    .map_err(|e| DialogError::General(Box::new(e)))?;
                client.connect().await;

                println!("Publishing key package event...");
                let output = client
                    .send_event(&key_package_event)
                    .await
                    .map_err(|e| DialogError::General(Box::new(e)))?;
                println!(
                    "Event ID: {}",
                    output.id().to_bech32().map_err(|e| DialogError::General(Box::new(e)))?
                );
                println!("Sent to: {:?}", output.success);
                println!("Not sent to: {:?}", output.failed);
            }
        }
        Some(("create-group", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let (keys, nostr_mls) = generate_identity_with_key(&sk_hex)?;
            println!("Using key for: {}", key_arg);

            let group_name = sub_matches.get_one::<String>("name").unwrap();
            let counterparty_pk_hex = sub_matches.get_one::<String>("counterparty").unwrap();
            let counterparty_pk = PublicKey::from_hex(counterparty_pk_hex)
                .map_err(|e| DialogError::General(Box::new(e)))?;

            let relay_url_str = "ws://localhost:8080";
            let client = Client::new(keys.clone());
            client
                .add_relay(relay_url_str)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;
            client.connect().await;

            println!("Fetching key package for counterparty: {}", counterparty_pk.to_hex());
            let filter = Filter::new()
                .kind(Kind::MlsKeyPackage)
                .author(counterparty_pk);
            let timeout = std::time::Duration::from_secs(10);

            let events = client
                .fetch_events(filter, timeout)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;

            if let Some(key_package_event) = events.first() {
                println!("Found key package event for counterparty: {}", key_package_event.id);

                // Parse the key package to validate it (following mls_sqlite.rs working example)
                let _counterparty_key_package = nostr_mls.parse_key_package(key_package_event)
                    .map_err(|e| DialogError::General(Box::new(e)))?;
                println!("Successfully parsed counterparty's key package");

                let admins = vec![keys.public_key(), counterparty_pk];
                let relay_url = RelayUrl::parse(relay_url_str).unwrap();

                let config = NostrGroupConfigData::new(
                    group_name.to_string(),
                    "".to_string(),
                    None,
                    None,
                    vec![relay_url],
                );

                let group_create_result = nostr_mls
                    .create_group(
                        &keys.public_key(),
                        vec![key_package_event.clone()],
                        admins,
                        config,
                    )
                    .map_err(|e| DialogError::General(Box::new(e)))?;

                println!(
                    "Group created successfully. Group ID: {}",
                    ::hex::encode(group_create_result.group.mls_group_id.as_slice())
                );
                println!("Group created at epoch: {}", group_create_result.group.epoch);

                let welcome_rumors = group_create_result.welcome_rumors;
                println!("Publishing {} welcome rumor(s)...", welcome_rumors.len());

                for rumor in welcome_rumors {
                    let gift_wrap_event = EventBuilder::gift_wrap(&keys, &counterparty_pk, rumor, None)
                        .await
                        .map_err(|e| DialogError::General(Box::new(e)))?;

                    println!("Publishing welcome rumor event (gift-wrapped): {}", gift_wrap_event.id);
                    let output = client
                        .send_event(&gift_wrap_event)
                        .await
                        .map_err(|e| DialogError::General(Box::new(e)))?;
                    println!("Published welcome rumor event ID: {:?}", output.id());
                }
            } else {
                println!("Could not find key package for counterparty on the relay.");
            }
        }
        Some(("send-message", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let (keys, nostr_mls) = generate_identity_with_key(&sk_hex)?;
            println!("Using key for: {}", key_arg);

            let group_id_hex = sub_matches.get_one::<String>("group-id").unwrap();
            let stored_group = find_group_by_id(&nostr_mls, group_id_hex)?;
            let group_id = stored_group.mls_group_id.clone();

            let relay_url_str = "ws://localhost:8080";
            let client = Client::new(keys.clone());
            client
                .add_relay(relay_url_str)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;
            client.connect().await;

            // CRITICAL: Fetch and process any MLS evolution events before sending
            // This ensures Alice's group state is synchronized with other members
            let nostr_group_id_hex = ::hex::encode(&stored_group.nostr_group_id);
            let filter = Filter::new()
                .kind(Kind::MlsGroupMessage)
                .custom_tag(nostr_sdk::SingleLetterTag::lowercase(nostr_sdk::Alphabet::H), nostr_group_id_hex);

            let events = client.fetch_events(filter, std::time::Duration::from_secs(10)).await.map_err(|e| DialogError::General(Box::new(e)))?;
            println!("Found {} MLS group evolution events before sending", events.len());
            for event in events {
                println!("Processing evolution event {} with kind {:?}", event.id, event.kind);
                if let Err(e) = nostr_mls.process_message(&event) {
                    println!("Failed to process evolution event {}: {}", event.id, e);
                } else {
                    println!("Successfully processed evolution event {}", event.id);
                }
            }

            // Debug: Check what epoch Alice thinks the group is at
            let groups = nostr_mls.get_groups()?;
            if let Some(alice_group) = groups.iter().find(|g| g.mls_group_id == group_id) {
                println!("Alice's view: Group is at epoch {}", alice_group.epoch);
            }

            let message = sub_matches.get_one::<String>("message").unwrap();

            let rumor = EventBuilder::new(Kind::TextNote, message).build(keys.public_key());

            let message_event = nostr_mls
                .create_message(&group_id, rumor)
                .map_err(|e| DialogError::General(Box::new(e)))?;
            
            println!("Created message event with kind: {:?}", message_event.kind);

            println!("Sending message event: {}", message_event.id);
            let output = client
                .send_event(&message_event)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;

            println!("Sent message event ID: {:?}", output.id());

            // CRITICAL: Process the message event locally to maintain MLS group state synchronization
            // This is required in MLS - the sender must also process their own message
            println!("Processing message event locally...");
            if let Err(e) = nostr_mls.process_message(&message_event) {
                println!("Failed to process own message locally: {}", e);
            } else {
                println!("Successfully processed own message locally");
            }
        }
        Some(("list-invites", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let (keys, nostr_mls) = generate_identity_with_key(&sk_hex)?;
            println!("Listing invites for: {}", key_arg);

            // First, fetch gift-wrapped events from the relay
            let relay_url_str = "ws://localhost:8080";
            let client = Client::new(keys.clone());
            client
                .add_relay(relay_url_str)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;
            client.connect().await;

            // Fetch gift-wrapped events (Kind::GiftWrap)
            let filter = Filter::new().kind(Kind::GiftWrap).pubkey(keys.public_key());
            let events = client
                .fetch_events(filter, std::time::Duration::from_secs(10))
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;

            // Process gift-wrapped events to extract welcome messages
            for event in events {
                // Try to extract rumor from gift wrap using NIP-59
                if let Ok(unwrapped_gift) = nip59::extract_rumor(&keys, &event).await {
                    // Process the welcome rumor
                    if let Err(e) = nostr_mls.process_welcome(&event.id, &unwrapped_gift.rumor) {
                        // Ignore errors - the event might not be a welcome message
                        tracing::debug!("Failed to process welcome rumor from {}: {}", unwrapped_gift.sender, e);
                    }
                }
            }

            let pending_welcomes = nostr_mls.get_pending_welcomes()?;
            if pending_welcomes.is_empty() {
                println!("No pending invites found.");
            } else {
                println!("Pending invites:");
                for welcome in pending_welcomes {
                    println!("  Group Name: {}", welcome.group_name);
                    println!("  Group ID: {}", ::hex::encode(welcome.mls_group_id.as_slice()));
                    println!("  Member Count: {}", welcome.member_count);
                    println!("");
                }
            }
        }
        Some(("accept-invite", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let (_keys, nostr_mls) = generate_identity_with_key(&sk_hex)?;
            println!("Accepting invite for: {}", key_arg);

            let group_id_hex = sub_matches.get_one::<String>("group-id").unwrap();
            let group_id_bytes = ::hex::decode(group_id_hex).map_err(|e| DialogError::General(Box::new(e)))?;
            let group_id = GroupId::from_slice(&group_id_bytes);

            let pending_welcomes = nostr_mls.get_pending_welcomes()?;
            if let Some(welcome) = pending_welcomes
                .iter()
                .find(|w| w.mls_group_id == group_id)
            {
                nostr_mls.accept_welcome(welcome)?;
                println!("Successfully joined group: {}", welcome.group_name);
            } else {
                println!("No pending invite found for group ID: {}", group_id_hex);
            }
        }
        Some(("get-pubkey", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let keys =
                Keys::parse(&sk_hex).map_err(|e| DialogError::General(Box::new(e)))?;
            println!("{}", keys.public_key().to_hex());
        }
        Some(("get-messages", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let (keys, nostr_mls) = generate_identity_with_key(&sk_hex)?;
            println!("Getting messages for: {}", key_arg);

            let group_id_hex = sub_matches.get_one::<String>("group-id").unwrap();
            let stored_group = find_group_by_id(&nostr_mls, group_id_hex)?;
            let mls_group_id = stored_group.mls_group_id.clone();
            
            // Fetch and process MLS group messages from the relay
            let relay_url_str = "ws://localhost:8080";
            let client = Client::new(keys.clone());
            client
                .add_relay(relay_url_str)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;
            client.connect().await;
            
            // Filter for MLS group messages tagged with this group's Nostr Group ID
            let nostr_group_id_hex = ::hex::encode(&stored_group.nostr_group_id);
            let filter = Filter::new()
                .kind(Kind::MlsGroupMessage)
                .custom_tag(nostr_sdk::SingleLetterTag::lowercase(nostr_sdk::Alphabet::H), nostr_group_id_hex);

            let events = client.fetch_events(filter, std::time::Duration::from_secs(10)).await.map_err(|e| DialogError::General(Box::new(e)))?;
            println!("Found {} MLS group message events on relay", events.len());
            for event in events {
                println!("Processing event {} with kind {:?}", event.id, event.kind);
                if let Err(e) = nostr_mls.process_message(&event) {
                    println!("Failed to process message event {}: {}", event.id, e);
                } else {
                    println!("Successfully processed message event {}", event.id);
                }
            }

            // Debug: Check what epoch Bob thinks the group is at
            let groups = nostr_mls.get_groups()?;
            if let Some(bob_group) = groups.iter().find(|g| g.mls_group_id == mls_group_id) {
                println!("Bob's view: Group is at epoch {}", bob_group.epoch);
            }

            let messages = nostr_mls.get_messages(&mls_group_id)?;
            if messages.is_empty() {
                println!("No messages found in group.");
            } else {
                println!("\n--- Messages for group {} ---", group_id_hex);
                for message in messages {
                    println!("From: {}", message.pubkey.to_hex());
                    println!("Content: {}", message.content);
                    println!("--------------------");
                }
            }
        }
        Some(("list", _sub_matches)) => {
            println!("Listing key packages from relay...");
            let relay_url = RelayUrl::parse("ws://localhost:8080").unwrap();
            let client = Client::new(Keys::generate());
            client
                .add_relay(relay_url.to_string())
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;
            client.connect().await;

            let filter = Filter::new().kind(Kind::MlsKeyPackage);
            let timeout = std::time::Duration::from_secs(10);

            let events = client
                .fetch_events(filter, timeout)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;

            if events.is_empty() {
                println!("No key packages found.");
            } else {
                println!("Found {} key packages:", events.len());
                for event in events {
                    println!("- Event ID: {}", event.id);
                    println!("  Public Key: {}", event.pubkey);
                    println!("  Content: {}", event.content);
                    println!("  Kind: {}", event.kind);
                    println!("  Tags: {:?}", event.tags);
                    println!("");
                }
            }
        }
        Some(("create-group-and-send", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let (keys, nostr_mls) = generate_identity_with_key(&sk_hex)?;
            println!("Using key for: {}", key_arg);

            let group_name = sub_matches.get_one::<String>("name").unwrap();
            let counterparty_pk_hex = sub_matches.get_one::<String>("counterparty").unwrap();
            let counterparty_pk = PublicKey::from_hex(counterparty_pk_hex)
                .map_err(|e| DialogError::General(Box::new(e)))?;

            let message = sub_matches.get_one::<String>("message").unwrap();

            let relay_url_str = "ws://localhost:8080";
            let client = Client::new(keys.clone());
            client
                .add_relay(relay_url_str)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;
            client.connect().await;

            println!("Fetching key package for counterparty: {}", counterparty_pk.to_hex());
            let filter = Filter::new()
                .kind(Kind::MlsKeyPackage)
                .author(counterparty_pk);
            let timeout = std::time::Duration::from_secs(10);

            let events = client
                .fetch_events(filter, timeout)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;

            if let Some(key_package_event) = events.first() {
                println!("Found key package event for counterparty: {}", key_package_event.id);

                // Parse the key package to validate it (following mls_sqlite.rs working example)
                let _counterparty_key_package = nostr_mls.parse_key_package(key_package_event)
                    .map_err(|e| DialogError::General(Box::new(e)))?;
                println!("Successfully parsed counterparty's key package");

                let admins = vec![keys.public_key(), counterparty_pk];
                let relay_url = RelayUrl::parse(relay_url_str).unwrap();

                let config = NostrGroupConfigData::new(
                    group_name.to_string(),
                    "".to_string(),
                    None,
                    None,
                    vec![relay_url],
                );

                let group_create_result = nostr_mls
                    .create_group(
                        &keys.public_key(),
                        vec![key_package_event.clone()],
                        admins,
                        config,
                    )
                    .map_err(|e| DialogError::General(Box::new(e)))?;

                println!(
                    "Group created successfully. Group ID: {}",
                    ::hex::encode(group_create_result.group.mls_group_id.as_slice())
                );
                println!("Group created at epoch: {}", group_create_result.group.epoch);

                let welcome_rumors = group_create_result.welcome_rumors;
                println!("Publishing {} welcome rumor(s)...", welcome_rumors.len());

                for rumor in welcome_rumors {
                    let gift_wrap_event = EventBuilder::gift_wrap(&keys, &counterparty_pk, rumor, None)
                        .await
                        .map_err(|e| DialogError::General(Box::new(e)))?;

                    println!("Publishing welcome rumor event (gift-wrapped): {}", gift_wrap_event.id);
                    let output = client
                        .send_event(&gift_wrap_event)
                        .await
                        .map_err(|e| DialogError::General(Box::new(e)))?;
                    println!("Published welcome rumor event ID: {:?}", output.id());
                }

                let rumor = EventBuilder::new(Kind::TextNote, message).build(keys.public_key());

                let message_event = nostr_mls
                    .create_message(&group_create_result.group.mls_group_id, rumor)
                    .map_err(|e| DialogError::General(Box::new(e)))?;
                
                println!("Created message event with kind: {:?}", message_event.kind);

                println!("Sending message event: {}", message_event.id);
                let output = client
                    .send_event(&message_event)
                    .await
                    .map_err(|e| DialogError::General(Box::new(e)))?;

                println!("Sent message event ID: {:?}", output.id());

                // CRITICAL: Process the message event locally to maintain MLS group state synchronization
                // This is required in MLS - the sender must also process their own message
                println!("Processing message event locally...");
                if let Err(e) = nostr_mls.process_message(&message_event) {
                    println!("Failed to process own message locally: {}", e);
                } else {
                    println!("Successfully processed own message locally");
                }
            } else {
                println!("Could not find key package for counterparty on the relay.");
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
