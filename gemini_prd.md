## Encrypted Note-to-Self Chat App: Product Requirements Document

This document outlines the product requirements, architecture, and API specifications for a secure, note-to-self chat application. The application is inspired by the Whitenoise project, leveraging Nostr for communication and MLS for end-to-end encryption. The initial focus is on a simplified use case: sending encrypted notes to oneself across multiple devices.

This PRD provides a detailed technical plan for extending the existing `dialog_client` to support this functionality and enable its integration with an iOS application using Swift and UniFFI.

### 1\. Vision & Strategy

The vision is to create a secure, private, and user-friendly application for personal note-taking and reminders, with the assurance of end-to-end encryption and the flexibility of the Nostr network. By starting with a "note-to-self" feature, we can build a robust and secure foundation before potentially expanding to more complex messaging features.

The strategy is to build a core Rust library, `dialog_client`, that encapsulates all the complex logic of Nostr communication, MLS encryption, and state management. This library will be exposed to a Swift-based iOS application via UniFFI, following a modern, reactive architectural pattern inspired by Elm/Iced.

### 2\. High-Level Architecture

The proposed architecture is based on the **Elm/Iced pattern**, which promotes a unidirectional data flow and a clear separation of concerns. This model is well-suited for a reactive UI framework like SwiftUI and integrates cleanly with a Rust core over an FFI boundary.

The core components of this architecture are:

  * **State (Model):** A single, comprehensive Rust struct that represents the entire state of the application. This includes the user's keys, connected relays, notes, and the MLS group state. This state will be managed within the Rust core.
  * **Message:** A Rust enum that defines all possible user actions and system events that can mutate the state. For example, `ConnectToRelay`, `PublishNote`, `NostrEventReceived`.
  * **Update:** A Rust function that takes the current state and a message and produces a new state. This function contains the core business logic of the application.
  * **View:** The SwiftUI-based user interface. The view is a direct reflection of the current state. It sends `Message`s to the Rust core in response to user interactions.

To facilitate communication between the Swift UI and the Rust core, we will implement a long-lived Tokio thread in Rust that manages the application state and interacts with the Nostr network and MLS library.

The communication flow will be as follows:

1.  The Swift UI sends a `Message` to the Rust core via a UniFFI-exposed function.
2.  This function pushes the `Message` into a `tokio::sync::mpsc` channel.
3.  The long-lived Tokio thread has a receiver for this channel. Upon receiving a `Message`, it calls the `update` function to modify the application `State`.
4.  After the `State` is updated, the Rust core uses a **UniFFI Callback Interface** to notify the Swift side that a new state is available.
5.  The Swift side, which will conform to this callback protocol, receives the updated state, updates its local `ObservableObject`, and the SwiftUI `View` automatically re-renders.

### 3\. Detailed `dialog_client` Enhancements

To support the note-to-self application, the `dialog_client` will be significantly enhanced. The following sections detail the required changes to its structure, types, and APIs.

#### 3.1. Project Structure (`dialog_client/src/`)

The proposed modular architecture from the `dialog_client/README.md` will be implemented to organize the growing complexity:

```
src/
├── lib.rs             # Main entry point, UniFFI definitions, and the core `DialogClient`
├── error.rs           # Centralized `DialogClientError` type
├── state.rs           # Definition of `AppState` and related data structures
├── message.rs         # The `Message` enum for all actions and events
├── update.rs          # The core `update` function logic
└── core/
    ├── mod.rs
    ├── nostr_handling.rs  # Logic for interacting with nostr-sdk
    └── mls_handling.rs    # Logic for interacting with openmls
```

#### 3.2. Core Rust Types

The following Rust types will form the foundation of the application's logic.

##### `dialog_client/src/state.rs`

```rust
use nostr_sdk::Event;
use openmls::prelude::MlsGroup;
use std::collections::HashMap;

// The single source of truth for the application's state.
#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub keys: Option<nostr_sdk::Keys>,
    pub relays: HashMap<String, RelayState>,
    pub notes: Vec<Event>,
    pub mls_group: Option<MlsGroup>,
    pub is_loading: bool,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct RelayState {
    pub is_connected: bool,
    // Add other relay-specific state as needed, e.g., auth status
}
```

##### `dialog_client/src/message.rs`

```rust
use nostr_sdk::{Event, PublicKey};

// All possible actions and events that can occur in the app.
#[derive(Debug)]
pub enum Message {
    // User-initiated actions
    Initialize,
    ConnectToRelay { url: String },
    PublishNote { content: String },

    // Internal events
    NostrEventReceived(Event),
    MlsMessageReceived(Vec<u8>),
    StateUpdated,
    ErrorOccurred(String),
}
```

##### `dialog_client/src/error.rs`

A comprehensive error enum will be used for robust error handling.

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DialogClientError {
    #[error("Nostr SDK error: {0}")]
    NostrSdk(#[from] nostr_sdk::client::Error),

    #[error("OpenMLS error: {0}")]
    OpenMls(String), // openmls errors are not std::Error, so we'll wrap them in a String

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("UniFFI error: {0}")]
    UniFFI(#[from] uniffi::UnexpectedUniFFICallbackError),
}

pub type Result<T> = std::result::Result<T, DialogClientError>;
```

#### 3.3. UniFFI Interface Definition (`dialog_client/src/dialog_client.udl`)

The UniFFI Definition Language (UDL) file will define the contract between Rust and Swift.

```udl
namespace dialog_client {
    // Callback interface for Rust to notify Swift of state changes
    interface StateUpdateCallback {
        on_update(state: AppState);
    };

    // The main application state object
    record AppState {
        // We will expose a serialized version of the state for simplicity across the FFI boundary
        // A more optimized approach could involve exposing individual fields.
        string json;
    };

    // The messages Swift can send to Rust
    enum Message {
        "Initialize",
        "ConnectToRelay" { url: string },
        "PublishNote" { content: string },
    };

    // The core client object that manages the application logic
    interface DialogClient {
        constructor(callback: StateUpdateCallback);
        void send_message(message: Message);
        string get_current_state_json();
    };
};
```

*Note: For simplicity, we are serializing `AppState` to JSON to pass it across the FFI boundary. A more optimized solution for performance-critical applications might involve defining the entire `AppState` struct in the UDL.*

#### 3.4. Core Client Logic (`dialog_client/src/lib.rs`)

The main `lib.rs` file will be rewritten to implement the Elm/Iced architecture.

```rust
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Re-export types for UniFFI
pub use crate::error::{DialogClientError, Result};
pub use crate::state::AppState;
pub use crate::message::Message;

uniffi::include_scaffolding!("dialog_client");

#[uniffi::export(callback_interface)]
pub trait StateUpdateCallback: Send + Sync {
    fn on_update(&self, state: String);
}

#[derive(Debug)]
pub struct DialogClient {
    state: Arc<Mutex<AppState>>,
    message_sender: mpsc::Sender<Message>,
}

#[uniffi::export]
impl DialogClient {
    #[uniffi::constructor]
    pub fn new(callback: Box<dyn StateUpdateCallback>) -> Arc<Self> {
        let (message_sender, mut message_receiver) = mpsc::channel(100);
        let initial_state = AppState::default();
        let state = Arc::new(Mutex::new(initial_state));

        let client = Arc::new(Self {
            state: state.clone(),
            message_sender,
        });

        // Spawn the long-lived Tokio task
        let state_clone = state.clone();
        tokio::spawn(async move {
            while let Some(message) = message_receiver.recv().await {
                let new_state = update::update(state_clone.lock().unwrap().clone(), message).await;
                *state_clone.lock().unwrap() = new_state.clone();

                // Notify Swift of the state update
                if let Ok(json_state) = serde_json::to_string(&new_state) {
                    callback.on_update(json_state);
                }
            }
        });

        client
    }

    pub fn send_message(&self, message: Message) {
        // It's safe to ignore the error here as the receiver lives as long as the client
        let _ = self.message_sender.try_send(message);
    }

    pub fn get_current_state_json(&self) -> String {
        serde_json::to_string(&*self.state.lock().unwrap()).unwrap_or_default()
    }
}
```

#### 3.5. The `update` Function (`dialog_client/src/update.rs`)

This function is the heart of the application's logic. It's where all state transitions happen.

```rust
use crate::{AppState, Message, core::{nostr_handling, mls_handling}};

pub async fn update(mut state: AppState, message: Message) -> AppState {
    match message {
        Message::Initialize => {
            // Generate keys, setup initial MLS state for the user (as a group of 1)
            let (new_state, commands) = mls_handling::initialize_self_group(state).await;
            state = new_state;
            // Execute any generated commands (e.g., initial nostr connection)
        }
        Message::ConnectToRelay { url } => {
            state.is_loading = true;
            match nostr_handling::connect(&url).await {
                Ok(_) => {
                    let mut relay_state = state.relays.entry(url).or_default();
                    relay_state.is_connected = true;
                },
                Err(e) => state.last_error = Some(e.to_string()),
            }
            state.is_loading = false;
        }
        Message::PublishNote { content } => {
            state.is_loading = true;
            // 1. Encrypt the note content using the current MLS group state
            // 2. Publish the encrypted content as a Nostr event
            // This will involve both mls_handling and nostr_handling modules
            state.is_loading = false;
        }
        Message::NostrEventReceived(event) => {
            // 1. Attempt to decrypt the event content using the MLS group state
            // 2. If successful, add the decrypted note to the `notes` vector
        }
        _ => { /* Handle other messages */ }
    }
    state
}
```

### 4\. Swift (iOS) Implementation

The iOS application will be built with SwiftUI. The `DialogClient` from the Rust core will be wrapped in a Swift class that acts as an `ObservableObject`.

#### 4.1. Swift `DialogClient` Wrapper

```swift
import Foundation
import SwiftUI
import dialog_client

class DialogClientStore: ObservableObject {
    @Published var state: AppState

    private var client: dialog_client.DialogClient?
    private let callback: StateUpdateCallback

    init() {
        self.state = AppState(json: "{}") // Initial empty state
        self.callback = StateUpdateCallback(
            onUpdate: { [weak self] stateJson in
                DispatchQueue.main.async {
                    self?.state = AppState(json: stateJson)
                }
            }
        )
        self.client = dialog_client.newDialogClient(callback: self.callback)
        // Load initial state
        self.state = AppState(json: self.client?.getCurrentStateJson() ?? "{}")
        // Trigger initialization
        self.sendMessage(message: .initialize)
    }

    func sendMessage(message: dialog_client.Message) {
        client?.sendMessage(message: message)
    }
}

// Implement the callback protocol
class StateUpdateCallback: dialog_client.StateUpdateCallback {
    private let onUpdateClosure: (String) -> Void

    init(onUpdate: @escaping (String) -> Void) {
        self.onUpdateClosure = onUpdate
    }

    func onUpdate(state: String) {
        self.onUpdateClosure(state)
    }
}

// Make the Rust AppState usable in Swift
extension dialog_client.AppState {
    // Add computed properties to access the JSON fields
    // This requires AppState to be Codable on the Rust side
    // For now, we'll just expose the raw JSON
}
```

#### 4.2. SwiftUI View Example

```swift
import SwiftUI
import dialog_client

struct ContentView: View {
    @StateObject private var store = DialogClientStore()

    var body: some View {
        NavigationView {
            VStack {
                List(store.state.notes, id: \.id) { note in
                    Text(note.content)
                }

                HStack {
                    TextField("New note", text: $newNoteContent)
                    Button("Send") {
                        store.sendMessage(message: .publishNote(content: newNoteContent))
                        newNoteContent = ""
                    }
                }
                .padding()
            }
            .navigationTitle("My Notes")
            .onAppear {
                store.sendMessage(message: .connectToRelay(url: "wss://relay.damus.io"))
            }
        }
    }

    @State private var newNoteContent: String = ""
}
```

### 5\. Next Steps & Roadmap

This PRD lays the foundation for the encrypted note-to-self app. The immediate next steps are:

1.  **Implement the new `dialog_client` architecture:** Refactor the existing code to match the proposed structure, including the `AppState`, `Message`, and `update` logic.
2.  **Integrate `openmls`:** Implement the `mls_handling.rs` module for creating and managing the user's "self-group" and for encrypting/decrypting notes.
3.  **Implement `nostr-sdk` logic in `nostr_handling.rs`:** Wire up the `nostr-sdk` to be driven by the `update` function, including connecting to relays, publishing events, and subscribing to filters.
4.  **Develop the UniFFI layer:** Create the `dialog_client.udl` file and ensure the Rust types are correctly exposed to Swift.
5.  **Build the iOS UI:** Develop the SwiftUI views and connect them to the `DialogClientStore`.

This detailed plan provides a clear path forward for building a secure and robust note-to-self application, with a solid architectural foundation for future expansion.