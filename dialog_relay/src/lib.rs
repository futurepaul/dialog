use anyhow::Result;
use nostr_relay_builder::{LocalRelay, RelayBuilder};
use tracing::{info, debug, warn};
use std::net::IpAddr;
use tokio::signal;
use std::time::Duration;

pub async fn run_relay() -> Result<()> {
    // Initialize with debug logging (no trace spam)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    info!("🚀 Starting Dialog Relay with debug logging");
    debug!("Debug logging enabled - showing connections, events, and protocol flow");

    // Configure the relay 
    let addr: IpAddr = "127.0.0.1".parse()?;
    info!("📍 Parsed address: {}", addr);
    
    let builder = RelayBuilder::default()
        .addr(addr)
        .port(7979);

    info!("⚙️  Relay configured to listen on {}:7979", addr);
    info!("🔧 Building relay with RelayBuilder...");
    debug!("RelayBuilder configuration: addr={}, port=7979", addr);

    info!("🌟 Starting relay server...");

    // Run the relay - this returns a relay object immediately
    let relay = LocalRelay::run(builder).await?;

    info!("✅ Relay started successfully!");
    info!("🌐 Relay URL: {}", relay.url());
    info!("📡 WebSocket endpoint ready for connections");
    warn!("🔍 Relay is running with debug logging - showing key operations");
    
    // Keep the program running with proper signal handling
    tokio::select! {
        _ = signal::ctrl_c() => {
            warn!("🛑 Received shutdown signal (Ctrl+C)");
        }
        _ = async {
            let mut counter = 0;
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                counter += 1;
                info!("💓 Relay heartbeat #{} - still running on {}", counter, relay.url());
                if counter % 6 == 0 {
                    debug!("📊 Relay has been running for {} minutes", counter / 6);
                }
            }
        } => {}
    }
    
    warn!("🔌 Shutting down relay...");
    info!("👋 Dialog Relay stopped");
    Ok(())
}
