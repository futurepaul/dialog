pub mod ht_mcp_automation;
pub mod whitenoise_interop;
pub mod test_scenarios;
pub mod whitenoise_coordination;
pub mod welcome_compatibility;
pub mod automation_coordination;

use anyhow::Result;
use tracing_subscriber;

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("info,whitenoise_dialog_integration=debug")
        .init();
}

pub async fn run_interop_tests() -> Result<()> {
    init_logging();
    
    let scenarios = test_scenarios::TestScenarios::new();
    
    // Run comprehensive interop test suite
    scenarios.run_complete_interop_test().await?;
    scenarios.run_stress_test().await?;
    scenarios.run_error_recovery_test().await?;
    
    Ok(())
}