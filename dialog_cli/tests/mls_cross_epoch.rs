use anyhow::Result;
use nostr_mls::prelude::*;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_sdk::prelude::*;
use tempfile::TempDir;

#[tokio::test]
async fn mls_cross_epoch() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    // Prepare temp DBs
    let alice_dir = TempDir::new()?;
    let bob_dir = TempDir::new()?;
    let alice_db = alice_dir.path().join("alice_cross_epoch.db");
    let bob_db = bob_dir.path().join("bob_cross_epoch.db");

    // Keys
    let alice_keys = Keys::generate();
    let bob_keys = Keys::generate();

    // Instances
    let mut alice_mls = NostrMls::new(NostrMlsSqliteStorage::new(alice_db.clone())?);
    let mut bob_mls = NostrMls::new(NostrMlsSqliteStorage::new(bob_db.clone())?);

    let relay_url = RelayUrl::parse("ws://localhost:8080")?;

    // Bob key package
    let (bob_kp_encoded, bob_kp_tags) = bob_mls
        .create_key_package_for_event(&bob_keys.public_key(), [relay_url.clone()])?;
    let bob_kp_event = EventBuilder::new(Kind::MlsKeyPackage, bob_kp_encoded)
        .tags(bob_kp_tags)
        .sign_with_keys(&bob_keys)?;

    // Alice creates group
    let admins = vec![alice_keys.public_key(), bob_keys.public_key()];
    let config = NostrGroupConfigData::new(
        "cross-epoch".to_string(),
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

    // Deliver welcome(s) to Bob
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

    // Ensure both at epoch 1
    let alice_epoch1 = alice_mls.get_groups()?.iter().find(|g| g.mls_group_id == group_id).unwrap().epoch;
    let bob_epoch1 = bob_mls.get_groups()?.iter().find(|g| g.mls_group_id == group_id).unwrap().epoch;
    assert_eq!(alice_epoch1, 1);
    assert_eq!(bob_epoch1, 1);

    // ---- Epoch change: Alice self_update ----
    let update_res = alice_mls.self_update(&group_id)?;
    // Alice's commit event encrypted with epoch1 secret
    let commit_event = update_res.evolution_event.clone();

    // (Alice publishes) She must merge her own pending commit locally
    alice_mls.merge_pending_commit(&group_id)?;

    // Bob processes commit event
    let bob_process_res = bob_mls.process_message(&commit_event);
    assert!(bob_process_res.is_ok(), "Bob failed to process commit event: {:?}", bob_process_res.err());
    // Merge Bob's pending as well
    bob_mls.merge_pending_commit(&group_id)?;

    // Check epochs advanced to 2
    let alice_epoch2 = alice_mls.get_groups()?.iter().find(|g| g.mls_group_id == group_id).unwrap().epoch;
    let bob_epoch2 = bob_mls.get_groups()?.iter().find(|g| g.mls_group_id == group_id).unwrap().epoch;
    assert_eq!(alice_epoch2, 2, "Alice epoch not 2 after self_update");
    assert_eq!(bob_epoch2, 2, "Bob epoch not 2 after processing commit");

    // Alice sends a message at epoch2
    let plaintext2 = EventBuilder::new(Kind::TextNote, "hello epoch2").build(alice_keys.public_key());
    let msg_event2 = alice_mls.create_message(&group_id, plaintext2)?;
    alice_mls.process_message(&msg_event2)?; // Alice processes own message

    // Bob processes epoch2 message
    let bob_msg_res = bob_mls.process_message(&msg_event2);
    assert!(bob_msg_res.is_ok(), "Bob failed to decrypt epoch2 message: {:?}", bob_msg_res.err());
    let msgs = bob_mls.get_messages(&group_id)?;
    assert!(msgs.iter().any(|m| m.content.contains("hello epoch2")), "Bob could not decrypt epoch2 content");

    Ok(())
}