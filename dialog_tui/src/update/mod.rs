use crossterm::event::{KeyCode, KeyModifiers};
use crate::model::{AppState, Msg, Cmd, ActivePane, ConversationId, DialogMode, Contact, ContactId, PendingInvite, PowerToolsMode, LogEntry};

pub fn update(state: &AppState, msg: Msg) -> (AppState, Cmd) {
    match msg {
        Msg::KeyPress(key) => handle_key_press(state, key),
        Msg::SendMessage => handle_send_message(state),
        Msg::SelectConversation(id) => handle_select_conversation(state, id),
        Msg::SwitchPane(pane) => {
            let mut new_state = state.clone();
            new_state.active_pane = pane;
            (new_state, Cmd::None)
        }
        Msg::MessageReceived(message) => handle_message_received(state, message),
        Msg::WebSocketConnected => {
            let mut new_state = state.clone();
            new_state.connection_status = crate::model::state::ConnectionStatus::Connected;
            (new_state, Cmd::None)
        }
        Msg::WebSocketDisconnected => {
            let mut new_state = state.clone();
            new_state.connection_status = crate::model::state::ConnectionStatus::Disconnected;
            (new_state, Cmd::None)
        }
        Msg::TerminalResized(w, h) => {
            let mut new_state = state.clone();
            new_state.terminal_size = (w, h);
            (new_state, Cmd::None)
        }
        Msg::ToggleHelp => {
            let mut new_state = state.clone();
            new_state.show_help = !new_state.show_help;
            (new_state, Cmd::None)
        }
        Msg::ShowAddContactDialog => handle_show_add_contact_dialog(state),
        Msg::ShowCreateConversationDialog => handle_show_create_conversation_dialog(state),
        Msg::ShowPublishKeypackageDialog => handle_show_publish_keypackage_dialog(state),
        Msg::ShowAcceptInviteDialog => handle_show_accept_invite_dialog(state),
        Msg::AddContact(pubkey, petname) => handle_add_contact(state, pubkey, petname),
        Msg::ContactAdded(contact) => handle_contact_added(state, contact),
        Msg::CreateConversationWithContact(contact_id) => handle_create_conversation(state, contact_id),
        Msg::PublishKeypackage => handle_publish_keypackage(state),
        Msg::AcceptInvite(invite_index) => handle_accept_invite(state, invite_index),
        Msg::InviteReceived(invite) => handle_invite_received(state, invite),
        Msg::SelectInvite(invite_index) => handle_select_invite(state, invite_index),
        Msg::FetchNewMessages => (state.clone(), Cmd::FetchNewMessages),
        Msg::ExpireToasts => handle_expire_toasts(state),
        Msg::TogglePowerTools => handle_toggle_power_tools(state),
        Msg::PowerToolsSelect(index) => handle_power_tools_select(state, index),
        Msg::PowerToolsAction => handle_power_tools_action(state),
        Msg::PowerToolsModeSwitch(mode) => handle_power_tools_mode_switch(state, mode),
        Msg::LogMessage(entry) => handle_log_message(state, entry),
        Msg::DialogInput(c) => handle_dialog_input(state, c),
        Msg::DialogBackspace => handle_dialog_backspace(state),
        Msg::DialogSubmit => handle_dialog_submit(state),
        Msg::DialogCancel => handle_dialog_cancel(state),
        Msg::DialogNextField => handle_dialog_next_field(state),
        Msg::SelectContact(id) => handle_select_contact(state, id),
        Msg::Quit => (state.clone(), Cmd::Exit),
        _ => (state.clone(), Cmd::None),
    }
}

fn handle_key_press(state: &AppState, key: crossterm::event::KeyEvent) -> (AppState, Cmd) {
    // Handle dialog mode first
    if state.dialog_state.mode != DialogMode::Normal {
        return handle_dialog_key_press(state, key);
    }
    
    // Global shortcuts
    match key.code {
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            (state.clone(), Cmd::Exit)
        }
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            handle_show_publish_keypackage_dialog(state)
        }
        KeyCode::F(1) => {
            let mut new_state = state.clone();
            new_state.show_help = !new_state.show_help;
            (new_state, Cmd::None)
        }
        KeyCode::F(2) => handle_toggle_power_tools(state),
        KeyCode::Tab => handle_tab(state),
        _ => handle_pane_specific_keys(state, key),
    }
}

fn handle_dialog_key_press(state: &AppState, key: crossterm::event::KeyEvent) -> (AppState, Cmd) {
    match key.code {
        KeyCode::Esc => {
            tracing::info!("Dialog cancelled via Esc key");
            handle_dialog_cancel(state)
        },
        KeyCode::Enter => {
            tracing::info!("Dialog submitted via Enter key - mode: {:?}", state.dialog_state.mode);
            handle_dialog_submit(state)
        },
        KeyCode::Tab => handle_dialog_next_field(state),
        KeyCode::Backspace => handle_dialog_backspace(state),
        KeyCode::Char('j') if state.dialog_state.mode == DialogMode::CreateConversation => {
            handle_dialog_nav_down(state)
        }
        KeyCode::Char('k') if state.dialog_state.mode == DialogMode::CreateConversation => {
            handle_dialog_nav_up(state)
        }
        KeyCode::Char('j') if state.dialog_state.mode == DialogMode::AcceptInvite => {
            handle_dialog_nav_down(state)
        }
        KeyCode::Char('k') if state.dialog_state.mode == DialogMode::AcceptInvite => {
            handle_dialog_nav_up(state)
        }
        KeyCode::Char(c) => handle_dialog_input(state, c),
        _ => (state.clone(), Cmd::None),
    }
}

fn handle_pane_specific_keys(state: &AppState, key: crossterm::event::KeyEvent) -> (AppState, Cmd) {
    // Handle power tools keys first
    if state.active_pane == ActivePane::PowerTools {
        return handle_power_tools_keys(state, key);
    }
    
    match key.code {
        KeyCode::Char('h') if state.active_pane != ActivePane::Input => {
            (state.clone(), Cmd::None) // Left - could switch to conversations
        }
        KeyCode::Char('l') if state.active_pane != ActivePane::Input => {
            (state.clone(), Cmd::None) // Right - could switch to chat
        }
        KeyCode::Char('j') if state.active_pane == ActivePane::Contacts => {
            handle_contacts_nav_down(state)
        }
        KeyCode::Char('k') if state.active_pane == ActivePane::Contacts => {
            handle_contacts_nav_up(state)
        }
        KeyCode::Char('j') if state.active_pane == ActivePane::Conversations => {
            handle_conversations_nav_down(state)
        }
        KeyCode::Char('k') if state.active_pane == ActivePane::Conversations => {
            handle_conversations_nav_up(state)
        }
        KeyCode::Char('j') if state.active_pane == ActivePane::Chat => {
            let mut new_state = state.clone();
            new_state.scroll_offset = new_state.scroll_offset.saturating_sub(1);
            (new_state, Cmd::None)
        }
        KeyCode::Char('k') if state.active_pane == ActivePane::Chat => {
            let mut new_state = state.clone();
            new_state.scroll_offset = new_state.scroll_offset.saturating_add(1);
            (new_state, Cmd::None)
        }
        KeyCode::Enter if state.active_pane == ActivePane::Contacts => {
            handle_show_add_contact_dialog(state)
        }
        KeyCode::Enter if state.active_pane == ActivePane::Conversations => {
            handle_conversations_enter(state)
        }
        KeyCode::Enter if state.active_pane == ActivePane::Input => {
            handle_send_message(state)
        }
        KeyCode::Char(c) if state.active_pane == ActivePane::Input => {
            let mut new_state = state.clone();
            new_state.input_buffer.push(c);
            (new_state, Cmd::None)
        }
        KeyCode::Backspace if state.active_pane == ActivePane::Input => {
            let mut new_state = state.clone();
            new_state.input_buffer.pop();
            (new_state, Cmd::None)
        }
        _ => (state.clone(), Cmd::None),
    }
}

fn handle_tab(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.active_pane = match state.active_pane {
        ActivePane::Contacts => ActivePane::Conversations,
        ActivePane::Conversations => ActivePane::Chat,
        ActivePane::Chat => ActivePane::Input,
        ActivePane::Input => ActivePane::Contacts,
        ActivePane::PowerTools => ActivePane::Contacts, // Exit power tools on tab
    };
    (new_state, Cmd::None)
}

fn handle_contacts_nav_down(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    
    if state.contacts.is_empty() {
        return (state.clone(), Cmd::None);
    }
    
    let contact_keys: Vec<_> = state.contacts.keys().cloned().collect();
    
    // Find current position
    let current_pos = if let Some(selected_id) = &state.selected_contact {
        contact_keys.iter().position(|id| id == selected_id).unwrap_or(0)
    } else {
        0
    };
    
    // Calculate next position
    let next_pos = (current_pos + 1) % contact_keys.len();
    
    // Update selection
    if let Some(contact_id) = contact_keys.get(next_pos) {
        new_state.selected_contact = Some(contact_id.clone());
    }
    
    (new_state, Cmd::None)
}

fn handle_contacts_nav_up(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    
    if state.contacts.is_empty() {
        return (state.clone(), Cmd::None);
    }
    
    let contact_keys: Vec<_> = state.contacts.keys().cloned().collect();
    
    // Find current position
    let current_pos = if let Some(selected_id) = &state.selected_contact {
        contact_keys.iter().position(|id| id == selected_id).unwrap_or(0)
    } else {
        contact_keys.len() - 1
    };
    
    // Calculate previous position
    let prev_pos = if current_pos == 0 {
        contact_keys.len() - 1
    } else {
        current_pos - 1
    };
    
    // Update selection
    if let Some(contact_id) = contact_keys.get(prev_pos) {
        new_state.selected_contact = Some(contact_id.clone());
    }
    
    (new_state, Cmd::None)
}

fn handle_conversations_nav_down(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    
    let invite_count = state.pending_invites.len();
    let conv_count = state.conversations.len();
    let _has_separator = invite_count > 0;
    let total_items = invite_count + conv_count;
    
    if total_items == 0 {
        return (state.clone(), Cmd::None);
    }
    
    // Determine current position
    let current_pos = if let Some(invite_idx) = state.selected_invite {
        invite_idx
    } else if let Some(conv_id) = &state.selected_conversation {
        // Find position of this conversation
        let conv_keys: Vec<_> = state.conversations.keys().collect();
        if let Some(conv_idx) = conv_keys.iter().position(|&id| id == conv_id) {
            invite_count + conv_idx
        } else {
            0
        }
    } else {
        // Nothing selected, start at beginning
        0
    };
    
    // Calculate next position
    let next_pos = (current_pos + 1) % total_items;
    
    // Update selection based on new position
    if next_pos < invite_count {
        // Selecting an invite
        new_state.selected_invite = Some(next_pos);
        new_state.selected_conversation = None;
    } else {
        // Selecting a conversation
        let conv_idx = next_pos - invite_count;
        let conv_keys: Vec<_> = new_state.conversations.keys().cloned().collect();
        if let Some(conv_id) = conv_keys.get(conv_idx) {
            new_state.selected_conversation = Some(conv_id.clone());
            new_state.selected_invite = None;
        }
    }
    
    (new_state, Cmd::None)
}

fn handle_conversations_nav_up(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    
    let invite_count = state.pending_invites.len();
    let conv_count = state.conversations.len();
    let _has_separator = invite_count > 0;
    let total_items = invite_count + conv_count;
    
    if total_items == 0 {
        return (state.clone(), Cmd::None);
    }
    
    // Determine current position
    let current_pos = if let Some(invite_idx) = state.selected_invite {
        invite_idx
    } else if let Some(conv_id) = &state.selected_conversation {
        // Find position of this conversation
        let conv_keys: Vec<_> = state.conversations.keys().collect();
        if let Some(conv_idx) = conv_keys.iter().position(|&id| id == conv_id) {
            invite_count + conv_idx
        } else {
            0
        }
    } else {
        // Nothing selected, start at end
        total_items - 1
    };
    
    // Calculate previous position
    let prev_pos = if current_pos == 0 {
        total_items - 1
    } else {
        current_pos - 1
    };
    
    // Update selection based on new position
    if prev_pos < invite_count {
        // Selecting an invite
        new_state.selected_invite = Some(prev_pos);
        new_state.selected_conversation = None;
    } else {
        // Selecting a conversation
        let conv_idx = prev_pos - invite_count;
        let conv_keys: Vec<_> = new_state.conversations.keys().cloned().collect();
        if let Some(conv_id) = conv_keys.get(conv_idx) {
            new_state.selected_conversation = Some(conv_id.clone());
            new_state.selected_invite = None;
        }
    }
    
    (new_state, Cmd::None)
}

fn handle_send_message(state: &AppState) -> (AppState, Cmd) {
    if state.input_buffer.trim().is_empty() || state.selected_conversation.is_none() {
        return (state.clone(), Cmd::None);
    }

    let mut new_state = state.clone();
    let message = new_state.input_buffer.clone();
    new_state.input_buffer.clear();

    let cmd = Cmd::SendMessage(message, state.selected_conversation.clone().unwrap());
    (new_state, cmd)
}

fn handle_select_conversation(state: &AppState, id: ConversationId) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.selected_conversation = Some(id.clone());
    new_state.scroll_offset = 0;
    new_state.active_pane = ActivePane::Chat;

    // Clear unread count
    if let Some(conversation) = new_state.conversations.get_mut(&id) {
        conversation.unread_count = 0;
    }

    // Load history if needed
    let cmd = if new_state.messages.get(&id).is_none() {
        Cmd::LoadConversationHistory(id)
    } else {
        Cmd::None
    };

    (new_state, cmd)
}

fn handle_message_received(state: &AppState, message: crate::model::ChatMessage) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    
    // Add message to appropriate conversation
    new_state.messages
        .entry(message.conversation_id.clone())
        .or_insert_with(Vec::new)
        .push(message.clone());

    // Update conversation last message time and unread count
    if let Some(conversation) = new_state.conversations.get_mut(&message.conversation_id) {
        conversation.last_message_time = Some(message.timestamp);
        if Some(&message.conversation_id) != new_state.selected_conversation.as_ref() {
            conversation.unread_count += 1;
        }
    }

    let save_cmd = Cmd::SaveMessage(message);
    (new_state, save_cmd)
}

// Dialog handlers
fn handle_show_add_contact_dialog(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.dialog_state.mode = DialogMode::AddContact;
    new_state.dialog_state.input_buffer.clear();
    new_state.dialog_state.field_index = 0;
    new_state.dialog_state.stored_fields.clear();
    (new_state, Cmd::None)
}

fn handle_show_create_conversation_dialog(state: &AppState) -> (AppState, Cmd) {
    tracing::info!("Showing create conversation dialog");
    if state.contacts.is_empty() {
        tracing::warn!("Cannot show create conversation dialog: no contacts available");
        return (state.clone(), Cmd::None); // No contacts to create conversation with
    }
    
    tracing::info!("Opening create conversation dialog with {} contacts", state.contacts.len());
    let mut new_state = state.clone();
    new_state.dialog_state.mode = DialogMode::CreateConversation;
    new_state.dialog_state.input_buffer.clear();
    new_state.dialog_state.field_index = 0;
    new_state.dialog_state.stored_fields.clear();
    (new_state, Cmd::None)
}

fn handle_show_publish_keypackage_dialog(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.dialog_state.mode = DialogMode::PublishKeypackage;
    new_state.dialog_state.input_buffer.clear();
    new_state.dialog_state.field_index = 0;
    new_state.dialog_state.stored_fields.clear();
    (new_state, Cmd::None)
}

fn handle_dialog_input(state: &AppState, c: char) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.dialog_state.input_buffer.push(c);
    (new_state, Cmd::None)
}

fn handle_dialog_backspace(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.dialog_state.input_buffer.pop();
    (new_state, Cmd::None)
}

fn handle_dialog_cancel(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.dialog_state.mode = DialogMode::Normal;
    new_state.dialog_state.input_buffer.clear();
    new_state.dialog_state.field_index = 0;
    new_state.dialog_state.stored_fields.clear();
    (new_state, Cmd::None)
}

fn handle_dialog_next_field(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    match state.dialog_state.mode {
        DialogMode::AddContact => {
            // Store current field and move to next
            new_state.dialog_state.stored_fields.push(state.dialog_state.input_buffer.clone());
            new_state.dialog_state.field_index = (state.dialog_state.field_index + 1) % 2;
            new_state.dialog_state.input_buffer.clear();
        }
        DialogMode::CreateConversation => {
            // Navigate through contacts
            let contact_count = state.contacts.len();
            if contact_count > 0 {
                new_state.dialog_state.field_index = (state.dialog_state.field_index + 1) % contact_count;
            }
        }
        _ => {}
    }
    (new_state, Cmd::None)
}

fn handle_dialog_nav_down(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    match state.dialog_state.mode {
        DialogMode::CreateConversation => {
            let contact_count = state.contacts.len();
            if contact_count > 0 {
                new_state.dialog_state.field_index = (state.dialog_state.field_index + 1) % contact_count;
            }
        }
        DialogMode::AcceptInvite => {
            let invite_count = state.pending_invites.len();
            if invite_count > 0 {
                new_state.dialog_state.field_index = (state.dialog_state.field_index + 1) % invite_count;
            }
        }
        _ => {}
    }
    (new_state, Cmd::None)
}

fn handle_dialog_nav_up(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    match state.dialog_state.mode {
        DialogMode::CreateConversation => {
            let contact_count = state.contacts.len();
            if contact_count > 0 {
                new_state.dialog_state.field_index = 
                    if state.dialog_state.field_index == 0 { 
                        contact_count - 1 
                    } else { 
                        state.dialog_state.field_index - 1 
                    };
            }
        }
        DialogMode::AcceptInvite => {
            let invite_count = state.pending_invites.len();
            if invite_count > 0 {
                new_state.dialog_state.field_index = 
                    if state.dialog_state.field_index == 0 { 
                        invite_count - 1 
                    } else { 
                        state.dialog_state.field_index - 1 
                    };
            }
        }
        _ => {}
    }
    (new_state, Cmd::None)
}

fn handle_dialog_submit(state: &AppState) -> (AppState, Cmd) {
    match state.dialog_state.mode {
        DialogMode::AddContact => {
            // Check if we have both fields
            if state.dialog_state.field_index == 0 {
                // Still on first field, move to next
                return handle_dialog_next_field(state);
            } else if state.dialog_state.field_index == 1 && !state.dialog_state.stored_fields.is_empty() {
                // On second field with first field stored, submit
                let pubkey = state.dialog_state.stored_fields[0].clone();
                let petname = state.dialog_state.input_buffer.clone();
                
                if !pubkey.trim().is_empty() && !petname.trim().is_empty() {
                    let mut new_state = state.clone();
                    new_state.dialog_state.mode = DialogMode::Normal;
                    new_state.dialog_state.input_buffer.clear();
                    new_state.dialog_state.field_index = 0;
                    new_state.dialog_state.stored_fields.clear();
                    return (new_state, Cmd::Batch(vec![
                        Cmd::SaveContact(Contact {
                            id: uuid::Uuid::new_v4().to_string(),
                            pubkey: match nostr_sdk::PublicKey::from_hex(&pubkey) {
                                Ok(pk) => pk,
                                Err(_) => return (state.clone(), Cmd::None), // Invalid pubkey
                            },
                            petname,
                            created_at: chrono::Utc::now(),
                        }),
                        Cmd::LoadContacts,
                    ]));
                }
            }
            handle_dialog_cancel(state)
        }
        DialogMode::CreateConversation => {
            // Get selected contact
            let contacts: Vec<&Contact> = state.contacts.values().collect();
            tracing::info!("Dialog submit for CreateConversation: {} contacts available, field_index: {}", 
                contacts.len(), state.dialog_state.field_index);
            
            if let Some(contact) = contacts.get(state.dialog_state.field_index) {
                let contact_id = contact.id.clone();
                tracing::info!("Creating MLS group with contact: {} (id: {})", contact.petname, contact_id);
                
                let mut new_state = state.clone();
                new_state.dialog_state.mode = DialogMode::Normal;
                new_state.dialog_state.input_buffer.clear();
                new_state.dialog_state.field_index = 0;
                new_state.dialog_state.stored_fields.clear();
                return (new_state, Cmd::CreateMlsGroup(contact_id));
            } else {
                tracing::warn!("No contact found at field_index {} when submitting CreateConversation dialog", state.dialog_state.field_index);
            }
            handle_dialog_cancel(state)
        }
        DialogMode::PublishKeypackage => {
            let mut new_state = state.clone();
            new_state.dialog_state.mode = DialogMode::Normal;
            new_state.dialog_state.input_buffer.clear();
            new_state.dialog_state.field_index = 0;
            new_state.dialog_state.stored_fields.clear();
            (new_state, Cmd::PublishKeypackageToRelay)
        }
        DialogMode::AcceptInvite => {
            // Accept the selected invite
            let invite_index = state.dialog_state.field_index;
            let mut new_state = state.clone();
            new_state.dialog_state.mode = DialogMode::Normal;
            new_state.dialog_state.input_buffer.clear();
            new_state.dialog_state.field_index = 0;
            new_state.dialog_state.stored_fields.clear();
            (new_state, Cmd::AcceptPendingInvite(invite_index))
        }
        DialogMode::Normal => (state.clone(), Cmd::None),
    }
}

fn handle_add_contact(state: &AppState, pubkey: String, petname: String) -> (AppState, Cmd) {
    // Parse pubkey and create contact
    match nostr_sdk::PublicKey::from_hex(&pubkey) {
        Ok(pk) => {
            let contact_id = uuid::Uuid::new_v4().to_string();
            let contact = Contact {
                id: contact_id,
                pubkey: pk,
                petname,
                created_at: chrono::Utc::now(),
            };
            (state.clone(), Cmd::SaveContact(contact))
        }
        Err(_) => {
            // TODO: Show error message
            (state.clone(), Cmd::None)
        }
    }
}

fn handle_contact_added(state: &AppState, contact: Contact) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.contacts.insert(contact.id.clone(), contact);
    (new_state, Cmd::None)
}

fn handle_select_contact(state: &AppState, contact_id: ContactId) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.selected_contact = Some(contact_id);
    (new_state, Cmd::None)
}

fn handle_create_conversation(state: &AppState, contact_id: ContactId) -> (AppState, Cmd) {
    // This will trigger MLS group creation
    (state.clone(), Cmd::CreateMlsGroup(contact_id))
}

fn handle_publish_keypackage(state: &AppState) -> (AppState, Cmd) {
    (state.clone(), Cmd::PublishKeypackageToRelay)
}

// Invite-related handlers
fn handle_conversations_enter(state: &AppState) -> (AppState, Cmd) {
    tracing::info!("Conversations enter pressed");
    
    // Check if we have a selected invite
    if let Some(invite_index) = state.selected_invite {
        tracing::info!("Selected invite {} - showing accept dialog", invite_index);
        return handle_show_accept_invite_dialog_for_invite(state, invite_index);
    }
    
    // Check if we have a selected conversation
    if let Some(conv_id) = &state.selected_conversation {
        tracing::info!("Selected conversation {}, switching to chat", conv_id);
        // Switch to the chat pane for the selected conversation
        let mut new_state = state.clone();
        new_state.active_pane = ActivePane::Chat;
        return (new_state, Cmd::None);
    }
    
    // Otherwise, show create conversation dialog
    tracing::info!("No invite or conversation selected - showing create conversation dialog");
    handle_show_create_conversation_dialog(state)
}

fn handle_show_accept_invite_dialog(state: &AppState) -> (AppState, Cmd) {
    if state.pending_invites.is_empty() {
        return (state.clone(), Cmd::None);
    }
    
    let mut new_state = state.clone();
    new_state.dialog_state.mode = DialogMode::AcceptInvite;
    new_state.dialog_state.input_buffer.clear();
    new_state.dialog_state.field_index = 0;
    new_state.dialog_state.stored_fields.clear();
    (new_state, Cmd::None)
}

fn handle_show_accept_invite_dialog_for_invite(state: &AppState, invite_index: usize) -> (AppState, Cmd) {
    if invite_index >= state.pending_invites.len() {
        return (state.clone(), Cmd::None);
    }
    
    let mut new_state = state.clone();
    new_state.dialog_state.mode = DialogMode::AcceptInvite;
    new_state.dialog_state.input_buffer.clear();
    new_state.dialog_state.field_index = invite_index;
    new_state.dialog_state.stored_fields.clear();
    (new_state, Cmd::None)
}

fn handle_accept_invite(state: &AppState, invite_index: usize) -> (AppState, Cmd) {
    (state.clone(), Cmd::AcceptPendingInvite(invite_index))
}

fn handle_invite_received(state: &AppState, invite: PendingInvite) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.pending_invites.push(invite);
    (new_state, Cmd::None)
}

fn handle_select_invite(state: &AppState, invite_index: usize) -> (AppState, Cmd) {
    if invite_index >= state.pending_invites.len() {
        return (state.clone(), Cmd::None);
    }
    
    let mut new_state = state.clone();
    new_state.selected_invite = Some(invite_index);
    (new_state, Cmd::None)
}

// Power Tools handlers
fn handle_toggle_power_tools(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    if state.active_pane == ActivePane::PowerTools {
        new_state.active_pane = ActivePane::Contacts;
    } else {
        new_state.active_pane = ActivePane::PowerTools;
        new_state.power_tools_mode = PowerToolsMode::Menu;
        new_state.power_tools_selection = 0;
    }
    (new_state, Cmd::None)
}

fn handle_power_tools_keys(state: &AppState, key: crossterm::event::KeyEvent) -> (AppState, Cmd) {
    match (state.power_tools_mode.clone(), key.code) {
        (PowerToolsMode::Menu, KeyCode::Char('j')) => {
            let mut new_state = state.clone();
            new_state.power_tools_selection = (state.power_tools_selection + 1) % 8; // 8 menu items
            (new_state, Cmd::None)
        }
        (PowerToolsMode::Menu, KeyCode::Char('k')) => {
            let mut new_state = state.clone();
            new_state.power_tools_selection = if state.power_tools_selection == 0 { 7 } else { state.power_tools_selection - 1 };
            (new_state, Cmd::None)
        }
        (PowerToolsMode::Menu, KeyCode::Enter) => handle_power_tools_action(state),
        (PowerToolsMode::Menu, KeyCode::Char('l')) => {
            let mut new_state = state.clone();
            new_state.power_tools_mode = PowerToolsMode::DebugLog;
            (new_state, Cmd::None)
        }
        (PowerToolsMode::DebugLog, KeyCode::Char('j')) => {
            // TODO: Implement log scrolling
            (state.clone(), Cmd::None)
        }
        (PowerToolsMode::DebugLog, KeyCode::Char('k')) => {
            // TODO: Implement log scrolling
            (state.clone(), Cmd::None)
        }
        (PowerToolsMode::DebugLog, KeyCode::Char('c')) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let mut new_state = state.clone();
            new_state.debug_logs.clear();
            (new_state, Cmd::None)
        }
        (_, KeyCode::Esc) => {
            if state.power_tools_mode == PowerToolsMode::DebugLog {
                let mut new_state = state.clone();
                new_state.power_tools_mode = PowerToolsMode::Menu;
                (new_state, Cmd::None)
            } else {
                let mut new_state = state.clone();
                new_state.active_pane = ActivePane::Contacts;
                (new_state, Cmd::None)
            }
        }
        _ => (state.clone(), Cmd::None),
    }
}

fn handle_power_tools_select(state: &AppState, index: usize) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.power_tools_selection = index;
    (new_state, Cmd::None)
}

fn handle_power_tools_action(state: &AppState) -> (AppState, Cmd) {
    match state.power_tools_selection {
        0 => (state.clone(), Cmd::ResetAllState),
        1 => (state.clone(), Cmd::DeleteAllContacts),
        2 => (state.clone(), Cmd::DeleteAllConversations),
        3 => (state.clone(), Cmd::RescanRelays),
        4 => (state.clone(), Cmd::RepublishKeypackage),
        5 => {
            let mut new_state = state.clone();
            new_state.power_tools_mode = PowerToolsMode::DebugLog;
            (new_state, Cmd::None)
        }
        6 => (state.clone(), Cmd::FetchNewMessages),
        7 => (state.clone(), Cmd::FetchPendingInvites),
        _ => (state.clone(), Cmd::None),
    }
}

fn handle_power_tools_mode_switch(state: &AppState, mode: PowerToolsMode) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.power_tools_mode = mode;
    (new_state, Cmd::None)
}

fn handle_log_message(state: &AppState, entry: LogEntry) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    new_state.debug_logs.push(entry);
    
    // Keep only the last 1000 log entries to prevent memory bloat
    if new_state.debug_logs.len() > 1000 {
        new_state.debug_logs.drain(0..new_state.debug_logs.len() - 1000);
    }
    
    (new_state, Cmd::None)
}

fn handle_expire_toasts(state: &AppState) -> (AppState, Cmd) {
    let mut new_state = state.clone();
    let now = chrono::Utc::now();
    
    // Remove expired toasts
    new_state.toast_notifications.retain(|toast| {
        let elapsed = now.signed_duration_since(toast.timestamp).num_seconds() as u64;
        elapsed < toast.duration_secs
    });
    
    (new_state, Cmd::None)
}