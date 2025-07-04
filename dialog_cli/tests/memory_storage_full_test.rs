use anyhow::Result;
use nostr_mls::prelude::*;
use nostr_mls_memory_storage::NostrMlsMemoryStorage;
use nostr_sdk::{EventBuilder, Keys, Kind, RelayUrl, Timestamp, UnsignedEvent, EventId};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

#[tokio::test]
async fn test_alice_bob_bidirectional_messaging_memory_storage() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    info!("=== Starting Alice and Bob bidirectional messaging test with memory storage ===");

    // Setup Alice and Bob with memory storage
    let alice_keys = Keys::generate();
    let alice_mls = NostrMls::new(NostrMlsMemoryStorage::default());
    let bob_keys = Keys::generate();
    let bob_mls = NostrMls::new(NostrMlsMemoryStorage::default());
    let relay_url = RelayUrl::parse("ws://localhost:8080")?;

    info!("Alice pubkey: {}", alice_keys.public_key().to_hex());
    info!("Bob pubkey: {}", bob_keys.public_key().to_hex());

    // Create Bob's key package
    let (bob_kp_encoded, bob_kp_tags) =
        bob_mls.create_key_package_for_event(&bob_keys.public_key(), [relay_url.clone()])?;
    let bob_kp_event = EventBuilder::new(Kind::MlsKeyPackage, bob_kp_encoded)
        .tags(bob_kp_tags)
        .sign_with_keys(&bob_keys)?;

    info!("Created Bob's key package");

    // Alice creates group with Bob
    let config = NostrGroupConfigData::new(
        "bidirectional-test-group".to_string(),
        "Test group for bidirectional messaging".to_string(),
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

    info!("Alice created group with ID: {}", hex::encode(group_id.as_slice()));
    info!("Alice group is at epoch: {}", create_res.group.epoch);

    // Bob accepts the welcome and joins the group
    bob_mls.process_welcome(&EventId::all_zeros(), &welcome_rumor)?;
    let pending_welcome = bob_mls
        .get_pending_welcomes()?
        .into_iter()
        .find(|w| w.mls_group_id == group_id)
        .expect("Welcome not found for Bob");
    bob_mls.accept_welcome(&pending_welcome)?;
    info!("Bob joined group");

    // Verify both are at same epoch
    let alice_groups = alice_mls.get_groups()?;
    let bob_groups = bob_mls.get_groups()?;
    let alice_group = alice_groups.iter().find(|g| g.mls_group_id == group_id).unwrap();
    let bob_group = bob_groups.iter().find(|g| g.mls_group_id == group_id).unwrap();
    
    info!("Alice's group epoch: {}", alice_group.epoch);
    info!("Bob's group epoch: {}", bob_group.epoch);

    // Test 1: Alice sends message to Bob
    info!("=== Test 1: Alice -> Bob ===");
    let alice_message = UnsignedEvent::new(
        alice_keys.public_key(),
        Timestamp::now(),
        Kind::TextNote,
        vec![],
        "Hello Bob! This is Alice sending you a message using memory storage.".to_string(),
    );
    let alice_message_event = alice_mls.create_message(&group_id, alice_message)?;
    alice_mls.process_message(&alice_message_event)?;
    info!("Alice sent message");

    // Bob processes Alice's message with retry logic
    let mut success = false;
    for i in 0..5 {
        info!("Bob attempting to process Alice's message (attempt {})...", i + 1);
        if bob_mls.process_message(&alice_message_event).is_ok() {
            info!("Bob successfully processed Alice's message");
            success = true;
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    assert!(success, "Bob failed to process Alice's message");

    // Verify Bob received Alice's message
    let bob_messages = bob_mls.get_messages(&group_id)?;
    assert_eq!(bob_messages.len(), 1, "Bob should have one message from Alice");
    assert_eq!(
        bob_messages[0].content, 
        "Hello Bob! This is Alice sending you a message using memory storage.",
        "Alice's message content doesn't match"
    );
    assert_eq!(
        bob_messages[0].pubkey, 
        alice_keys.public_key(),
        "Message should be from Alice"
    );
    info!("✅ Bob successfully received Alice's message");

    // Test 2: Bob sends message to Alice
    info!("=== Test 2: Bob -> Alice ===");
    let bob_message = UnsignedEvent::new(
        bob_keys.public_key(),
        Timestamp::now(),
        Kind::TextNote,
        vec![],
        "Hi Alice! Bob here, replying to your message. Memory storage works great!".to_string(),
    );
    let bob_message_event = bob_mls.create_message(&group_id, bob_message)?;
    bob_mls.process_message(&bob_message_event)?;
    info!("Bob sent reply message");

    // Alice processes Bob's message with retry logic
    let mut success = false;
    for i in 0..5 {
        info!("Alice attempting to process Bob's message (attempt {})...", i + 1);
        if alice_mls.process_message(&bob_message_event).is_ok() {
            info!("Alice successfully processed Bob's message");
            success = true;
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    assert!(success, "Alice failed to process Bob's message");

    // Verify Alice received Bob's message
    let alice_messages = alice_mls.get_messages(&group_id)?;
    assert_eq!(alice_messages.len(), 2, "Alice should have two messages total");
    
    // Find Bob's message (should be the second one)
    let bob_msg = alice_messages.iter()
        .find(|msg| msg.pubkey == bob_keys.public_key())
        .expect("Should find Bob's message");
    
    assert_eq!(
        bob_msg.content, 
        "Hi Alice! Bob here, replying to your message. Memory storage works great!",
        "Bob's message content doesn't match"
    );
    info!("✅ Alice successfully received Bob's reply");

    // Test 3: Multiple message exchange
    info!("=== Test 3: Multiple message exchange ===");
    
    // Alice sends another message
    let alice_message2 = UnsignedEvent::new(
        alice_keys.public_key(),
        Timestamp::now(),
        Kind::TextNote,
        vec![],
        "Great! Let's test multiple messages. This is message #2 from Alice.".to_string(),
    );
    let alice_message2_event = alice_mls.create_message(&group_id, alice_message2)?;
    alice_mls.process_message(&alice_message2_event)?;
    
    // Bob processes and replies
    bob_mls.process_message(&alice_message2_event)?;
    
    let bob_message2 = UnsignedEvent::new(
        bob_keys.public_key(),
        Timestamp::now(),
        Kind::TextNote,
        vec![],
        "Perfect! Bob's message #2. Bidirectional communication confirmed!".to_string(),
    );
    let bob_message2_event = bob_mls.create_message(&group_id, bob_message2)?;
    bob_mls.process_message(&bob_message2_event)?;
    
    // Alice processes Bob's second message
    alice_mls.process_message(&bob_message2_event)?;

    // Final verification - both should have all 4 messages
    let final_alice_messages = alice_mls.get_messages(&group_id)?;
    let final_bob_messages = bob_mls.get_messages(&group_id)?;
    
    assert_eq!(final_alice_messages.len(), 4, "Alice should have 4 messages total");
    assert_eq!(final_bob_messages.len(), 4, "Bob should have 4 messages total");
    
    info!("✅ Multiple message exchange successful");
    info!("Alice has {} messages, Bob has {} messages", 
          final_alice_messages.len(), final_bob_messages.len());

    // Display final message summary
    info!("=== Final Message Summary ===");
    for (i, msg) in final_alice_messages.iter().enumerate() {
        let sender = if msg.pubkey == alice_keys.public_key() { "Alice" } else { "Bob" };
        info!("Message {}: {} -> {}", i + 1, sender, msg.content.chars().take(50).collect::<String>());
    }

    info!("🎉 SUCCESS: Alice and Bob can send messages to each other bidirectionally using memory storage!");

    Ok(())
}

#[tokio::test]  
async fn test_memory_storage_performance() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    
    info!("=== Testing memory storage performance ===");
    
    let start = std::time::Instant::now();
    
    // Setup
    let alice_keys = Keys::generate();
    let alice_mls = NostrMls::new(NostrMlsMemoryStorage::default());
    let bob_keys = Keys::generate(); 
    let bob_mls = NostrMls::new(NostrMlsMemoryStorage::default());
    let relay_url = RelayUrl::parse("ws://localhost:8080")?;
    
    // Quick group setup
    let (bob_kp_encoded, bob_kp_tags) =
        bob_mls.create_key_package_for_event(&bob_keys.public_key(), [relay_url.clone()])?;
    let bob_kp_event = EventBuilder::new(Kind::MlsKeyPackage, bob_kp_encoded)
        .tags(bob_kp_tags)
        .sign_with_keys(&bob_keys)?;
    
    let config = NostrGroupConfigData::new(
        "performance-test".to_string(),
        "".to_string(),
        None,
        None,
        vec![relay_url],
    );
    let admins = vec![alice_keys.public_key(), bob_keys.public_key()];
    let create_res = alice_mls.create_group(&alice_keys.public_key(), vec![bob_kp_event], admins, config)?;
    let group_id = create_res.group.mls_group_id.clone();
    
    bob_mls.process_welcome(&EventId::all_zeros(), &create_res.welcome_rumors[0])?;
    let pending_welcome = bob_mls.get_pending_welcomes()?.into_iter().next().unwrap();
    bob_mls.accept_welcome(&pending_welcome)?;
    
    let setup_time = start.elapsed();
    info!("Setup completed in {:?}", setup_time);
    
    // Send multiple messages quickly
    let message_start = std::time::Instant::now();
    let num_messages = 10;
    
    for i in 0..num_messages {
        let message = UnsignedEvent::new(
            alice_keys.public_key(),
            Timestamp::now(),
            Kind::TextNote,
            vec![],
            format!("Performance test message #{}", i + 1),
        );
        let message_event = alice_mls.create_message(&group_id, message)?;
        alice_mls.process_message(&message_event)?;
        bob_mls.process_message(&message_event)?;
    }
    
    let message_time = message_start.elapsed();
    info!("Sent and processed {} messages in {:?}", num_messages, message_time);
    info!("Average time per message: {:?}", message_time / num_messages);
    
    let total_time = start.elapsed();
    info!("Total test time: {:?}", total_time);
    
    // Verify all messages were received
    let messages = bob_mls.get_messages(&group_id)?;
    assert_eq!(messages.len(), num_messages as usize);
    
    info!("✅ Performance test completed successfully");
    
    Ok(())
} 