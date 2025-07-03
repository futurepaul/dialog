use anyhow::Result;
use nostr_mls::prelude::*;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_sdk::prelude::*;
use tempfile::TempDir;
use tracing::info;

#[tokio::test]
async fn mls_persistence_storage_test() -> Result<()> {
    // Setup temporary directories for Alice and Bob storage
    let alice_dir = TempDir::new()?;
    let bob_dir = TempDir::new()?;

    let alice_db = alice_dir.path().join("alice.db");
    let bob_db = bob_dir.path().join("bob.db");

    // Initialize Alice and Bob identities
    let alice_keys = Keys::generate();
    let bob_keys = Keys::generate();

    // Create initial NostrMls instances
    let alice_storage = NostrMlsSqliteStorage::new(alice_db.clone()).unwrap();
    let mut alice_mls = NostrMls::new(alice_storage);

    let bob_storage = NostrMlsSqliteStorage::new(bob_db.clone()).unwrap();
    let mut bob_mls = NostrMls::new(bob_storage);

    // Bob publishes a key-package event (locally, without a relay)
    let relay_url = RelayUrl::parse("ws://localhost:8080").unwrap();
    let (bob_kp_encoded, bob_kp_tags) = bob_mls
        .create_key_package_for_event(&bob_keys.public_key(), [relay_url.clone()])?;

    let bob_kp_event = EventBuilder::new(Kind::MlsKeyPackage, bob_kp_encoded)
        .tags(bob_kp_tags)
        .sign_with_keys(&bob_keys)
        .map_err(|e| anyhow::anyhow!(e))?;

    // Alice creates a group with Bob
    let admins = vec![alice_keys.public_key(), bob_keys.public_key()];
    let config = NostrGroupConfigData::new(
        "persistence-test".to_string(),
        "".to_string(),
        None,
        None,
        vec![relay_url],
    );

    let create_res = alice_mls.create_group(
        &alice_keys.public_key(),
        vec![bob_kp_event.clone()],
        admins,
        config,
    )?;

    let group_id = create_res.group.mls_group_id.clone();
    let welcome_rumors = create_res.welcome_rumors.clone();
    info!("Group created at epoch {}", create_res.group.epoch);

    // Simulate Alice process termination by dropping the instance
    drop(alice_mls);

    // Re-instantiate Alice's MLS from the same SQLite storage (fresh process)
    let alice_storage_2 = NostrMlsSqliteStorage::new(alice_db.clone()).unwrap();
    let mut alice_mls = NostrMls::new(alice_storage_2);

    // Alice constructs a plaintext text note and encrypts it for the group
    let plaintext_event = EventBuilder::new(Kind::TextNote, "hello persistence")
        .build(alice_keys.public_key());

    let msg_event = alice_mls.create_message(&group_id, plaintext_event)?;

    // Deliver the welcome(s) to Bob and have him process them
    for rumor in &welcome_rumors {
        // Gift-wrap like the CLI does so we get a realistic EventId
        let gw_event = EventBuilder::gift_wrap(&alice_keys, &bob_keys.public_key(), rumor.clone(), None)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        bob_mls.process_welcome(&gw_event.id, rumor)?;
    }

    // Bob accepts the invite
    let pending = bob_mls
        .get_pending_welcomes()?
        .into_iter()
        .find(|w| w.mls_group_id == group_id)
        .expect("welcome not found");
    bob_mls.accept_welcome(&pending)?;

    // Bob processes Alice's message
    bob_mls.process_message(&msg_event)?;

    // Verify Bob decrypted the message
    let msgs = bob_mls.get_messages(&group_id)?;
    assert!(msgs.iter().any(|m| m.content.contains("hello persistence")), "Bob failed to decrypt Alice's message after persistence reload");

    // Additionally, assert both parties agree on epoch
    let alice_group = alice_mls
        .get_groups()?
        .into_iter()
        .find(|g| g.mls_group_id == group_id)
        .expect("Alice group missing");
    let bob_group = bob_mls
        .get_groups()?
        .into_iter()
        .find(|g| g.mls_group_id == group_id)
        .expect("Bob group missing");

    assert_eq!(alice_group.epoch, bob_group.epoch, "Epoch divergence after persistence reload");

    Ok(())
}