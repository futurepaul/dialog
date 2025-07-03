use anyhow::Result;
use nostr_mls::prelude::*;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_sdk::prelude::*;
use tempfile::TempDir;

#[tokio::test]
async fn mls_bob_sends_first() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    // Temporary SQLite DBs
    let alice_dir = TempDir::new()?;
    let bob_dir = TempDir::new()?;

    let alice_db = alice_dir.path().join("alice_bob_first.db");
    let bob_db = bob_dir.path().join("bob_bob_first.db");

    // Key pairs
    let alice_keys = Keys::generate();
    let bob_keys = Keys::generate();

    // MLS instances
    let mut alice_mls = NostrMls::new(NostrMlsSqliteStorage::new(alice_db.clone())?);
    let mut bob_mls = NostrMls::new(NostrMlsSqliteStorage::new(bob_db.clone())?);

    let relay_url = RelayUrl::parse("ws://localhost:8080")?;

    // Bob publishes key package locally for Alice to use
    let (bob_kp_encoded, bob_kp_tags) = bob_mls
        .create_key_package_for_event(&bob_keys.public_key(), [relay_url.clone()])?;
    let bob_kp_event = EventBuilder::new(Kind::MlsKeyPackage, bob_kp_encoded)
        .tags(bob_kp_tags)
        .sign_with_keys(&bob_keys)?;

    // Alice creates group with Bob
    let admins = vec![alice_keys.public_key(), bob_keys.public_key()];
    let config = NostrGroupConfigData::new(
        "bob-first".to_string(),
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

    // Deliver welcome rumour(s) to Bob and accept
    for rumor in &create_res.welcome_rumors {
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
        .expect("missing welcome");
    bob_mls.accept_welcome(&pending)?;

    // Bob sends a message to the group
    let plaintext = EventBuilder::new(Kind::TextNote, "hello from bob").build(bob_keys.public_key());
    let msg_event = bob_mls.create_message(&group_id, plaintext)?;

    // Bob processes his own message
    bob_mls.process_message(&msg_event)?;

    // Verify Bob can decrypt his own message
    let msgs = bob_mls.get_messages(&group_id)?;
    assert!(msgs.iter().any(|m| m.content.contains("hello from bob")), "Bob failed to decrypt his own message");

    // (Optional) deliver to Alice and see if she can decrypt
    let alice_result = alice_mls.process_message(&msg_event);
    assert!(alice_result.is_ok(), "Alice failed to process Bob's message: {:?}", alice_result.err());
    let alice_msgs = alice_mls.get_messages(&group_id)?;
    assert!(alice_msgs.iter().any(|m| m.content.contains("hello from bob")), "Alice failed to decrypt Bob's message");

    Ok(())
}