# Whitenoise Architecture Analysis for UniFFI/Swift Note App Development

## Whitenoise implements secure messaging through layered encryption

Whitenoise combines the Message Layer Security (MLS) protocol with Nostr's decentralized infrastructure to create a privacy-focused messaging platform. **The project uses OpenMLS for group encryption while leveraging Nostr as both an authentication and delivery service**, implementing custom NIPs (443-446) specifically designed for MLS operations. Their architecture demonstrates how to build secure, decentralized applications using Rust's strong type system and async capabilities.

The core innovation lies in their double encryption approach: MLS manages group state and key rotation for forward secrecy, while NIP-44 encrypts the actual message content using export secrets derived from the MLS group state. This design ensures metadata protection through ephemeral keys for all group events, preventing correlation attacks while maintaining the security guarantees of both protocols.

## MLS integration with Nostr creates a unique security model

### Custom Protocol Implementation

Whitenoise bridges MLS and Nostr through their `nostr-openmls` library (now integrated into rust-nostr as `nostr-mls`), which simplifies the complex MLS API for Nostr-specific use cases. The implementation uses four custom event kinds:

```rust
// Key package distribution for asynchronous member addition
Kind::Custom(443) // KeyPackage events
Kind::Custom(444) // Welcome messages (gift-wrapped)
Kind::Custom(445) // Group messages (NIP-44 encrypted)
Kind::Custom(446) // Group metadata events
```

The security model achieves remarkable privacy properties. **All group events are published from ephemeral keys**, making it impossible for observers to determine communication participants or correlate messages. The system implements two ciphersuites: the required MLS standard (`MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519`) and a custom Nostr-optimized variant (`MLS_256_DHKEMK256_CHACHA20POLY1305_SHA256_K256`) that uses secp256k1 for compatibility with the Bitcoin/Nostr ecosystem.

### Cryptographic Architecture

The encryption flow demonstrates sophisticated key management:

```rust
// Export secret derivation for message encryption
let (export_secret_hex, epoch) = nostr_mls
    .export_secret_as_hex_secret_key_and_epoch(group_id)?;
let export_nostr_keys = Keys::parse(&export_secret_hex)?;

// NIP-44 encryption with derived keys
let encrypted_content = nip44::encrypt(
    export_nostr_keys.secret_key(),
    &export_nostr_keys.public_key(),
    &serialized_message,
    Version::V2,
)?;
```

This approach provides forward secrecy through epoch-based key rotation while maintaining compatibility with Nostr's existing encryption standards.

## Rust architecture patterns emphasize type safety and async operations

### Tokio-Based Async Architecture

Whitenoise fully embraces Rust's async ecosystem, using Tokio for all I/O operations. Their architecture demonstrates best practices for handling concurrent operations in a messaging context:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let nostr_mls = NostrMls::new(PathBuf::from("./nostr-mls"), None);
    
    // All operations are async-first
    let group_create_result = nostr_mls.create_group(/*...*/).await?;
    let join_result = nostr_mls.join_group_from_welcome(/*...*/).await?;
}
```

The codebase shows consistent patterns for error handling using `Result<T, E>` types throughout, ensuring that cryptographic operations cannot fail silently. **Their type system design prevents misuse of cryptographic primitives** by encoding security requirements directly into the type signatures.

### Module Organization

The project follows a clean library architecture that separates concerns effectively:
- Core MLS functionality wrapped in high-level APIs
- Nostr protocol handling through the `nostr-sdk` crate
- Custom crypto provider implementations for platform compatibility
- Frontend-agnostic design allowing multiple UI implementations

## Adapting Whitenoise patterns for UniFFI note-to-self apps

### Simplified Architecture for Single-User Scenarios

For a note-to-self app, you can adapt Whitenoise's patterns while simplifying the group management complexity. Using the Elm/Iced architecture pattern with UniFFI provides clean separation between Rust logic and Swift UI:

```rust
#[derive(uniffi::Record)]
pub struct NoteState {
    pub notes: Vec<EncryptedNote>,
    pub sync_status: SyncStatus,
}

#[derive(uniffi::Enum)]
pub enum NoteMessage {
    CreateNote { content: String },
    UpdateNote { id: String, content: String },
    DeleteNote { id: String },
    SyncWithRelays,
}

#[uniffi::export]
impl NoteCore {
    pub fn update(&mut self, message: NoteMessage) -> Vec<NoteEffect> {
        match message {
            NoteMessage::CreateNote { content } => {
                let encrypted = self.encrypt_note(content);
                self.state.notes.push(encrypted);
                vec![
                    NoteEffect::SaveLocal,
                    NoteEffect::PublishToNostr
                ]
            }
            // Handle other messages
        }
    }
}
```

### Implementing Long-Lived Event Loops with UniFFI

Based on Whitenoise's event loop patterns and UniFFI best practices, implement background synchronization using Tokio with proper Swift integration:

```rust
#[uniffi::export]
pub struct SyncManager {
    runtime: Arc<Runtime>,
    nostr_client: Arc<Client>,
}

impl SyncManager {
    #[uniffi::constructor]
    pub fn new(relays: Vec<String>) -> Arc<Self> {
        let runtime = Runtime::new().unwrap();
        let client = runtime.block_on(async {
            let client = Client::new(&Keys::generate());
            for relay in relays {
                client.add_relay(&relay).await.ok();
            }
            client
        });
        
        Arc::new(Self {
            runtime: Arc::new(runtime),
            nostr_client: Arc::new(client),
        })
    }
    
    #[uniffi::method]
    pub fn start_sync(&self, callback: Box<dyn SyncCallback>) {
        let client = self.nostr_client.clone();
        self.runtime.spawn(async move {
            loop {
                // Subscribe to note events
                let filter = Filter::new()
                    .kind(Kind::Custom(30001)) // Custom kind for notes
                    .author(client.keys().public_key());
                    
                match client.get_events_from(vec![filter], None).await {
                    Ok(events) => callback.on_notes_received(events),
                    Err(e) => callback.on_error(e.to_string()),
                }
                
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
    }
}
```

### Encrypted Storage and Key Management

Adapt Whitenoise's encryption approach for simpler single-user scenarios:

```rust
#[uniffi::export]
pub struct EncryptedNoteStore {
    master_key: Arc<SecretKey>,
    db_path: PathBuf,
}

impl EncryptedNoteStore {
    #[uniffi::method]
    pub async fn save_note(&self, content: String) -> Result<String, StoreError> {
        // Derive note-specific key using HKDF
        let note_id = generate_id();
        let note_key = derive_key(&self.master_key, &note_id);
        
        // Encrypt content with NIP-44
        let encrypted = nip44::encrypt(
            &note_key,
            &self.master_key.public_key(),
            content.as_bytes(),
            Version::V2,
        )?;
        
        // Store locally and prepare Nostr event
        let event = EventBuilder::new(
            Kind::Custom(30001), // Replaceable parameterized event
            encrypted,
            &[Tag::Identifier(note_id.clone())]
        );
        
        Ok(note_id)
    }
}
```

## Swift integration using Elm/Iced patterns

### Message-Driven Architecture in Swift

Implement a clean message-passing interface that mirrors the Rust core:

```swift
class NoteViewModel: ObservableObject {
    @Published var notes: [Note] = []
    @Published var syncStatus: SyncStatus = .idle
    
    private let core: NoteCore
    private let syncManager: SyncManager
    
    init() {
        self.core = NoteCore()
        self.syncManager = SyncManager(relays: [
            "wss://relay.damus.io",
            "wss://nos.lol",
            "wss://relay.nostr.band"
        ])
        
        // Start background sync with callback
        syncManager.startSync(callback: SyncHandler(viewModel: self))
    }
    
    func send(_ message: NoteMessage) {
        let effects = core.update(message: message)
        
        Task {
            for effect in effects {
                await handleEffect(effect)
            }
        }
    }
    
    @MainActor
    private func handleEffect(_ effect: NoteEffect) async {
        switch effect {
        case .updateUI(let state):
            self.notes = state.notes
            self.syncStatus = state.syncStatus
            
        case .publishToNostr:
            // Trigger async publish
            await syncManager.publishPendingNotes()
            
        case .saveLocal:
            // Handle local persistence
            break
        }
    }
}

// Callback handler for background sync
class SyncHandler: SyncCallbackProtocol {
    weak var viewModel: NoteViewModel?
    
    func onNotesReceived(notes: [EncryptedNote]) {
        Task { @MainActor in
            viewModel?.send(.mergeRemoteNotes(notes: notes))
        }
    }
}
```

### Memory Management and Threading

Following UniFFI best practices for long-lived operations, ensure proper memory management:

```swift
extension NoteViewModel {
    func startBackgroundSync() {
        // Use weak references in callbacks to prevent retain cycles
        let handler = SyncHandler(viewModel: self)
        
        // Configure background task for iOS
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: "com.app.sync",
            using: nil
        ) { task in
            self.syncManager.performBackgroundSync { [weak self] in
                task.setTaskCompleted(success: true)
            }
        }
    }
}
```

## Conclusion

Whitenoise's architecture provides valuable patterns for building secure, decentralized applications with Rust and Swift. **Their approach to combining MLS with Nostr demonstrates how to layer security protocols effectively** while maintaining usability. For a note-to-self app, the key architectural insights include using Tokio for async operations, implementing the Elm/Iced pattern for clean state management, and leveraging UniFFI's callback interfaces for bidirectional communication.

The simplified single-user architecture can retain Whitenoise's security benefits—including end-to-end encryption and decentralized storage—while eliminating the complexity of group management. By focusing on NIP-44 encryption with derived keys and using Nostr's replaceable events for note storage, you can build a privacy-preserving note app that syncs across devices without central servers. The combination of Rust's type safety, UniFFI's Swift integration, and message-driven architecture creates a robust foundation for secure personal information management.