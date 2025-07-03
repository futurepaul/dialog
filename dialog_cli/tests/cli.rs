/*
MLS EPOCH SYNCHRONIZATION TEST ANALYSIS - UPDATED
==================================================

MAJOR FINDING:
Even when Alice creates the group AND sends the message in the SAME CLI process,
Bob still cannot decrypt the message. This rules out several hypotheses:

❌ RULED OUT: Separate CLI process issue
❌ RULED OUT: Database state consistency between processes  
❌ RULED OUT: Network timing affecting state synchronization

CONFIRMED ISSUE:
The problem is fundamental to the MLS group state generation, not process isolation.

COMPARISON WITH WORKING CODE (prompts/mls_sqlite.rs):
The key difference is that the working test operates entirely in memory with:
- Same NostrMls instance for group creation and message creation
- No database persistence between operations
- No relay communication between group creation and message creation
- Direct object passing instead of CLI argument parsing

vs our CLI which:
- Creates new NostrMls instance for each command
- Persists state to SQLite database between commands  
- Communicates through relay for all operations
- Recreates group state from database for each operation

NEW HYPOTHESIS:
The issue might be in how the NostrMls state is persisted/restored from SQLite:
1. Alice creates group (state saved to SQLite)
2. Alice sends message (new NostrMls instance, loads state from SQLite)
3. The loaded state might be different from the original state

OR:

The welcome message Bob receives might not contain the correct epoch 1 state
that matches Alice's final epoch 1 state after merge_pending_commit().

NEXT INVESTIGATION:
1. Check if exporter secrets are correctly persisted/restored
2. Compare the actual exporter secrets between Alice and Bob
3. Verify if the welcome message contains the right epoch state
4. Test with in-memory storage to isolate SQLite persistence issues
*/

use anyhow::Result;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn get_bob_pubkey() -> Result<String> {
    dotenv::from_filename(".env.local").ok();
    let bob_pk = std::env::var("BOB_PK_HEX")?;
    Ok(bob_pk)
}

#[test]
fn mls_epoch_test() -> Result<()> {
    // Clear any existing state
    std::process::Command::new("rm")
        .args(["-rf", ".dialog_cli_data"])
        .output()?;

    // 1. Publish keys for alice and bob
    Command::cargo_bin("dialog_cli")?
        .arg("publish-key")
        .arg("--key")
        .arg("alice")
        .assert()
        .success();

    Command::cargo_bin("dialog_cli")?
        .arg("publish-key")
        .arg("--key")
        .arg("bob")
        .assert()
        .success();

    // 2. Alice creates a group with Bob
    let bob_pubkey = get_bob_pubkey()?;

    let output = Command::cargo_bin("dialog_cli")?
        .arg("create-group")
        .arg("--key")
        .arg("alice")
        .arg("--name")
        .arg("epoch-test-group")
        .arg("--counterparty")
        .arg(&bob_pubkey)
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    println!("Create-group output:");
    println!("{}", stdout);
    
    let group_id = stdout
        .lines()
        .find(|line| line.contains("Group ID"))
        .and_then(|line| line.split_whitespace().last())
        .expect("Could not find group ID in output")
        .to_string();

    println!("Created group with ID: {}", group_id);

    // 3. Alice sends a message BEFORE Bob accepts the invite
    let alice_output = Command::cargo_bin("dialog_cli")?
        .arg("send-message")
        .arg("--key")
        .arg("alice")
        .arg("--group-id")
        .arg(&group_id)
        .arg("--message")
        .arg("epoch-test")
        .output()?;

    println!("Alice's send-message output:");
    println!("{}", String::from_utf8_lossy(&alice_output.stdout));
    if !alice_output.stderr.is_empty() {
        println!("Alice stderr: {}", String::from_utf8_lossy(&alice_output.stderr));
    }
    assert!(alice_output.status.success(), "Alice's send-message failed");

    println!("Alice sent message to group: {}", group_id);

    // 4. NOW Bob accepts the invite (after Alice sent the message)
    Command::cargo_bin("dialog_cli")?
        .arg("list-invites")
        .arg("--key")
        .arg("bob")
        .assert()
        .success()
        .stdout(predicate::str::contains(&group_id));

    Command::cargo_bin("dialog_cli")?
        .arg("accept-invite")
        .arg("--key")
        .arg("bob")
        .arg("--group-id")
        .arg(&group_id)
        .assert()
        .success();

    println!("Bob joined group: {}", group_id);

    // 5. Bob should be able to decrypt the message Alice sent before he joined
    let result = Command::cargo_bin("dialog_cli")?
        .arg("get-messages")
        .arg("--key")
        .arg("bob")
        .arg("--group-id")
        .arg(&group_id)
        .output()?;

    println!("Bob's get-messages output:");
    println!("{}", String::from_utf8_lossy(&result.stdout));
    if !result.stderr.is_empty() {
        println!("Stderr: {}", String::from_utf8_lossy(&result.stderr));
    }

    // If this fails, we'll see exactly what the issue is
    assert!(result.status.success(), "get-messages failed");
    assert!(
        String::from_utf8_lossy(&result.stdout).contains("epoch-test"),
        "Bob should see Alice's message"
    );

    Ok(())
}

#[test]
fn ping_pong_test() -> Result<()> {
    // NOTE: This test assumes a local relay is running at ws://localhost:8080
    // and that a .env.local file exists with BOB_PK_HEX.

    // 1. Publish keys for alice and bob
    Command::cargo_bin("dialog_cli")?
        .arg("publish-key")
        .arg("--key")
        .arg("alice")
        .assert()
        .success();

    Command::cargo_bin("dialog_cli")?
        .arg("publish-key")
        .arg("--key")
        .arg("bob")
        .assert()
        .success();

    // give relay time to process
    std::thread::sleep(std::time::Duration::from_secs(1));

    // 2. Alice creates a group with Bob.
    let bob_pubkey = get_bob_pubkey()?;

    let output = Command::cargo_bin("dialog_cli")?
        .arg("create-group")
        .arg("--key")
        .arg("alice")
        .arg("--name")
        .arg("ping-pong-group")
        .arg("--counterparty")
        .arg(&bob_pubkey)
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    let group_id = stdout
        .lines()
        .find(|line| line.contains("Group ID"))
        .and_then(|line| line.split_whitespace().last())
        .expect("Could not find group ID in output")
        .to_string();

    // 3. Bob lists invites and accepts
    Command::cargo_bin("dialog_cli")?
        .arg("list-invites")
        .arg("--key")
        .arg("bob")
        .assert()
        .success()
        .stdout(predicate::str::contains(&group_id));

    Command::cargo_bin("dialog_cli")?
        .arg("accept-invite")
        .arg("--key")
        .arg("bob")
        .arg("--group-id")
        .arg(&group_id)
        .assert()
        .success();

    // give time for Bob to fully join
    std::thread::sleep(std::time::Duration::from_secs(1));

    // 4. Alice sends a "ping" message to the group.
    Command::cargo_bin("dialog_cli")?
        .arg("send-message")
        .arg("--key")
        .arg("alice")
        .arg("--group-id")
        .arg(&group_id)
        .arg("--message")
        .arg("ping")
        .assert()
        .success();
    
    // give relay time to process
    std::thread::sleep(std::time::Duration::from_secs(1));

    // 5. Bob gets messages and sees "ping"
    Command::cargo_bin("dialog_cli")?
        .arg("get-messages")
        .arg("--key")
        .arg("bob")
        .arg("--group-id")
        .arg(&group_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("ping"));

    // 6. Bob sends "pong"
    Command::cargo_bin("dialog_cli")?
        .arg("send-message")
        .arg("--key")
        .arg("bob")
        .arg("--group-id")
        .arg(&group_id)
        .arg("--message")
        .arg("pong")
        .assert()
        .success();

    // give relay time to process
    std::thread::sleep(std::time::Duration::from_secs(1));

    // 7. Alice gets messages and sees "pong"
    Command::cargo_bin("dialog_cli")?
        .arg("get-messages")
        .arg("--key")
        .arg("alice")
        .arg("--group-id")
        .arg(&group_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("pong"));

    Ok(())
}

#[test]
fn mls_same_process_test() -> Result<()> {
    // TEST: Create group and send message in same CLI process
    // This tests if the epoch issue is related to separate CLI invocations
    
    // Clear any existing state
    std::process::Command::new("rm")
        .args(["-rf", ".dialog_cli_data"])
        .output()?;

    // 1. Publish keys for alice and bob
    Command::cargo_bin("dialog_cli")?
        .arg("publish-key")
        .arg("--key")
        .arg("alice")
        .assert()
        .success();

    Command::cargo_bin("dialog_cli")?
        .arg("publish-key")
        .arg("--key")
        .arg("bob")
        .assert()
        .success();

    // 2. Alice creates group AND sends message in same process
    let bob_pubkey = get_bob_pubkey()?;

    let output = Command::cargo_bin("dialog_cli")?
        .arg("create-group-and-send")
        .arg("--key")
        .arg("alice")
        .arg("--name")
        .arg("same-process-test-group")
        .arg("--counterparty")
        .arg(&bob_pubkey)
        .arg("--message")
        .arg("same-process-test")
        .output()?;

    println!("Create-group-and-send output:");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.stderr.is_empty() {
        println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "create-group-and-send failed");

    let stdout = String::from_utf8(output.stdout)?;
    let group_id = stdout
        .lines()
        .find(|line| line.contains("Group ID"))
        .and_then(|line| line.split_whitespace().last())
        .expect("Could not find group ID in output")
        .to_string();

    println!("Created group with ID: {}", group_id);

    // 3. Bob accepts the invite
    Command::cargo_bin("dialog_cli")?
        .arg("list-invites")
        .arg("--key")
        .arg("bob")
        .assert()
        .success()
        .stdout(predicate::str::contains(&group_id));

    Command::cargo_bin("dialog_cli")?
        .arg("accept-invite")
        .arg("--key")
        .arg("bob")
        .arg("--group-id")
        .arg(&group_id)
        .assert()
        .success();

    println!("Bob joined group: {}", group_id);

    // 4. Bob should be able to decrypt the message Alice sent before he joined
    let result = Command::cargo_bin("dialog_cli")?
        .arg("get-messages")
        .arg("--key")
        .arg("bob")
        .arg("--group-id")
        .arg(&group_id)
        .output()?;

    println!("Bob's get-messages output:");
    println!("{}", String::from_utf8_lossy(&result.stdout));
    if !result.stderr.is_empty() {
        println!("Stderr: {}", String::from_utf8_lossy(&result.stderr));
    }

    // If this test passes, it means the issue is related to separate CLI processes
    assert!(result.status.success(), "get-messages failed");
    assert!(
        String::from_utf8_lossy(&result.stdout).contains("same-process-test"),
        "Bob should see Alice's message when sent in same process"
    );

    Ok(())
} 