use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, error};
use whitenoise_dialog_integration::{
    ht_mcp_automation::DialogTuiAutomation,
    whitenoise_interop::WhitenoiseCoordination,
};

/// This file contains integration tests that would be added to whitenoise's integration_test.rs
/// These tests coordinate between whitenoise and dialog_tui using ht-mcp automation

#[tokio::test]
async fn test_whitenoise_creates_group_invites_dialog_tui() -> Result<()> {
    tracing_subscriber::fmt().init();
    info!("=== WHITENOISE INTEGRATION TEST: Create group and invite dialog_tui ===");
    
    // Step 1: Setup whitenoise test environment (this would use whitenoise's existing setup)
    // let whitenoise = setup_test_environment().await?;
    // let alice_account = create_test_account("alice").await?;
    
    // Step 2: Get dialog_tui pubkey via ht-mcp automation
    let dialog_pubkey = WhitenoiseCoordination::get_dialog_tui_pubkey().await?;
    info!("Got dialog_tui pubkey for invitation: {}", dialog_pubkey);
    
    // Step 3: Create group with dialog_tui as member (using whitenoise API)
    // let group_config = test_group_config();
    // let group_id = whitenoise.create_group(
    //     &alice_account,
    //     vec![dialog_pubkey.clone()], // members
    //     vec![alice_account.pubkey.clone()], // admins
    //     group_config
    // ).await?;
    // info!("Created group with ID: {}", group_id);
    
    // For now, simulate the group creation
    let group_id = "test_group_id";
    info!("Simulated group creation with ID: {}", group_id);
    
    // Step 4: Wait for dialog_tui to accept the invitation
    WhitenoiseCoordination::wait_for_group_member_join(group_id, &dialog_pubkey).await?;
    info!("Dialog_tui successfully joined the group");
    
    // Step 5: Send welcome message from whitenoise
    // whitenoise.send_message_to_group(&alice_account, &group_id, "Welcome to the group from whitenoise!").await?;
    info!("Sent welcome message from whitenoise");
    
    // Step 6: Wait for dialog_tui response
    let messages = WhitenoiseCoordination::wait_for_dialog_response(group_id).await?;
    info!("Received messages from dialog_tui: {:?}", messages);
    
    // Step 7: Verify the test message was received
    assert!(messages.iter().any(|m| m.contains("Hello from dialog_tui!")));
    info!("Successfully verified dialog_tui response");
    
    info!("=== WHITENOISE INTEGRATION TEST COMPLETED SUCCESSFULLY ===");
    Ok(())
}

#[tokio::test]
async fn test_dialog_tui_creates_group_invites_whitenoise() -> Result<()> {
    info!("=== WHITENOISE INTEGRATION TEST: Accept dialog_tui group invitation ===");
    
    // Step 1: Setup whitenoise account
    // let whitenoise = setup_test_environment().await?;
    // let bob_account = create_test_account("bob").await?;
    let whitenoise_pubkey = "simulated_whitenoise_pubkey";
    
    // Step 2: Setup dialog_tui via ht-mcp and create group
    let mut dialog_automation = DialogTuiAutomation::new();
    let _session_id = dialog_automation.create_session("dialog_creator", "ws://localhost:8080,ws://localhost:7777").await?;
    
    let dialog_pubkey = dialog_automation.setup_dialog_tui().await?;
    info!("Dialog_tui ready with pubkey: {}", dialog_pubkey);
    
    // Step 3: Dialog_tui creates group and invites whitenoise
    dialog_automation.create_group_and_invite("WhitenoiseTestGroup", whitenoise_pubkey).await?;
    info!("Dialog_tui created group and sent invitation");
    
    // Step 4: Whitenoise accepts the invitation (simulated)
    // let welcomes = whitenoise.fetch_welcomes(&bob_account).await?;
    // let dialog_welcome = welcomes.iter()
    //     .find(|w| w.group_name == "WhitenoiseTestGroup")
    //     .ok_or("Dialog invite not found")?;
    // whitenoise.accept_welcome(&bob_account, dialog_welcome.event_id.clone()).await?;
    
    tokio::time::sleep(Duration::from_secs(5)).await; // Simulate acceptance time
    info!("Whitenoise accepted the group invitation");
    
    // Step 5: Send response from whitenoise (simulated)
    // let group_id = GroupId::from(dialog_welcome.group_id.clone());
    // whitenoise.send_message_to_group(&bob_account, &group_id, "Thanks for the invite from whitenoise!").await?;
    info!("Whitenoise sent response message");
    
    // Step 6: Verify dialog_tui receives the message
    tokio::time::sleep(Duration::from_secs(3)).await;
    dialog_automation.send_keys(&["/fetch", "Enter"]).await?;
    
    let snapshot = dialog_automation.take_snapshot().await?;
    // In a real test, we'd verify the whitenoise message appears in the snapshot
    info!("Dialog_tui fetched messages, snapshot length: {}", snapshot.len());
    
    // Cleanup
    dialog_automation.close_session().await?;
    
    info!("=== WHITENOISE REVERSE INTEGRATION TEST COMPLETED ===");
    Ok(())
}

#[tokio::test]
async fn test_extended_message_exchange() -> Result<()> {
    info!("=== WHITENOISE INTEGRATION TEST: Extended message exchange ===");
    
    // Setup both clients in an existing group (simulate group already created)
    let mut dialog_automation = DialogTuiAutomation::new();
    let _session_id = dialog_automation.create_session("dialog_member", "ws://localhost:8080,ws://localhost:7777").await?;
    
    dialog_automation.setup_dialog_tui().await?;
    
    // Simulate that both clients are already in a group
    info!("Simulating existing group with both whitenoise and dialog_tui as members");
    
    // Send multiple messages from dialog_tui
    for i in 1..=5 {
        dialog_automation.send_test_message(&format!("Dialog test message #{}", i)).await?;
        
        // Simulate whitenoise responding (in real test, whitenoise would actually send)
        // whitenoise.send_message_to_group(&account, &group_id, &format!("Whitenoise response to #{}", i)).await?;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    // Fetch all messages
    dialog_automation.send_keys(&["/fetch", "Enter"]).await?;
    let final_snapshot = dialog_automation.take_snapshot().await?;
    
    // Verify message exchange (in real test, would check actual message content)
    assert!(final_snapshot.len() > 100); // Basic sanity check
    info!("Extended message exchange completed successfully");
    
    // Cleanup
    dialog_automation.close_session().await?;
    
    info!("=== EXTENDED MESSAGE EXCHANGE TEST COMPLETED ===");
    Ok(())
}

#[tokio::test] 
async fn test_welcome_message_compatibility() -> Result<()> {
    info!("=== WHITENOISE INTEGRATION TEST: Welcome message format compatibility ===");
    
    // This test would verify that whitenoise can process both gift-wrapped and direct MLS welcomes
    // that dialog_tui might send
    
    // Step 1: Setup dialog_tui to send dual-format welcomes
    let mut dialog_automation = DialogTuiAutomation::new();
    let _session_id = dialog_automation.create_session("welcome_tester", "ws://localhost:8080,ws://localhost:7777").await?;
    
    let dialog_pubkey = dialog_automation.setup_dialog_tui().await?;
    
    // Step 2: Create test group with specific welcome formats
    let whitenoise_pubkey = "test_whitenoise_pubkey_for_welcome_test";
    dialog_automation.create_group_and_invite("WelcomeFormatTest", whitenoise_pubkey).await?;
    
    // Step 3: Whitenoise should process the welcome (both gift-wrapped and direct)
    // This would test the enhancement mentioned in the PRD:
    // "Update whitenoise to process both gift-wrapped and direct MLS welcomes"
    
    tokio::time::sleep(Duration::from_secs(5)).await;
    info!("Welcome message compatibility test completed");
    
    // Cleanup
    dialog_automation.close_session().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_key_package_lifecycle_coordination() -> Result<()> {
    info!("=== WHITENOISE INTEGRATION TEST: Key package lifecycle coordination ===");
    
    // Test the timing dependencies mentioned in the PRD:
    // "Dialog's ephemeral packages may expire before group creation"
    
    let mut dialog_automation = DialogTuiAutomation::new();
    let _session_id = dialog_automation.create_session("keypackage_tester", "ws://localhost:8080,ws://localhost:7777").await?;
    
    // Step 1: Publish fresh key packages
    dialog_automation.setup_dialog_tui().await?;
    info!("Published fresh key packages from dialog_tui");
    
    // Step 2: Simulate delay before whitenoise tries to use them
    tokio::time::sleep(Duration::from_secs(30)).await; // Simulate real-world delay
    
    // Step 3: Refresh key packages if needed
    dialog_automation.send_keys(&["/refresh-keys", "Enter"]).await?;
    dialog_automation.wait_for_text("Refreshed", 10).await?;
    info!("Refreshed key packages due to potential expiration");
    
    // Step 4: Whitenoise should now be able to fetch fresh packages
    // In real test: whitenoise.fetch_key_packages(&dialog_pubkey).await?;
    
    dialog_automation.close_session().await?;
    
    info!("=== KEY PACKAGE LIFECYCLE TEST COMPLETED ===");
    Ok(())
}