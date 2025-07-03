use clap::{Arg, Command};
use dotenv::{dotenv, from_path};
use nostr_mls::prelude::*;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_sdk::prelude::*;
use std::{env, path::PathBuf};
use tempfile::TempDir;
use thiserror::Error;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use nip04;

/// Generate a new identity and return the keys, NostrMls instance, and temp directory
/// We use a different temp directory for each identity because OpenMLS doesn't have a concept of partitioning storage for different identities.
/// Because of this, we need to create diffrent databases for each identity.
fn generate_identity_with_key(
    sk_hex: &str,
) -> Result<(Keys, NostrMls<NostrMlsSqliteStorage>, TempDir), Box<dyn std::error::Error>> {
    let keys = Keys::parse(sk_hex)?;
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("mls.db");
    let nostr_mls = NostrMls::new(NostrMlsSqliteStorage::new(db_path).unwrap());
    Ok((keys, nostr_mls, temp_dir))
}

/// Generate a new identity and return the keys, NostrMls instance, and temp directory
/// We use a different temp directory for each identity because OpenMLS doesn't have a concept of partitioning storage for different identities.
/// Because of this, we need to create diffrent databases for each identity.
fn generate_identity() -> (Keys, NostrMls<NostrMlsSqliteStorage>, TempDir) {
    let keys = Keys::generate();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("mls.db");
    let nostr_mls = NostrMls::new(NostrMlsSqliteStorage::new(db_path).unwrap());
    (keys, nostr_mls, temp_dir)
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
        .subcommand(Command::new("list").about("Lists all MLS key packages from the relay"))
        .get_matches();

    match matches.subcommand() {
        Some(("publish-key", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let (keys, nostr_mls, _temp_dir) = generate_identity_with_key(&sk_hex)?;
            println!("Using provided key: {}", key_arg);

            let relay_url = RelayUrl::parse("ws://localhost:8080").unwrap();

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
        Some(("create-group", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let (keys, nostr_mls, _temp_dir) = generate_identity_with_key(&sk_hex)?;
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
                .author(counterparty_pk)
                .limit(1);
            let timeout = std::time::Duration::from_secs(10);

            let events = client
                .fetch_events(filter, timeout)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;

            if let Some(key_package_event) = events.first() {
                println!("Found key package event for counterparty: {}", key_package_event.id);

                let members = vec![keys.public_key(), counterparty_pk];
                let relay_url = RelayUrl::parse(relay_url_str).unwrap();

                let group_create_result = nostr_mls
                    .create_group(
                        group_name,
                        "", // description
                        &keys.public_key(),
                        vec![key_package_event.clone()],
                        members,
                        vec![relay_url],
                    )
                    .map_err(|e| DialogError::General(Box::new(e)))?;

                println!(
                    "Group created successfully. MLS Group ID: {}",
                    hex::encode(group_create_result.group.mls_group_id.as_slice())
                );

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
            let (keys, nostr_mls, _temp_dir) = generate_identity_with_key(&sk_hex)?;
            println!("Using key for: {}", key_arg);

            let group_id_hex = sub_matches.get_one::<String>("group-id").unwrap();
            let group_id_bytes = hex::decode(group_id_hex).map_err(|e| DialogError::General(Box::new(e)))?;
            let group_id = GroupId::from_slice(&group_id_bytes);

            let message = sub_matches.get_one::<String>("message").unwrap();

            let rumor = EventBuilder::new(Kind::TextNote, message).build(keys.public_key());

            let message_event = nostr_mls
                .create_message(&group_id, rumor)
                .map_err(|e| DialogError::General(Box::new(e)))?;

            let relay_url_str = "ws://localhost:8080";
            let client = Client::new(keys.clone());
            client
                .add_relay(relay_url_str)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;
            client.connect().await;

            println!("Sending message event: {}", message_event.id);
            let output = client
                .send_event(&message_event)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;

            println!("Sent message event ID: {:?}", output.id());
        }
        Some(("list-invites", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let (keys, nostr_mls, _temp_dir) = generate_identity_with_key(&sk_hex)?;
            println!("Listing invites for: {}", key_arg);

            let relay_url_str = "ws://localhost:8080";
            let client = Client::new(keys.clone());
            client
                .add_relay(relay_url_str)
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;
            client.connect().await;

            let filter = Filter::new()
                .kind(Kind::GiftWrap)
                .pubkey(keys.public_key());

            let events = client
                .fetch_events(filter, std::time::Duration::from_secs(10))
                .await
                .map_err(|e| DialogError::General(Box::new(e)))?;

            println!("Found {} potential invites", events.len());

            for event in events {
                if let Ok(rumor) = client.unwrap_gift_wrap(&event).await {
                    nostr_mls.process_welcome(&event.id, &rumor.rumor)?;
                }
            }

            let pending_welcomes = nostr_mls.get_pending_welcomes()?;
            if pending_welcomes.is_empty() {
                println!("No pending invites found.");
            } else {
                println!("Pending invites:");
                for welcome in pending_welcomes {
                    println!("  Group Name: {}", welcome.group_name);
                    println!("  Group ID: {}", hex::encode(welcome.mls_group_id.as_slice()));
                    println!("  Member Count: {}", welcome.member_count);
                    println!("");
                }
            }
        }
        Some(("accept-invite", sub_matches)) => {
            let key_arg = sub_matches.get_one::<String>("key").unwrap();
            let sk_hex = get_secret_key(key_arg)?;
            let (_keys, nostr_mls, _temp_dir) = generate_identity_with_key(&sk_hex)?;
            println!("Accepting invite for: {}", key_arg);

            let group_id_hex = sub_matches.get_one::<String>("group-id").unwrap();
            let group_id_bytes = hex::decode(group_id_hex).map_err(|e| DialogError::General(Box::new(e)))?;
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
        _ => unreachable!(),
    }

    Ok(())
}
