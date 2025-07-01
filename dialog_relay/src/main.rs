use anyhow::Result;
use dialog_relay::run_relay;

#[tokio::main]
async fn main() -> Result<()> {
    run_relay().await
}