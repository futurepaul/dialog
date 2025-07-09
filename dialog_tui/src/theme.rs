use ratatui::style::{Color, Style};

pub struct Theme {
    // Background colors
    pub bg_primary: Color,
    pub bg_secondary: Color,
    
    // Foreground colors
    pub fg_primary: Color,
    pub fg_secondary: Color,
    
    // Border colors
    pub border: Color,
    pub border_focused: Color,
}

impl Theme {
    pub fn claude_code() -> Self {
        Self {
            // Background
            bg_primary: Color::Rgb(30, 31, 38),      // Main background
            bg_secondary: Color::Rgb(39, 40, 49),    // Input/status bar
            
            // Foreground
            fg_primary: Color::Rgb(248, 248, 242),   // Main text
            fg_secondary: Color::Rgb(139, 143, 150), // Muted text
            
            // Borders
            border: Color::Rgb(68, 71, 90),          // UI borders
            border_focused: Color::Rgb(139, 233, 253), // Focused borders
        }
    }
    
    // Style helpers - only the ones we actually use
    pub fn input_style(&self) -> Style {
        Style::default()
            .fg(self.fg_primary)
            .bg(self.bg_secondary)
    }
    
    pub fn status_style(&self) -> Style {
        Style::default()
            .fg(self.fg_secondary)
    }
    
    pub fn border_style(&self) -> Style {
        Style::default()
            .fg(self.border)
    }
    
    pub fn border_focused_style(&self) -> Style {
        Style::default()
            .fg(self.border_focused)
    }

    pub fn background_style(&self) -> Style {
        Style::default()
            .bg(self.bg_primary)
    }

    pub fn text_style(&self) -> Style {
        Style::default()
            .fg(self.fg_primary)
    }
}