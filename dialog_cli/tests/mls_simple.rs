use anyhow::Result;
use nostr_mls::prelude::*;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_sdk::prelude::*;
use tempfile::TempDir;
use tracing::info;

#[tokio::test]
async fn mls_simple_send_receive() -> Result<()> {
    // Ensure tracing logs appear when running `--nocapture`
    let _ = tracing_subscriber::fmt::try_init();

    // Setup temporary SQLite DBs
    let alice_dir = TempDir::new()?;
    let bob_dir = TempDir::new()?;

    let alice_db = alice_dir.path().join("alice_simple.db");
    let bob_db = bob_dir.path().join("bob_simple.db");

    // Keypairs
    let alice_keys = Keys::generate();
    let bob_keys = Keys::generate();

    // MLS instances
    let alice_storage = NostrMlsSqliteStorage::new(alice_db.clone()).unwrap();
    let mut alice_mls = NostrMls::new(alice_storage);

    let bob_storage = NostrMlsSqliteStorage::new(bob_db.clone()).unwrap();
    let mut bob_mls = NostrMls::new(bob_storage);

    let relay_url = RelayUrl::parse("ws://localhost:8080")?;

    // Bob publishes key package
    let (bob_kp_encoded, bob_kp_tags) = bob_mls
        .create_key_package_for_event(&bob_keys.public_key(), [relay_url.clone()])?;
    let bob_kp_event = EventBuilder::new(Kind::MlsKeyPackage, bob_kp_encoded)
        .tags(bob_kp_tags)
        .sign_with_keys(&bob_keys)?;

    // Alice creates group with Bob
    let admins = vec![alice_keys.public_key(), bob_keys.public_key()];
    let config = NostrGroupConfigData::new(
        "simple-test".to_string(),
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

    println!("create_res debug: {:?}", create_res);

    let group_id = create_res.group.mls_group_id.clone();
    let welcome_rumors = create_res.welcome_rumors.clone();
    info!("Group created at epoch {}", create_res.group.epoch);

    // Deliver welcomes to Bob
    for rumor in &welcome_rumors {
        let gw_event = EventBuilder::gift_wrap(
            &alice_keys,
            &bob_keys.public_key(),
            rumor.clone(),
            None,
        )
        .await?;
        bob_mls.process_welcome(&gw_event.id, rumor)?;
    }

    let pending = bob_mls
        .get_pending_welcomes()?
        .into_iter()
        .find(|w| w.mls_group_id == group_id)
        .expect("welcome missing");
    bob_mls.accept_welcome(&pending)?;

    // Log group epochs before sending message
    let alice_epoch_before = alice_mls
        .get_groups()?
        .into_iter()
        .find(|g| g.mls_group_id == group_id)
        .map(|g| g.epoch)
        .unwrap_or_default();
    let bob_epoch_before = bob_mls
        .get_groups()?
        .into_iter()
        .find(|g| g.mls_group_id == group_id)
        .map(|g| g.epoch)
        .unwrap_or_default();
    info!(
        "Epochs before message: Alice={}, Bob={}",
        alice_epoch_before, bob_epoch_before
    );

    println!("epochs: alice={}, bob={} (before message)", alice_epoch_before, bob_epoch_before);

    // Alice sends message
    let plaintext = EventBuilder::new(Kind::TextNote, "hello simple").build(alice_keys.public_key());
    let msg_event = alice_mls.create_message(&group_id, plaintext)?;

    // Sender must also process their own message to stay in sync (see CLI)
    alice_mls.process_message(&msg_event)?;

    println!("msg kind {:?}", msg_event.kind);

    // Bob processes
    bob_mls.process_message(&msg_event)?;

    let msgs = bob_mls.get_messages(&group_id)?;
    assert!(msgs.iter().any(|m| m.content.contains("hello simple")), "Bob failed to decrypt in simple test");

    Ok(())
}