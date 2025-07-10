# Key Package Management Research

## Executive Summary

This document analyzes key package management strategies for the Dialog project, comparing whitenoise's implementation with MLS RFC recommendations. The key finding is that **Dialog should implement proactive key package management** to ensure users don't miss group invitations due to stale key packages.

## The Problem

Currently, if Alice doesn't publish a new key package and Bob tries to create a group with an old key package:
- The group creation may fail if Alice no longer has the private keys
- Alice won't receive the invitation even if creation succeeds
- There's no automatic recovery mechanism

### Root Cause with Memory Storage

The fundamental issue is that Dialog uses `NostrMlsMemoryStorage`, which means:
- Private keys for key packages are lost on restart
- Published key packages remain on the relay indefinitely
- Alice can't decrypt welcome messages encrypted to her old packages
- There's no way to detect or clean up these "orphaned" packages

### Key Types Clarification

There are two types of keys involved:

1. **Nostr Identity Keys (from nsec)**
   - Persistent and deterministic (derived from nsec)
   - Used for signing Nostr events
   - Used for decrypting gift-wrapped messages
   - NOT lost on restart

2. **MLS HPKE Keys (in key packages)**
   - Randomly generated for each key package
   - Used for MLS encryption protocol
   - Used to decrypt welcome messages when joining groups
   - **LOST on restart with memory storage**
   
The confusion arises because while Alice keeps her Nostr identity (nsec), she loses the HPKE private keys that are essential for joining groups. These HPKE keys are randomly generated each time a key package is created and cannot be reconstructed from the nsec.

## Whitenoise's Approach

### Key Findings

1. **No Automatic Key Package Refresh**
   - Key packages are only published during initial account setup
   - No republishing on startup or periodic rotation
   - Key packages remain valid indefinitely

2. **Persistent State Management**
   ```rust
   pub struct OnboardingState {
       pub inbox_relays: bool,
       pub key_package_relays: bool,
       pub key_package_published: bool,  // Tracks if key package was ever published
   }
   ```

3. **Manual Lifecycle Management**
   - `publish_key_package_for_account()` - Explicit publishing
   - `delete_key_package_from_relays_for_account()` - Manual deletion
   - No automatic expiration or rotation

4. **Autoconnect Behavior**
   - Loads accounts from database on startup
   - Sets up subscriptions automatically
   - Does NOT republish key packages

## MLS RFC Recommendations

### Key Package Lifetime

From RFC 9420 and RFC 9750:

1. **Single Use Principle**
   - Key packages are intended for single use
   - Exception: "last resort" key packages can be reused

2. **Lifetime Fields**
   - `not_before`: Earliest valid time (seconds since Unix epoch)
   - `not_after`: Latest valid time
   - Applications MUST define maximum acceptable lifetime

3. **Last Resort Key Packages**
   - Special designated packages for multiple use
   - Should be rotated "as soon as possible after being used"
   - Minimize usage through proper provisioning

### Best Practices from RFCs

1. **Proactive Rotation**
   - Before credentials expire
   - After key package usage
   - When signature keys change

2. **Forward Secrecy**
   - Delete private keys after use
   - Rotate compromised keys immediately
   - Minimize key material lifetime

## Recommended Implementation for Dialog

### 1. Startup Behavior

```rust
async fn on_startup(&mut self) -> Result<()> {
    // Always connect to relay on startup
    self.connect_to_relay().await?;
    
    // Check key package age
    if self.should_refresh_key_packages()? {
        self.refresh_key_packages().await?;
    }
    
    Ok(())
}
```

### 2. Key Package Refresh Strategy

```rust
fn should_refresh_key_packages(&self) -> Result<bool> {
    // Option 1: Time-based (e.g., older than 24 hours)
    let last_published = self.get_last_key_package_publish_time()?;
    let age = SystemTime::now().duration_since(last_published)?;
    
    // Option 2: Usage-based (if we track usage)
    let unused_count = self.count_unused_key_packages()?;
    
    Ok(age > Duration::from_secs(86400) || unused_count < 5)
}
```

### 3. Key Package Management

```rust
impl DialogLib {
    /// Publish fresh key packages, optionally revoking old ones
    pub async fn refresh_key_packages(&self) -> Result<()> {
        // 1. Generate new key packages (e.g., 10 standard + 1 last resort)
        let packages = self.generate_key_packages(10)?;
        
        // 2. Publish to relay
        self.publish_key_packages_to_relay(packages).await?;
        
        // 3. Optionally revoke old packages
        if self.config.revoke_old_packages {
            self.revoke_old_key_packages().await?;
        }
        
        // 4. Update local state
        self.update_key_package_metadata()?;
        
        Ok(())
    }
}
```

### 4. Persistent State Requirements

```rust
struct KeyPackageMetadata {
    published_at: SystemTime,
    package_count: u32,
    last_resort_id: Option<String>,
    expiration: Option<SystemTime>,
}
```

## Implementation Options

### Option 1: Aggressive Refresh (Recommended for Testing)
- Publish new key packages on every startup
- Short lifetime (e.g., 24 hours)
- Delete old packages
- **Pros**: Maximum freshness, easier debugging
- **Cons**: More relay traffic, potential race conditions

### Option 2: Conservative Refresh
- Publish only when needed (low package count or expired)
- Longer lifetime (e.g., 7 days)
- Keep some old packages for reliability
- **Pros**: Less traffic, more reliable
- **Cons**: Potential for stale packages

### Option 3: Hybrid Approach
- Publish new packages on startup if > 24 hours old
- Keep last 2 generations of packages
- Track usage and refresh when low
- **Pros**: Balanced approach
- **Cons**: More complex implementation

## Key Package Consistency Solution

### The Insight

When Alice starts up, she could:
1. Query the relay for all her published key packages
2. Check which ones she has private keys for
3. Delete/replace orphaned packages she can't decrypt

### Implementation Approach

```rust
async fn sync_key_packages_on_startup(&mut self) -> Result<()> {
    // 1. Fetch all our key packages from relay
    let published_packages = self.fetch_own_key_packages().await?;
    
    // 2. Check which ones we have private keys for
    let mut orphaned = Vec::new();
    let mut valid = Vec::new();
    
    for package in published_packages {
        if self.has_private_key_for(&package)? {
            valid.push(package);
        } else {
            orphaned.push(package);
        }
    }
    
    // 3. Clean up orphaned packages
    if !orphaned.is_empty() {
        println!("Found {} orphaned key packages, cleaning up...", orphaned.len());
        self.delete_key_packages(orphaned).await?;
    }
    
    // 4. Ensure we have fresh packages available
    if valid.len() < MIN_KEY_PACKAGES {
        println!("Publishing fresh key packages...");
        self.publish_key_packages().await?;
    }
    
    Ok(())
}
```

### Benefits

1. **Consistency**: Ensures relay state matches local state
2. **Reliability**: Old invites won't fail due to missing private keys
3. **Cleanliness**: Removes unusable key packages from relay
4. **Fresh Start**: Each session starts with known-good packages

### Challenges

1. **Can't Recover Private Keys**: Once lost (due to memory storage), private keys can't be reconstructed from public packages
2. **Race Conditions**: Multiple clients might try to clean up simultaneously
3. **In-Flight Invites**: Deleting packages might invalidate invites that are in transit

## Immediate Steps for Dialog

1. **Add Autoconnect on Startup** ✅ (Implemented)
   ```rust
   // In dialog_tui main.rs after creating App
   if let Err(e) = app.dialog_lib.toggle_connection().await {
       app.add_message(&format!("Failed to autoconnect: {}", e));
   } else {
       app.add_message("Connected to relay");
   }
   ```

2. **Add Key Package Refresh Command** ✅ (Implemented)
   ```rust
   "/refresh-keys" => {
       self.add_message("Refreshing key packages...");
       match self.dialog_lib.refresh_key_packages().await {
           Ok(count) => {
               self.add_message(&format!("✅ Published {} fresh key packages", count));
           }
           Err(e) => {
               self.add_message(&format!("❌ Failed to refresh: {}", e));
           }
       }
   }
   ```

3. **Add Key Package Sync on Startup** (Proposed)
   - Query relay for our own key packages
   - Check consistency with local private keys
   - Clean up orphaned packages
   - Publish fresh packages if needed

4. **Track Key Package State** (Future)
   - Store key package metadata persistently
   - Track which packages are in use
   - Implement proper lifecycle management

## Security Considerations

1. **Race Conditions**
   - Multiple clients refreshing simultaneously
   - Need coordination or acceptance of duplicates

2. **Key Compromise**
   - Old packages with compromised keys remain valid
   - Need revocation mechanism

3. **Denial of Service**
   - Malicious clients could flood with packages
   - Rate limiting on relay side recommended

## Solution Paths

### Option 1: Clean Slate on Startup (Current Workaround)
- Delete all old key packages on startup
- Publish fresh packages immediately
- Simple but may invalidate in-flight invites

### Option 2: Persistent Storage (Whitenoise Approach)
- Switch to SQLite or file-based storage
- Preserve HPKE private keys across restarts
- More complex but maintains continuity

### Option 3: Deterministic Key Derivation (Non-standard)
- Derive HPKE keys from nsec + counter
- Could reconstruct keys on restart
- Breaks MLS security assumptions about forward secrecy

### Option 4: Key Package Sync and Cleanup (Proposed)
- Query relay for existing packages on startup
- Delete packages we can't decrypt
- Publish fresh packages to replace them
- Maintains standard MLS security while handling memory storage limitations

## Conclusion

Dialog should implement at minimum:
1. Autoconnect on startup ✅
2. Manual key package refresh command ✅
3. Key package sync/cleanup on startup (proposed)

For production, implement full lifecycle management with:
- Switch to persistent storage
- Automatic refresh based on age/usage
- Proper last resort package handling
- Revocation of old packages

The key insight is that MLS HPKE keys are randomly generated and cannot be reconstructed from the Nostr identity. This makes persistent storage or aggressive cleanup essential for reliable group messaging.