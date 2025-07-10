use anyhow::Result;
use whitenoise_dialog_integration::run_interop_tests;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Whitenoise-Dialog Interoperability Tests");
    
    run_interop_tests().await?;
    
    println!("All interoperability tests completed successfully");
    Ok(())
}