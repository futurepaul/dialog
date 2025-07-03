use anyhow::Result;
use nostr_mls::prelude::*;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_mls_storage::groups::GroupStorage;
use nostr_sdk::prelude::*;
use tempfile::TempDir;

#[tokio::test]
async fn mls_storage_roundtrip() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    // Setup temp DBs
    let bob_dir = TempDir::new()?;
    let alice_dir = TempDir::new()?;

    let bob_db = bob_dir.path().join("bob_round.db");
    let alice_db = alice_dir.path().join("alice_round.db");

    let alice_keys = Keys::generate();
    let bob_keys = Keys::generate();

    // MLS instances
    let mut alice_mls = NostrMls::new(NostrMlsSqliteStorage::new(alice_db.clone())?);
    let mut bob_mls = NostrMls::new(NostrMlsSqliteStorage::new(bob_db.clone())?);

    let relay_url = RelayUrl::parse("ws://localhost:8080")?;

    // Bob key package for Alice
    let (bob_kp_encoded, bob_kp_tags) = bob_mls
        .create_key_package_for_event(&bob_keys.public_key(), [relay_url.clone()])?;
    let bob_kp_event = EventBuilder::new(Kind::MlsKeyPackage, bob_kp_encoded)
        .tags(bob_kp_tags)
        .sign_with_keys(&bob_keys)?;

    // Alice creates group with Bob
    let admins = vec![alice_keys.public_key(), bob_keys.public_key()];
    let config = NostrGroupConfigData::new(
        "roundtrip".to_string(),
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

    // Deliver welcome(s) to Bob and accept
    for rumor in &create_res.welcome_rumors {
        let gw_event = EventBuilder::gift_wrap(&alice_keys, &bob_keys.public_key(), rumor.clone(), None).await?;
        bob_mls.process_welcome(&gw_event.id, rumor)?;
    }
    let pending = bob_mls
        .get_pending_welcomes()?
        .into_iter()
        .find(|w| w.mls_group_id == group_id)
        .expect("missing welcome");
    bob_mls.accept_welcome(&pending)?;

    // Bob sends a message to force secret export and storage
    let plaintext = EventBuilder::new(Kind::TextNote, "msg before reload").build(bob_keys.public_key());
    let msg_event = bob_mls.create_message(&group_id, plaintext)?;
    bob_mls.process_message(&msg_event)?;

    // Confirm secret now exists via direct storage query
    let bob_storage_check = NostrMlsSqliteStorage::new(bob_db.clone())?;
    let secret_epoch1 = bob_storage_check.get_group_exporter_secret(&group_id, 1)?;
    assert!(secret_epoch1.is_some(), "Exporter secret not persisted after message create/process");

    // Drop bob_mls to simulate program restart
    drop(bob_mls);

    // Reload Bob MLS instance
    let mut bob_mls2 = NostrMls::new(NostrMlsSqliteStorage::new(bob_db.clone())?);

    // Bob creates second message to ensure secret usable
    let plaintext2 = EventBuilder::new(Kind::TextNote, "msg after reload").build(bob_keys.public_key());
    let msg_event2 = bob_mls2.create_message(&group_id, plaintext2)?; // should succeed using persisted secret
    bob_mls2.process_message(&msg_event2)?;

    // Verify Bob decrypt message2
    let msgs = bob_mls2.get_messages(&group_id)?;
    assert!(msgs.iter().any(|m| m.content.contains("msg after reload")), "Bob failed to decrypt after reload");

    Ok(())
}