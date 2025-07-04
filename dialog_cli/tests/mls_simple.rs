use anyhow::Result;
use nostr_mls::prelude::*;
use nostr_mls_memory_storage::NostrMlsMemoryStorage;
use nostr_sdk::{EventBuilder, Keys, Kind, RelayUrl, Timestamp, UnsignedEvent};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

#[tokio::test]
async fn mls_bob_joins_and_then_alice_sends_with_retry() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let alice_keys = Keys::generate();
    let alice_mls = NostrMls::new(NostrMlsMemoryStorage::default());
    let bob_keys = Keys::generate();
    let bob_mls = NostrMls::new(NostrMlsMemoryStorage::default());
    let relay_url = RelayUrl::parse("ws://localhost:8080")?;

    let (bob_kp_encoded, bob_kp_tags) =
        bob_mls.create_key_package_for_event(&bob_keys.public_key(), [relay_url.clone()])?;
    let bob_kp_event = EventBuilder::new(Kind::MlsKeyPackage, bob_kp_encoded)
        .tags(bob_kp_tags)
        .sign_with_keys(&bob_keys)?;

    let config = NostrGroupConfigData::new(
        "test-group".to_string(),
        "".to_string(),
        None,
        None,
        vec![relay_url],
    );
    let admins = vec![alice_keys.public_key(), bob_keys.public_key()];
    let create_res =
        alice_mls.create_group(&alice_keys.public_key(), vec![bob_kp_event], admins, config)?;
    let group_id = create_res.group.mls_group_id.clone();
    let welcome_rumor = create_res
        .welcome_rumors
        .into_iter()
        .next()
        .expect("Should have a welcome rumor");

    bob_mls.process_welcome(&EventId::all_zeros(), &welcome_rumor)?;
    let pending_welcome = bob_mls
        .get_pending_welcomes()?
        .into_iter()
        .find(|w| w.mls_group_id == group_id)
        .expect("Welcome not found for Bob");
    bob_mls.accept_welcome(&pending_welcome)?;
    info!("Bob joined group");

    let unsigned_event = UnsignedEvent::new(
        alice_keys.public_key(),
        Timestamp::now(),
        Kind::TextNote,
        vec![],
        "hello from alice".to_string(),
    );
    let message_event = alice_mls.create_message(&group_id, unsigned_event)?;
    alice_mls.process_message(&message_event)?;
    info!("Alice sent message");

    const MAX_RETRIES: u32 = 5;
    let mut success = false;
    for i in 0..MAX_RETRIES {
        info!("Bob attempting to process message (attempt {})...", i + 1);
        if bob_mls.process_message(&message_event).is_ok() {
            info!("Bob successfully processed message");
            success = true;
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }

    if !success {
        anyhow::bail!("Bob failed to process message after {} retries", MAX_RETRIES);
    }

    let messages = bob_mls.get_messages(&group_id)?;
    assert_eq!(messages.len(), 1, "Bob should have one message");
    assert_eq!(
        messages[0].content, "hello from alice",
        "Message content does not match"
    );
    info!("SUCCESS: Bob successfully decrypted Alice's message");

    Ok(())
}