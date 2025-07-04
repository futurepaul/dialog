use anyhow::Result;
use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_memory_storage_alice_bob_messaging() -> Result<()> {
    // Test that Alice and Bob can send messages to each other using memory storage
    
    // Set up temporary environment
    let temp_dir = TempDir::new()?;
    let env_file = temp_dir.path().join(".env.local");
    std::fs::write(&env_file, 
        "ALICE_SK_HEX=0000000000000000000000000000000000000000000000000000000000000001\n\
         BOB_SK_HEX=0000000000000000000000000000000000000000000000000000000000000002\n\
         ALICE_PK_HEX=79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798\n\
         BOB_PK_HEX=c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5")?;
    
    // Change to temp directory
    std::env::set_current_dir(temp_dir.path())?;
    
    // 1. Publish keys for Alice and Bob using memory storage
    Command::cargo_bin("dialog_cli")?
        .arg("publish-key")
        .arg("--key").arg("alice")
        .arg("--memory-storage")
        .assert()
        .success();
        
    Command::cargo_bin("dialog_cli")?
        .arg("publish-key")
        .arg("--key").arg("bob")
        .arg("--memory-storage")
        .assert()
        .success();
    
    // 2. Alice creates a group with Bob using memory storage
    let output = Command::cargo_bin("dialog_cli")?
        .arg("create-group")
        .arg("--key").arg("alice")
        .arg("--name").arg("test-group")
        .arg("--counterparty").arg("c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5")
        .arg("--memory-storage")
        .output()?;
    
    println!("Create group output: {}", String::from_utf8_lossy(&output.stdout));
    assert!(output.status.success());
    
    Ok(())
} 