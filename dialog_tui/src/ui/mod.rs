use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::model::{AppState, ActivePane, DialogMode, PowerToolsMode};

pub fn render(state: &AppState, frame: &mut Frame) {
    // Handle power tools panel
    if state.active_pane == ActivePane::PowerTools {
        render_power_tools(state, frame);
        return;
    }

    if state.show_help {
        render_help(frame);
        return;
    }

    // Handle dialog overlays
    if state.dialog_state.mode != DialogMode::Normal {
        render_dialog(state, frame);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25), // Left sidebar (contacts + conversations)
            Constraint::Min(40),    // Chat area
        ])
        .split(frame.area());

    // Split left sidebar vertically for contacts and conversations
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Contacts
            Constraint::Percentage(50), // Conversations
        ])
        .split(chunks[0]);

    render_contacts(state, frame, sidebar_chunks[0]);
    render_conversations(state, frame, sidebar_chunks[1]);
    render_chat_area(state, frame, chunks[1]);
}

fn render_contacts(state: &AppState, frame: &mut Frame, area: Rect) {
    let contacts: Vec<ListItem> = state.contacts
        .values()
        .map(|contact| {
            let style = if Some(&contact.id) == state.selected_contact.as_ref() {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::styled(&contact.petname, style)
            ]))
        })
        .collect();

    let mut title = "Contacts".to_string();
    if state.active_pane == ActivePane::Contacts && !contacts.is_empty() {
        title.push_str(" (Enter: New Contact)");
    } else if state.active_pane == ActivePane::Contacts {
        title.push_str(" (Enter: Add Contact)");
    }

    let contacts_list = List::new(contacts)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title.as_str())
            .border_style(Style::default().fg(
                if state.active_pane == ActivePane::Contacts {
                    Color::Green
                } else {
                    Color::Gray
                }
            )));

    frame.render_widget(contacts_list, area);
}

fn render_conversations(state: &AppState, frame: &mut Frame, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();
    
    // Add pending invites at the top
    for (i, invite) in state.pending_invites.iter().enumerate() {
        let style = if Some(i) == state.selected_invite {
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Blue).add_modifier(Modifier::ITALIC)
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled("📨 ", style),
            Span::styled(format!("Invite from {}", invite.petname), style)
        ])));
    }
    
    // Add separator if there are invites
    if !state.pending_invites.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("────────────────────", Style::default().fg(Color::DarkGray))
        ])));
    }
    
    // Add regular conversations
    for conv in state.conversations.values() {
        let style = if Some(&conv.id) == state.selected_conversation.as_ref() {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if conv.unread_count > 0 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let unread_indicator = if conv.unread_count > 0 {
            format!(" ({})", conv.unread_count)
        } else {
            String::new()
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{}{}", conv.name, unread_indicator), style)
        ])));
    }

    let mut title = "Conversations".to_string();
    if !state.pending_invites.is_empty() {
        title.push_str(&format!(" ({} pending)", state.pending_invites.len()));
    }
    if state.active_pane == ActivePane::Conversations && !state.contacts.is_empty() {
        title.push_str(" (Enter: Action)");
    } else if state.active_pane == ActivePane::Conversations {
        title.push_str(" (Add contacts first)");
    }

    let conversations_list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title.as_str())
            .border_style(Style::default().fg(
                if state.active_pane == ActivePane::Conversations {
                    Color::Green
                } else {
                    Color::Gray
                }
            )));

    frame.render_widget(conversations_list, area);
}

fn render_chat_area(state: &AppState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),    // Messages
            Constraint::Length(3),  // Input
            Constraint::Length(1),  // Status bar
        ])
        .split(area);

    render_messages(state, frame, chunks[0]);
    render_input(state, frame, chunks[1]);
    render_status_bar(state, frame, chunks[2]);
}

fn render_messages(state: &AppState, frame: &mut Frame, area: Rect) {
    let messages = if let Some(conversation_id) = &state.selected_conversation {
        state.messages.get(conversation_id).map(|msgs| msgs.as_slice()).unwrap_or(&[])
    } else {
        &[]
    };

    let message_widgets: Vec<ListItem> = messages
        .iter()
        .rev() // Show newest at bottom
        .skip(state.scroll_offset as usize)
        .take(area.height as usize - 2) // Account for borders
        .map(|msg| {
            let style = if msg.is_own {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            let content = if msg.is_own {
                format!("You: {}", msg.content)
            } else {
                format!("{}: {}", msg.sender.to_string()[..8].to_string(), msg.content)
            };

            ListItem::new(Line::from(vec![
                Span::styled(content, style),
                Span::raw(" "),
                Span::styled(
                    format!("{}", msg.timestamp.format("%H:%M")),
                    Style::default().fg(Color::DarkGray)
                ),
            ]))
        })
        .collect();

    let messages_list = List::new(message_widgets)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(state.selected_conversation.as_ref().map(|s| s.as_str()).unwrap_or("No Conversation"))
            .border_style(Style::default().fg(
                if state.active_pane == ActivePane::Chat {
                    Color::Green
                } else {
                    Color::Gray
                }
            )));

    frame.render_widget(messages_list, area);
}

fn render_input(state: &AppState, frame: &mut Frame, area: Rect) {
    let input = Paragraph::new(state.input_buffer.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Message (Enter to send)")
            .border_style(Style::default().fg(
                if state.active_pane == ActivePane::Input {
                    Color::Green
                } else {
                    Color::Gray
                }
            )));

    frame.render_widget(input, area);

    // Show cursor when input is active
    if state.active_pane == ActivePane::Input {
        frame.set_cursor_position((
            area.x + state.input_buffer.len() as u16 + 1,
            area.y + 1,
        ));
    }
}

fn render_status_bar(state: &AppState, frame: &mut Frame, area: Rect) {
    let status = match state.connection_status {
        crate::model::state::ConnectionStatus::Connected => {
            Span::styled("● Connected", Style::default().fg(Color::Green))
        }
        crate::model::state::ConnectionStatus::Connecting => {
            Span::styled("◐ Connecting...", Style::default().fg(Color::Yellow))
        }
        crate::model::state::ConnectionStatus::Disconnected => {
            Span::styled("○ Disconnected", Style::default().fg(Color::Red))
        }
    };

    let help = Span::raw(" | Ctrl-Q: Quit | Tab: Switch Pane | F1: Help | F2: Power Tools | j/k: Navigate");
    
    // Add debug info about selected conversation
    let debug_info = if let Some(conv_id) = &state.selected_conversation {
        Span::styled(
            format!(" | Conv: {}", &conv_id[..8.min(conv_id.len())]),
            Style::default().fg(Color::DarkGray)
        )
    } else {
        Span::raw("")
    };

    let status_line = Line::from(vec![status, help, debug_info]);
    let status_bar = Paragraph::new(status_line)
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(status_bar, area);
}

fn render_dialog(state: &AppState, frame: &mut Frame) {
    let area = frame.area();
    
    // Create a centered dialog box
    let dialog_width = area.width.min(50);
    let dialog_height = area.height.min(10);
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;
    
    let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);
    
    // Clear the background
    frame.render_widget(
        Block::default()
            .style(Style::default().bg(Color::Black))
            .borders(Borders::NONE),
        area,
    );
    
    match state.dialog_state.mode {
        DialogMode::AddContact => render_add_contact_dialog(state, frame, dialog_area),
        DialogMode::CreateConversation => render_create_conversation_dialog(state, frame, dialog_area),
        DialogMode::PublishKeypackage => render_publish_keypackage_dialog(state, frame, dialog_area),
        DialogMode::AcceptInvite => render_accept_invite_dialog(state, frame, dialog_area),
        DialogMode::Normal => {} // Should not happen
    }
}

fn render_add_contact_dialog(state: &AppState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Pubkey input
            Constraint::Length(3), // Petname input
            Constraint::Length(3), // Instructions
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new("Add Contact")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);
    
    // Input fields based on field_index
    let fields = ["Public Key (hex)", "Petname"];
    let current_field = state.dialog_state.field_index;
    
    for (i, field_name) in fields.iter().enumerate() {
        let input_text = if i == current_field {
            &state.dialog_state.input_buffer
        } else if i < state.dialog_state.stored_fields.len() {
            &state.dialog_state.stored_fields[i]
        } else {
            ""
        };
        
        let style = if i == current_field {
            Style::default().fg(Color::Green)
        } else if i < state.dialog_state.stored_fields.len() {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let input = Paragraph::new(input_text)
            .style(style)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(*field_name)
                .border_style(style));
        
        frame.render_widget(input, chunks[i + 1]);
        
        // Show cursor on active field
        if i == current_field {
            frame.set_cursor_position((
                chunks[i + 1].x + input_text.len() as u16 + 1,
                chunks[i + 1].y + 1,
            ));
        }
    }
    
    // Instructions
    let instructions = Paragraph::new("Tab: Next field | Enter: Submit | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(instructions, chunks[3]);
}

fn render_create_conversation_dialog(state: &AppState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Contact list
            Constraint::Length(3), // Instructions
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new("Create Conversation")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);
    
    // Contact list
    let contacts: Vec<ListItem> = state.contacts
        .values()
        .enumerate()
        .map(|(i, contact)| {
            let style = if i == state.dialog_state.field_index {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(&contact.petname, style)
            ]))
        })
        .collect();
    
    let contacts_list = List::new(contacts)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Select Contact")
            .border_style(Style::default().fg(Color::Green)));
    
    frame.render_widget(contacts_list, chunks[1]);
    
    // Instructions
    let instructions = Paragraph::new("j/k: Navigate | Enter: Create | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(instructions, chunks[2]);
}

fn render_publish_keypackage_dialog(_state: &AppState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Message
            Constraint::Length(3), // Instructions
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new("Publish Keypackage")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);
    
    // Message
    let message = Paragraph::new("This will publish your keypackage to the relay so others can add you to groups.")
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(message, chunks[1]);
    
    // Instructions
    let instructions = Paragraph::new("Enter: Publish | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(instructions, chunks[2]);
}

fn render_accept_invite_dialog(state: &AppState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(5), // Invite details
            Constraint::Length(3), // Instructions
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new("Accept Invite")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);
    
    // Show invite details
    let invite_index = state.dialog_state.field_index;
    let invite_text = if let Some(invite) = state.pending_invites.get(invite_index) {
        format!(
            "From: {}\nGroup: {}\nReceived: {}",
            invite.petname,
            invite.group_name.as_deref().unwrap_or("Unknown Group"),
            invite.timestamp.format("%Y-%m-%d %H:%M")
        )
    } else {
        "No invite selected".to_string()
    };
    
    let message = Paragraph::new(invite_text)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Invite Details"))
        .wrap(Wrap { trim: true });
    frame.render_widget(message, chunks[1]);
    
    // Instructions
    let instructions = Paragraph::new("Enter: Accept Invite | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(instructions, chunks[2]);
}

fn render_help(frame: &mut Frame) {
    let help_text = vec![
        Line::from("Dialog TUI - Keyboard Shortcuts"),
        Line::from(""),
        Line::from("Global:"),
        Line::from("  Ctrl-Q: Quit"),
        Line::from("  Tab: Switch between panes"),
        Line::from("  F1: Toggle this help"),
        Line::from("  Ctrl-P: Publish keypackage"),
        Line::from(""),
        Line::from("Contacts pane:"),
        Line::from("  j/k: Navigate up/down"),
        Line::from("  Enter: Add contact"),
        Line::from(""),
        Line::from("Conversations pane:"),
        Line::from("  j/k: Navigate up/down"),
        Line::from("  Enter: Create conversation"),
        Line::from(""),
        Line::from("Chat pane:"),
        Line::from("  j/k: Scroll messages"),
        Line::from(""),
        Line::from("Input pane:"),
        Line::from("  Type message, Enter to send"),
        Line::from("  Backspace: Delete character"),
        Line::from(""),
        Line::from("Press F1 to close help"),
    ];

    let help_widget = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .border_style(Style::default().fg(Color::Yellow)))
        .wrap(Wrap { trim: true });

    let area = frame.area();
    frame.render_widget(help_widget, area);
}

fn render_power_tools(state: &AppState, frame: &mut Frame) {
    match state.power_tools_mode {
        PowerToolsMode::Menu => render_power_tools_menu(state, frame),
        PowerToolsMode::DebugLog => render_debug_log(state, frame),
    }
}

fn render_power_tools_menu(state: &AppState, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Menu
            Constraint::Percentage(50), // Instructions
        ])
        .split(frame.area());

    // Power tools menu items
    let menu_items = vec![
        "🔧 Reset All State",
        "👥 Delete All Contacts",
        "💬 Delete All Conversations", 
        "📡 Rescan Relays",
        "🔑 Republish Keypackage",
        "📋 View Debug Log",
        "🔄 Fetch Messages Now",
        "📨 Fetch Invites Now",
    ];

    let menu_list: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == state.power_tools_selection {
                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![Span::styled(*item, style)]))
        })
        .collect();

    let menu = List::new(menu_list)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("🛠️ Power Tools")
            .border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(menu, chunks[0]);

    // Instructions panel
    let instructions = vec![
        Line::from("Navigation:"),
        Line::from("  j/k: Move up/down"),
        Line::from("  Enter: Execute action"),
        Line::from("  l: View debug log"),
        Line::from("  Esc: Return to main"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  Reset State: Clear all data"),
        Line::from("  Delete Contacts: Remove all contacts"),
        Line::from("  Delete Conversations: Remove all chats"),
        Line::from("  Rescan Relays: Reconnect and rescan"),
        Line::from("  Republish Keys: Send keypackage to relay"),
        Line::from("  Debug Log: View real-time debug output"),
        Line::from("  Fetch Messages: Check for new messages"),
        Line::from("  Fetch Invites: Check for new invites"),
        Line::from(""),
        Line::from("⚠️  Destructive actions cannot be undone!"),
    ];

    let instructions_widget = Paragraph::new(instructions)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Instructions")
            .border_style(Style::default().fg(Color::Gray)))
        .wrap(Wrap { trim: true });

    frame.render_widget(instructions_widget, chunks[1]);
}

fn render_debug_log(state: &AppState, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Log content
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    // Header
    let header = Paragraph::new("Debug Log (Live)")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Log entries (show latest entries, scrolled to bottom)
    let log_entries: Vec<ListItem> = state.debug_logs
        .iter()
        .rev() // Show newest first
        .take(chunks[1].height as usize - 2) // Account for borders
        .map(|entry| {
            let timestamp = entry.timestamp.format("%H:%M:%S%.3f");
            let level_style = match entry.level.as_str() {
                "ERROR" => Style::default().fg(Color::Red),
                "WARN" => Style::default().fg(Color::Yellow),
                "INFO" => Style::default().fg(Color::Green),
                "DEBUG" => Style::default().fg(Color::Blue),
                "TRACE" => Style::default().fg(Color::Magenta),
                _ => Style::default().fg(Color::White),
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", timestamp), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:5} ", entry.level), level_style),
                Span::styled(&entry.message, Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let log_list = List::new(log_entries)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("📋 Log Entries ({})", state.debug_logs.len()))
            .border_style(Style::default().fg(Color::Green)));

    frame.render_widget(log_list, chunks[1]);

    // Footer
    let footer = Paragraph::new("j/k: Scroll | Esc: Back to Power Tools Menu | Ctrl-C: Clear Log")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}