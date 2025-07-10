use tokio::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use std::process::Stdio;

#[tokio::test]
async fn test_tui_cli_basic_interop() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Automated Dialog TUI-CLI Interop Test ===");
    
    // 1. Start local relay
    println!("Starting relay...");
    let mut relay = Command::new("nak")
        .args(&["serve", "--verbose"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    
    sleep(Duration::from_secs(2)).await;
    
    // 2. Start dialog_tui in background with expect script
    println!("Starting dialog_tui with automation...");
    let tui_script = r#"#!/usr/bin/expect -f
set timeout 30
spawn cargo run --bin dialog_tui -- --key alice
expect "Connected to relay successfully"
expect "Published 5 key packages"

# Handle invite
expect "New group invitation received" {
    send "/invites\r"
    expect "Test Interop Group"
    send "\r"
    expect "Successfully joined group"
}

# Send message when we see CLI message
expect "75427ab8...: Hello from CLI" {
    send "Hello from TUI!\r"
}

# Keep alive for message verification
sleep 10
"#;
    
    // Write expect script to temp file
    let temp_script = "/tmp/dialog_tui_test.expect";
    std::fs::write(temp_script, tui_script)?;
    
    let mut tui = Command::new("expect")
        .arg("-f")
        .arg(temp_script)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    
    // Wait for TUI to be ready
    sleep(Duration::from_secs(5)).await;
    
    // 3. Get Alice's pubkey using CLI
    println!("Getting Alice's public key...");
    let pubkey_output = Command::new("cargo")
        .args(&["run", "--bin", "dialog_cli", "--", "get-pubkey", "--key", "alice"])
        .stderr(Stdio::null())
        .output()
        .await?;
    
    let alice_pubkey = String::from_utf8(pubkey_output.stdout)?
        .lines()
        .find(|line| line.len() == 64 && line.chars().all(|c| c.is_ascii_hexdigit()))
        .expect("Could not find Alice's pubkey")
        .to_string();
    
    println!("Alice pubkey: {}", alice_pubkey);
    
    // 4. Bob publishes key packages
    println!("Publishing Bob's key packages...");
    Command::new("cargo")
        .args(&["run", "--bin", "dialog_cli", "--", "publish-key", "--key", "bob"])
        .stderr(Stdio::null())
        .output()
        .await?;
    
    // 5. Bob creates group and invites Alice
    println!("Creating group and inviting Alice...");
    let create_output = Command::new("cargo")
        .args(&[
            "run", "--bin", "dialog_cli", "--",
            "create-group", "--key", "bob",
            "--name", "Test Interop Group",
            "--counterparty", &alice_pubkey
        ])
        .stderr(Stdio::null())
        .output()
        .await?;
    
    let output_str = String::from_utf8(create_output.stdout)?;
    let group_id = output_str
        .lines()
        .find(|line| line.contains("Group ID:"))
        .and_then(|line| line.split_whitespace().last())
        .expect("Could not find group ID");
    
    println!("Created group: {}", group_id);
    
    // 6. Bob sends message
    println!("Bob sending message...");
    Command::new("cargo")
        .args(&[
            "run", "--bin", "dialog_cli", "--",
            "send-message", "--key", "bob",
            "--group-id", group_id,
            "--message", "Hello from CLI!"
        ])
        .stderr(Stdio::null())
        .output()
        .await?;
    
    // Wait for TUI to respond
    sleep(Duration::from_secs(3)).await;
    
    // 7. Bob fetches messages to verify TUI response
    println!("Fetching messages to verify...");
    let messages_output = Command::new("cargo")
        .args(&[
            "run", "--bin", "dialog_cli", "--",
            "get-messages", "--key", "bob",
            "--group-id", group_id
        ])
        .stderr(Stdio::null())
        .output()
        .await?;
    
    let messages = String::from_utf8(messages_output.stdout)?;
    println!("Messages:\n{}", messages);
    
    // Verify both messages exist
    assert!(messages.contains("Hello from CLI!"), "CLI message not found");
    assert!(messages.contains("Hello from TUI!"), "TUI response not found");
    
    // Cleanup
    relay.kill().await.ok();
    tui.kill().await.ok();
    std::fs::remove_file(temp_script).ok();
    
    println!("✅ Dialog TUI-CLI interop test passed!");
    Ok(())
}

#[tokio::test]
#[ignore] // Run with: cargo test --package integration --test tui_cli_interop test_quick_smoke -- --ignored
async fn test_quick_smoke() -> Result<(), Box<dyn std::error::Error>> {
    // Quick smoke test that just verifies binaries compile and run
    println!("Running quick smoke test...");
    
    // Test dialog_cli help
    let help_output = Command::new("cargo")
        .args(&["run", "--bin", "dialog_cli", "--", "--help"])
        .output()
        .await?;
    
    assert!(help_output.status.success(), "dialog_cli --help failed");
    
    // Test dialog_tui help (it doesn't have --help but we can check it builds)
    let build_output = Command::new("cargo")
        .args(&["build", "--bin", "dialog_tui"])
        .output()
        .await?;
    
    assert!(build_output.status.success(), "dialog_tui build failed");
    
    println!("✅ Quick smoke test passed!");
    Ok(())
}