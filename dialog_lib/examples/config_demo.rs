use dialog_lib::{DialogConfig, DialogLib};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Using default configuration
    let lib_default = DialogLib::new().await?;
    println!("Default library created");

    // Example 2: Using environment variables
    // Set DIALOG_RELAY_URL=wss://my-relay.com
    let config = DialogConfig::from_env();
    let lib_env = DialogLib::new_with_relay(config.relay_url).await?;
    println!("Library created from environment variables");

    // Example 3: Using custom relay URL
    let lib_custom = DialogLib::new_with_relay("wss://my-custom-relay.com").await?;
    println!("Library created with custom relay URL");

    // Example 4: Using specific keys
    let keys = nostr_mls::prelude::Keys::generate();
    let lib_keys = DialogLib::new_with_keys(keys).await?;
    println!("Library created with specific keys");

    // Demonstrate using the library
    let contacts = lib_default.get_contacts().await?;
    println!("Found {} contacts", contacts.len());

    Ok(())
}