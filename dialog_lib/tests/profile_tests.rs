mod test_helpers;

use dialog_lib::{DialogLib, Profile};
use test_helpers::{EphemeralRelay, TestUser, TestScenario};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_profile_publish_and_load() {
    let scenario = TestScenario::new(&["alice"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    
    // Create DialogLib instance with ephemeral relay and Alice's keys
    let dialog_lib = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(), 
        scenario.relay_url()
    )
    .await
    .expect("Failed to create DialogLib");
    
    // Connect to the ephemeral relay
    dialog_lib.connect()
        .await
        .expect("Failed to connect to relay");
    
    // Give connection time to establish
    sleep(Duration::from_millis(100)).await;
    
    // Create and publish a profile
    let test_profile = Profile {
        display_name: Some("Alice Wonderland".to_string()),
        name: Some("alice".to_string()),
        about: Some("Testing profile publishing".to_string()),
        picture: Some("https://example.com/alice.jpg".to_string()),
        banner: None,
        website: None,
        lud16: None,
    };
    
    // Publish the profile
    dialog_lib.publish_profile(&test_profile)
        .await
        .expect("Failed to publish profile");
    
    // Give some time for the event to propagate
    sleep(Duration::from_millis(200)).await;
    
    // Create a second DialogLib instance to load the profile
    let bob = TestUser::new("bob", 2);
    let dialog_lib2 = DialogLib::new_with_keys_and_relay(
        bob.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create second DialogLib");
    
    dialog_lib2.connect()
        .await
        .expect("Failed to connect to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Load Alice's profile from the relay
    let alice_pk = alice.keys().public_key();
    let loaded_profile = dialog_lib2.load_profile(&alice_pk)
        .await
        .expect("Failed to load profile");
    
    // Verify the profile matches what we published
    if let Some(profile) = loaded_profile {
        assert_eq!(profile.display_name, test_profile.display_name);
        assert_eq!(profile.name, test_profile.name);
        assert_eq!(profile.about, test_profile.about);
        assert_eq!(profile.picture, test_profile.picture);
    } else {
        panic!("Profile should have been found");
    }
}

#[tokio::test]
async fn test_profile_updates() {
    let scenario = TestScenario::new(&["alice"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    
    let dialog_lib = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create DialogLib");
    
    dialog_lib.connect()
        .await
        .expect("Failed to connect to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Publish initial profile
    let initial_profile = Profile {
        display_name: Some("Alice".to_string()),
        name: None,
        about: Some("Initial profile".to_string()),
        picture: None,
        banner: None,
        website: None,
        lud16: None,
    };
    
    dialog_lib.publish_profile(&initial_profile)
        .await
        .expect("Failed to publish initial profile");
    
    sleep(Duration::from_millis(200)).await;
    
    // Update the profile
    let updated_profile = Profile {
        display_name: Some("Alice Wonderland".to_string()),
        name: Some("alice_wonderland".to_string()),
        about: Some("Updated profile".to_string()),
        picture: Some("https://example.com/alice.jpg".to_string()),
        banner: Some("https://example.com/banner.jpg".to_string()),
        website: Some("https://alice.example.com".to_string()),
        lud16: Some("alice@wallet.com".to_string()),
    };
    
    dialog_lib.publish_profile(&updated_profile)
        .await
        .expect("Failed to publish updated profile");
    
    sleep(Duration::from_millis(200)).await;
    
    // Load the profile and verify it has the updated information
    let alice_pk = alice.keys().public_key();
    let loaded_profile = dialog_lib.load_profile(&alice_pk)
        .await
        .expect("Failed to load updated profile");
    
    if let Some(profile) = loaded_profile {
        assert_eq!(profile.display_name, updated_profile.display_name);
        assert_eq!(profile.name, updated_profile.name);
        assert_eq!(profile.about, updated_profile.about);
        assert_eq!(profile.picture, updated_profile.picture);
        assert_eq!(profile.banner, updated_profile.banner);
        assert_eq!(profile.website, updated_profile.website);
        assert_eq!(profile.lud16, updated_profile.lud16);
    } else {
        panic!("Updated profile should have been found");
    }
}

#[tokio::test]
async fn test_profile_missing_user() {
    let scenario = TestScenario::new(&["alice"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    
    let dialog_lib = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create DialogLib");
    
    dialog_lib.connect()
        .await
        .expect("Failed to connect to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Try to load a profile for a user that doesn't exist
    let nonexistent_user = TestUser::new("nonexistent", 999);
    let nonexistent_pk = nonexistent_user.keys().public_key();
    let result = dialog_lib.load_profile(&nonexistent_pk).await;
    
    // Should return Ok(None) for missing profiles
    match result {
        Ok(profile_opt) => {
            assert!(profile_opt.is_none(), "Profile should not exist for nonexistent user");
        }
        Err(e) => {
            panic!("Loading missing profile should return Ok(None), not error: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_multiple_relays_profile_isolation() {
    // Test with two separate relay instances to ensure profiles are isolated
    let relay1 = EphemeralRelay::start()
        .await
        .expect("Failed to start first relay");
    
    let relay2 = EphemeralRelay::start()
        .await
        .expect("Failed to start second relay");
    
    let alice = TestUser::new("alice", 1);
    
    // Publish profile to first relay
    let dialog_lib1 = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        relay1.url()
    )
    .await
    .expect("Failed to create DialogLib for relay1");
    
    dialog_lib1.connect()
        .await
        .expect("Failed to connect to relay1");
    
    sleep(Duration::from_millis(100)).await;
    
    let test_profile = Profile {
        display_name: Some("Alice on Relay 1".to_string()),
        name: None,
        about: Some("Published to first relay".to_string()),
        picture: None,
        banner: None,
        website: None,
        lud16: None,
    };
    
    dialog_lib1.publish_profile(&test_profile)
        .await
        .expect("Failed to publish to relay1");
    
    sleep(Duration::from_millis(200)).await;
    
    // Try to load from second relay (should not find it)
    let dialog_lib2 = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        relay2.url()
    )
    .await
    .expect("Failed to create DialogLib for relay2");
    
    dialog_lib2.connect()
        .await
        .expect("Failed to connect to relay2");
    
    sleep(Duration::from_millis(100)).await;
    
    let alice_pk = alice.keys().public_key();
    let result = dialog_lib2.load_profile(&alice_pk).await;
    
    // Should not find the profile on the second relay
    match result {
        Ok(profile_opt) => {
            assert!(profile_opt.is_none(), "Profile should not exist on different relay");
        }
        Err(_) => {
            // Error is also acceptable when profile doesn't exist
        }
    }
}

#[tokio::test]
async fn test_simple_profile_helper() {
    let scenario = TestScenario::new(&["alice"])
        .await
        .expect("Failed to create test scenario");
    
    let alice = scenario.get_user("alice").unwrap();
    
    let dialog_lib = DialogLib::new_with_keys_and_relay(
        alice.keys().clone(),
        scenario.relay_url()
    )
    .await
    .expect("Failed to create DialogLib");
    
    dialog_lib.connect()
        .await
        .expect("Failed to connect to relay");
    
    sleep(Duration::from_millis(100)).await;
    
    // Use the simple profile helper
    dialog_lib.publish_simple_profile("Alice Simple")
        .await
        .expect("Failed to publish simple profile");
    
    sleep(Duration::from_millis(200)).await;
    
    // Load the profile and verify it has the display name
    let alice_pk = alice.keys().public_key();
    let loaded_profile = dialog_lib.load_profile(&alice_pk)
        .await
        .expect("Failed to load profile");
    
    if let Some(profile) = loaded_profile {
        assert_eq!(profile.display_name, Some("Alice Simple".to_string()));
        // Other fields should be None for simple profile
        assert_eq!(profile.about, None);
        assert_eq!(profile.picture, None);
    } else {
        panic!("Simple profile should have been found");
    }
}