use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{
    app::App,
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
        .map(|msg| ListItem::new(msg.as_str()))
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