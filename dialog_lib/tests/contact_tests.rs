mod test_helpers;

use dialog_lib::{DialogLib, Profile};
use test_helpers::{TestScenario, TestUser};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_add_contact_with_hex_pubkey() {
    let scenario = TestScenario::new(&["alice", "bob"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    let bob = scenario.get_user("bob").unwrap();
    
    let alice_dialog = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create Alice's DialogLib");
    
    alice_dialog.connect()
        .await
        .expect("Failed to connect Alice to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Add Bob as a contact using hex pubkey
    alice_dialog.add_contact(bob.pubkey_hex())
        .await
        .expect("Failed to add Bob as contact");
    
    // Get contacts and verify Bob was added
    let contacts = alice_dialog.get_contacts()
        .await
        .expect("Failed to get contacts");
    
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].pubkey, bob.keys().public_key());
    // Should use truncated pubkey with "(no profile)" since Bob hasn't published a profile
    assert!(contacts[0].name.ends_with("(no profile)"));
    assert!(contacts[0].name.starts_with(&bob.pubkey_hex()[0..8]));
}

#[tokio::test]
async fn test_add_contact_with_bech32_pubkey() {
    let scenario = TestScenario::new(&["alice", "bob"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    let bob = scenario.get_user("bob").unwrap();
    
    let alice_dialog = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create Alice's DialogLib");
    
    alice_dialog.connect()
        .await
        .expect("Failed to connect Alice to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Add Bob as a contact using bech32 pubkey
    alice_dialog.add_contact(bob.pubkey_bech32())
        .await
        .expect("Failed to add Bob as contact with bech32");
    
    // Get contacts and verify Bob was added
    let contacts = alice_dialog.get_contacts()
        .await
        .expect("Failed to get contacts");
    
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].pubkey, bob.keys().public_key());
}

#[tokio::test]
async fn test_add_invalid_pubkey() {
    let scenario = TestScenario::new(&["alice"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    
    let alice_dialog = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create Alice's DialogLib");
    
    alice_dialog.connect()
        .await
        .expect("Failed to connect Alice to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Try to add invalid pubkey
    let result = alice_dialog.add_contact("invalid_pubkey").await;
    
    // Should return an error
    assert!(result.is_err(), "Adding invalid pubkey should fail");
}

#[tokio::test]
async fn test_add_duplicate_contact() {
    let scenario = TestScenario::new(&["alice", "bob"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    let bob = scenario.get_user("bob").unwrap();
    
    let alice_dialog = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create Alice's DialogLib");
    
    alice_dialog.connect()
        .await
        .expect("Failed to connect Alice to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Add Bob as a contact
    alice_dialog.add_contact(bob.pubkey_hex())
        .await
        .expect("Failed to add Bob as contact");
    
    // Try to add Bob again
    let result = alice_dialog.add_contact(bob.pubkey_hex()).await;
    
    // Should either succeed (idempotent) or return a specific error
    match result {
        Ok(_) => {
            // If it succeeds, should still only have one contact
            let contacts = alice_dialog.get_contacts().await.unwrap();
            assert_eq!(contacts.len(), 1);
        }
        Err(_) => {
            // Error is also acceptable for duplicate contacts
        }
    }
}

#[tokio::test]
async fn test_contact_profile_loading_workflow() {
    let scenario = TestScenario::new(&["alice", "bob"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    let bob = scenario.get_user("bob").unwrap();
    
    // Create Bob's DialogLib and publish his profile
    let bob_dialog = DialogLib::new_with_keys_and_relay(
        bob.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create Bob's DialogLib");
    
    bob_dialog.connect()
        .await
        .expect("Failed to connect Bob to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Bob publishes his profile
    let bob_profile = Profile {
        display_name: Some("Bob Builder".to_string()),
        name: Some("bob".to_string()),
        about: Some("I build things!".to_string()),
        picture: Some("https://example.com/bob.jpg".to_string()),
        banner: None,
        website: None,
        lud16: None,
    };
    
    bob_dialog.publish_profile(&bob_profile)
        .await
        .expect("Failed to publish Bob's profile");
    
    sleep(Duration::from_millis(200)).await;
    
    // Create Alice's DialogLib
    let alice_dialog = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create Alice's DialogLib");
    
    alice_dialog.connect()
        .await
        .expect("Failed to connect Alice to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Alice adds Bob as a contact
    alice_dialog.add_contact(bob.pubkey_hex())
        .await
        .expect("Failed to add Bob as contact");
    
    // Load Bob's profile through Alice's service
    let bob_pk = bob.keys().public_key();
    let loaded_profile = alice_dialog.load_profile(&bob_pk)
        .await
        .expect("Failed to load Bob's profile");
    
    // Verify Alice can see Bob's profile information
    if let Some(profile) = loaded_profile {
        assert_eq!(profile.display_name, bob_profile.display_name);
        assert_eq!(profile.name, bob_profile.name);
        assert_eq!(profile.about, bob_profile.about);
        assert_eq!(profile.picture, bob_profile.picture);
    } else {
        panic!("Bob's profile should have been found");
    }
    
    // Verify Bob is in Alice's contacts
    let contacts = alice_dialog.get_contacts()
        .await
        .expect("Failed to get Alice's contacts");
    
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].pubkey, bob.keys().public_key());
}

#[tokio::test]
async fn test_multiple_contacts() {
    let scenario = TestScenario::new(&["alice", "bob", "charlie"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    let bob = scenario.get_user("bob").unwrap();
    let charlie = scenario.get_user("charlie").unwrap();
    
    let alice_dialog = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create Alice's DialogLib");
    
    alice_dialog.connect()
        .await
        .expect("Failed to connect Alice to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Add multiple contacts
    alice_dialog.add_contact(bob.pubkey_hex())
        .await
        .expect("Failed to add Bob as contact");
    
    alice_dialog.add_contact(charlie.pubkey_hex())
        .await
        .expect("Failed to add Charlie as contact");
    
    // Get contacts and verify both were added
    let contacts = alice_dialog.get_contacts()
        .await
        .expect("Failed to get contacts");
    
    assert_eq!(contacts.len(), 2);
    
    // Verify both Bob and Charlie are in the contacts
    let contact_pubkeys: Vec<_> = contacts.iter().map(|c| c.pubkey).collect();
    assert!(contact_pubkeys.contains(&bob.keys().public_key()));
    assert!(contact_pubkeys.contains(&charlie.keys().public_key()));
}

#[tokio::test]
async fn test_contact_online_status() {
    let scenario = TestScenario::new(&["alice", "bob"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    let bob = scenario.get_user("bob").unwrap();
    
    let alice_dialog = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create Alice's DialogLib");
    
    alice_dialog.connect()
        .await
        .expect("Failed to connect Alice to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Add Bob as a contact
    alice_dialog.add_contact(bob.pubkey_hex())
        .await
        .expect("Failed to add Bob as contact");
    
    let contacts = alice_dialog.get_contacts()
        .await
        .expect("Failed to get contacts");
    
    assert_eq!(contacts.len(), 1);
    
    // Check online status (initially should be false since Bob isn't connected)
    assert_eq!(contacts[0].online, false);
    
    // Connect Bob to the relay
    let bob_dialog = DialogLib::new_with_keys_and_relay(
        bob.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create Bob's DialogLib");
    
    bob_dialog.connect()
        .await
        .expect("Failed to connect Bob to relay");
    
    sleep(Duration::from_millis(200)).await;
    
    // Check contacts again - online status might be updated
    // Note: This depends on the implementation of online status tracking
    let updated_contacts = alice_dialog.get_contacts()
        .await
        .expect("Failed to get updated contacts");
    
    // The online status behavior depends on the implementation
    // For now, we just verify the contact is still there
    assert_eq!(updated_contacts.len(), 1);
}