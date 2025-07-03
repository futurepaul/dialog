use anyhow::Result;
use nostr_mls::prelude::*;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_sdk::prelude::*;
use tempfile::TempDir;

#[tokio::test]
async fn mls_message_before_accept() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let alice_db = TempDir::new()?.path().join("alice_pre.db");
    let bob_db = TempDir::new()?.path().join("bob_pre.db");

    let alice_keys = Keys::generate();
    let bob_keys = Keys::generate();

    let mut alice_mls = NostrMls::new(NostrMlsSqliteStorage::new(alice_db.clone()).unwrap());
    let mut bob_mls = NostrMls::new(NostrMlsSqliteStorage::new(bob_db.clone()).unwrap());

    let relay = RelayUrl::parse("ws://localhost:8080")?;

    let (bob_kp_encoded, bob_kp_tags) = bob_mls.create_key_package_for_event(&bob_keys.public_key(), [relay.clone()])?;
    let bob_kp_event = EventBuilder::new(Kind::MlsKeyPackage, bob_kp_encoded)
        .tags(bob_kp_tags)
        .sign_with_keys(&bob_keys)?;

    let create_res = alice_mls.create_group(
        &alice_keys.public_key(),
        vec![bob_kp_event.clone()],
        vec![alice_keys.public_key(), bob_keys.public_key()],
        NostrGroupConfigData::new("pre-message-test".into(), "".into(), None, None, vec![relay]),
    )?;
    let gid = create_res.group.mls_group_id.clone();

    // Alice sends message BEFORE Bob processes welcome
    let msg_event = alice_mls.create_message(&gid, EventBuilder::new(Kind::TextNote, "hi before").build(alice_keys.public_key()))?;

    // Now deliver welcomes
    for rumor in &create_res.welcome_rumors {
        let gw = EventBuilder::gift_wrap(&alice_keys, &bob_keys.public_key(), rumor.clone(), None).await?;
        bob_mls.process_welcome(&gw.id, rumor)?;
    }

    let pending = bob_mls.get_pending_welcomes()?.into_iter().find(|w| w.mls_group_id == gid).unwrap();
    bob_mls.accept_welcome(&pending)?;

    // Bob processes message
    bob_mls.process_message(&msg_event)?;

    let msgs = bob_mls.get_messages(&gid)?;
    assert!(msgs.iter().any(|m| m.content.contains("hi before")), "Bob failed to decrypt message sent before accept");

    Ok(())
}