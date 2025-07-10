use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear, ListState},
    style::{Style, Color, Modifier},
    Frame,
    text::{Line, Span},
};

use crate::{
    app::{App, SelectionMode, MessageType},
    theme::Theme,
};

pub fn draw(f: &mut Frame, app: &App) {
    let theme = Theme::claude_code();
    
    // Create fullscreen layout with messages area, input area, and status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Messages area (takes remaining space)
            Constraint::Length(3), // Text input area (with borders)
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // Draw messages area
    draw_messages(f, chunks[0], app, &theme);

    // Draw text input area
    draw_text_input(f, chunks[1], app, &theme);

    // Draw status bar
    draw_status_bar(f, chunks[2], app, &theme);
    
    // Draw search suggestions overlay if in search mode
    if app.is_in_search_mode() {
        draw_search_suggestions(f, chunks[1], app, &theme);
    }
    
    // Draw selection mode overlay if active
    if !matches!(app.selection_mode, SelectionMode::None) {
        draw_selection_mode(f, app, &theme);
    }
}

fn draw_messages(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let messages_block = Block::default()
        .borders(Borders::ALL)
        .title("Dialog")
        .border_style(theme.border_style())
        .style(theme.background_style());

    let inner_area = messages_block.inner(area);
    
    // Calculate how many messages can fit in the area
    let visible_height = inner_area.height as usize;
    let total_messages = app.messages.len();
    
    // Determine which messages to show based on scroll position
    let start_idx = if total_messages <= visible_height {
        0
    } else {
        total_messages.saturating_sub(visible_height)
    };
    
    let visible_messages: Vec<ListItem> = app.messages
        .iter()
        .skip(start_idx)
        .take(visible_height)
        .map(|msg| {
            let style = match msg.message_type {
                MessageType::Info => Style::default().fg(Color::Gray),
                MessageType::Success => Style::default().fg(Color::Green),
                MessageType::Warning => Style::default().fg(Color::Yellow),
                MessageType::Error => Style::default().fg(Color::Red),
                MessageType::Normal => theme.text_style(),
            };
            ListItem::new(msg.content.as_str()).style(style)
        })
        .collect();

    let messages_list = List::new(visible_messages)
        .style(theme.text_style());

    f.render_widget(messages_block, area);
    f.render_widget(messages_list, inner_area);
}

fn draw_text_input(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused_style())
        .style(theme.input_style());

    let inner_area = input_block.inner(area);
    f.render_widget(input_block, area);

    // Render the text area widget
    f.render_widget(app.text_area.widget(), inner_area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let status_text = app.get_status_text();
    
    let status = Paragraph::new(status_text)
        .style(theme.status_style());

    f.render_widget(status, area);
}

fn draw_search_suggestions(f: &mut Frame, input_area: Rect, app: &App, _theme: &Theme) {
    let (suggestions_len, title) = if app.is_chat_switching() {
        let conv_suggestions = app.get_conversation_suggestions();
        (conv_suggestions.len(), "@ Chat Switcher")
    } else {
        (0, "@ Contact Search")
    };

    if suggestions_len == 0 {
        return;
    }

    // Calculate popup area - position it above the input area
    let suggestion_height = std::cmp::min(suggestions_len + 2, 8) as u16; // +2 for borders, max 8 high
    let popup_width = std::cmp::min(60, input_area.width.saturating_sub(4)); // Leave some margin
    
    let popup_area = Rect {
        x: input_area.x + 2,
        y: input_area.y.saturating_sub(suggestion_height),
        width: popup_width,
        height: suggestion_height,
    };

    // Clear the area first
    f.render_widget(Clear, popup_area);

    // Create suggestion items
    let selected_idx = app.get_selected_suggestion();
    let items: Vec<ListItem> = app.get_conversation_suggestions()
        .iter()
        .enumerate()
        .map(|(i, suggestion)| {
            let style = if i == selected_idx {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(suggestion.display_text.as_str()).style(style)
        })
        .collect();

    let suggestions_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_alignment(Alignment::Left)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::DarkGray))
        );

    f.render_widget(suggestions_list, popup_area);
}

fn draw_selection_mode(f: &mut Frame, app: &App, theme: &Theme) {
    match &app.selection_mode {
        SelectionMode::None => return,
        SelectionMode::InviteSelection { invites, state } => {
            draw_invite_selection(f, invites, state, theme);
        }
        SelectionMode::ConversationSelection { state } => {
            draw_conversation_selection(f, &app.conversations, state, theme);
        }
        SelectionMode::ContactSelection { group_name, selections, state } => {
            draw_contact_selection(f, group_name, &app.contacts, selections, state, theme);
        }
    }
}

fn draw_invite_selection(f: &mut Frame, invites: &[dialog_lib::PendingInvite], state: &ListState, theme: &Theme) {
    let area = centered_rect(80, 80, f.area());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    let items: Vec<ListItem> = invites.iter().map(|invite| {
        ListItem::new(vec![
            Line::from(vec![
                Span::styled(&invite.group_name, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw(format!("  {} members", invite.member_count)),
            ]),
        ])
    }).collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Select Invite to Accept")
            .border_style(theme.border_focused_style()))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, area, &mut state.clone());
    
    // Help text
    let help = Paragraph::new("↑↓/jk: Navigate | Enter: Accept | Esc: Cancel")
        .style(theme.help_style())
        .alignment(Alignment::Center);
    
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(help, help_area);
}

fn draw_conversation_selection(f: &mut Frame, conversations: &[dialog_lib::Conversation], state: &ListState, theme: &Theme) {
    let area = centered_rect(80, 80, f.area());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    let items: Vec<ListItem> = conversations.iter().enumerate().map(|(i, conv)| {
        let group_indicator = if conv.is_group { "[GROUP] " } else { "" };
        let unread = if conv.unread_count > 0 {
            format!(" ({} unread)", conv.unread_count)
        } else {
            String::new()
        };
        
        ListItem::new(vec![
            Line::from(vec![
                Span::raw(format!("{}: {}{}", i + 1, group_indicator, conv.name)),
                Span::styled(unread, Style::default().fg(Color::Red)),
            ]),
        ])
    }).collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Select Conversation")
            .border_style(theme.border_focused_style()))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, area, &mut state.clone());
    
    // Help text
    let help = Paragraph::new("↑↓/jk: Navigate | Enter: Switch | Esc: Cancel")
        .style(theme.help_style())
        .alignment(Alignment::Center);
    
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(help, help_area);
}

fn draw_contact_selection(f: &mut Frame, group_name: &str, contacts: &[dialog_lib::Contact], selections: &[bool], state: &ListState, theme: &Theme) {
    let area = centered_rect(80, 80, f.area());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    let items: Vec<ListItem> = contacts.iter().zip(selections.iter()).map(|(contact, selected)| {
        let checkbox = if *selected { "[x]" } else { "[ ]" };
        let status = if contact.online { "(online)" } else { "(offline)" };
        
        ListItem::new(vec![
            Line::from(vec![
                Span::raw(format!("{} {} ", checkbox, contact.name)),
                Span::styled(status, Style::default().fg(if contact.online { Color::Green } else { Color::Gray })),
            ]),
        ])
    }).collect();
    
    let title = format!("Select Contacts for '{}'", group_name);
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(theme.border_focused_style()))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, area, &mut state.clone());
    
    // Help text
    let help = Paragraph::new("↑↓/jk: Navigate | Space: Toggle | Enter: Create | Esc: Cancel")
        .style(theme.help_style())
        .alignment(Alignment::Center);
    
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(help, help_area);
}

/// Helper function to create a centered rect using percentage of the available area
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}