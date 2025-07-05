mod app;
mod error;
mod model;
mod storage;
mod ui;
mod update;

use app::App;
use error::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use std::{env, path::PathBuf};
use dotenv::{dotenv, from_path};

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

        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break,
        }
    }
}

fn get_secret_key(key_arg: &str) -> Result<String> {
    match key_arg {
        "bob" => {
            find_and_load_env();
            env::var("BOB_SK_HEX")
                .map_err(|_| error::DialogTuiError::InvalidInput { 
                    message: "BOB_SK_HEX not found in environment variables".to_string() 
                })
        }
        "alice" => {
            find_and_load_env();
            env::var("ALICE_SK_HEX")
                .map_err(|_| error::DialogTuiError::InvalidInput { 
                    message: "ALICE_SK_HEX not found in environment variables".to_string() 
                })
        }
        hex_key => {
            // Validate that it looks like a hex string
            if hex_key.len() == 64 && hex_key.chars().all(|c| c.is_ascii_hexdigit()) {
                Ok(hex_key.to_string())
            } else {
                Err(error::DialogTuiError::InvalidInput { 
                    message: "Key must be either 'bob', 'alice', or a 64-character hex string".to_string() 
                })
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up tracing to file
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("/tmp/dialog_tui.log")?;
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_writer(log_file)
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Get arguments
    let args: Vec<String> = std::env::args().collect();
    
    // Check for test mode
    if args.len() > 1 && args[1] == "--test" {
        println!("Running in test mode - storage initialization only");
        let test_key = args.get(2)
            .map(|k| get_secret_key(k))
            .unwrap_or_else(|| Ok("0000000000000000000000000000000000000000000000000000000000000001".to_string()))?;
        
        // Test storage initialization without UI
        let mut storage = crate::storage::PerPubkeyStorage::new()?;
        let keys = nostr_sdk::Keys::parse(&test_key)?;
        storage.init_for_pubkey(&keys).await?;
        
        println!("✅ Storage initialization successful for pubkey: {}", keys.public_key().to_hex());
        println!("✅ Database created at: ~/.local/share/dialog_tui/{}/", keys.public_key().to_hex());
        return Ok(());
    }

    // Check if we have a proper terminal
    if !atty::is(atty::Stream::Stdout) {
        eprintln!("Error: dialog_tui requires a proper terminal environment.");
        eprintln!("Make sure you're running this directly in a terminal, not through an IDE or other tool.");
        eprintln!("");
        eprintln!("Usage: dialog_tui <alice|bob|hex_key>");
        eprintln!("Test mode: dialog_tui --test [alice|bob|hex_key]");
        std::process::exit(1);
    }

    // Get key argument from command line
    let key_arg = args.get(1).cloned();
    
    if key_arg.is_none() {
        eprintln!("Usage: dialog_tui <alice|bob|hex_key>");
        eprintln!("       dialog_tui alice    - Use Alice's key from ALICE_SK_HEX env var");
        eprintln!("       dialog_tui bob      - Use Bob's key from BOB_SK_HEX env var");
        eprintln!("       dialog_tui <hex>    - Use provided 64-char hex private key");
        eprintln!("");
        eprintln!("Test mode:");
        eprintln!("       dialog_tui --test [alice|bob|hex_key]");
        std::process::exit(1);
    }

    // Get the secret key based on the argument
    let private_key = get_secret_key(&key_arg.unwrap())?;

    // Initialize and run app
    let mut app = App::new().await?;
    app.init_with_key(Some(private_key)).await?;
    
    if let Err(e) = app.run().await {
        eprintln!("Application error: {}", e);
    }

    Ok(())
}
