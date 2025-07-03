use anyhow::Result;
use nostr_mls::prelude::*;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_mls_storage::groups::GroupStorage;
use nostr_sdk::prelude::*;
use tempfile::TempDir;

#[tokio::test]
async fn mls_exporter_secret_presence() -> Result<()> {
    // Initialize tracing for logs when running with `--nocapture`
    let _ = tracing_subscriber::fmt::try_init();

    // Temporary SQLite DBs for Alice and Bob
    let alice_dir = TempDir::new()?;
    let bob_dir = TempDir::new()?;

    let alice_db = alice_dir.path().join("alice_exporter.db");
    let bob_db = bob_dir.path().join("bob_exporter.db");

    // Generate key pairs
    let alice_keys = Keys::generate();
    let bob_keys = Keys::generate();

    // Create MLS instances
    let mut alice_mls = NostrMls::new(NostrMlsSqliteStorage::new(alice_db.clone())?);
    let mut bob_mls = NostrMls::new(NostrMlsSqliteStorage::new(bob_db.clone())?);

    let relay_url = RelayUrl::parse("ws://localhost:8080")?;

    // Bob publishes his key-package locally
    let (bob_kp_encoded, bob_kp_tags) = bob_mls
        .create_key_package_for_event(&bob_keys.public_key(), [relay_url.clone()])?;
    let bob_kp_event = EventBuilder::new(Kind::MlsKeyPackage, bob_kp_encoded)
        .tags(bob_kp_tags)
        .sign_with_keys(&bob_keys)?;

    // Alice creates the group and includes Bob
    let admins = vec![alice_keys.public_key(), bob_keys.public_key()];
    let config = NostrGroupConfigData::new(
        "exporter-test".to_string(),
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

    // Bob accepts the welcome
    let pending = bob_mls
        .get_pending_welcomes()?
        .into_iter()
        .find(|w| w.mls_group_id == group_id)
        .expect("missing welcome");
    bob_mls.accept_welcome(&pending)?;

    // Determine Bob's current epoch for the group
    let bob_epoch = bob_mls
        .get_groups()?
        .into_iter()
        .find(|g| g.mls_group_id == group_id)
        .map(|g| g.epoch)
        .expect("Bob should have group after accept_welcome");

    // Re-open storage directly to inspect exporter secret table
    let bob_storage_check = NostrMlsSqliteStorage::new(bob_db.clone())?;
    let secret_opt = bob_storage_check.get_group_exporter_secret(&group_id, bob_epoch)?;

    assert!(secret_opt.is_some(), "Exporter secret missing for epoch {} immediately after accept_welcome", bob_epoch);

    if let Some(sec) = secret_opt {
        assert_eq!(sec.secret.len(), 32, "Exporter secret should be 32 bytes");
    }

    Ok(())
}