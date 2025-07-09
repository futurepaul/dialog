use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    // Background colors
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_highlight: Color,
    
    // Foreground colors
    pub fg_primary: Color,
    pub fg_secondary: Color,
    pub fg_accent: Color,
    
    // Semantic colors
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    
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
            bg_highlight: Color::Rgb(48, 49, 59),    // Selected items
            
            // Foreground
            fg_primary: Color::Rgb(248, 248, 242),   // Main text
            fg_secondary: Color::Rgb(139, 143, 150), // Muted text
            fg_accent: Color::Rgb(139, 233, 253),    // Commands/highlights
            
            // Semantic
            success: Color::Rgb(80, 250, 123),       // Success messages
            error: Color::Rgb(255, 85, 85),          // Error messages
            warning: Color::Rgb(255, 184, 108),      // Warnings
            info: Color::Rgb(189, 147, 249),         // Info messages
            
            // Borders
            border: Color::Rgb(68, 71, 90),          // UI borders
            border_focused: Color::Rgb(139, 233, 253), // Focused borders
        }
    }
    
    // Style helpers
    pub fn base_style(&self) -> Style {
        Style::default()
            .fg(self.fg_primary)
            .bg(self.bg_primary)
    }
    
    pub fn input_style(&self) -> Style {
        Style::default()
            .fg(self.fg_primary)
            .bg(self.bg_secondary)
    }
    
    pub fn status_style(&self) -> Style {
        Style::default()
            .fg(self.fg_secondary)
            .bg(self.bg_secondary)
    }
    
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.fg_primary)
            .bg(self.bg_highlight)
    }
    
    pub fn command_style(&self) -> Style {
        Style::default()
            .fg(self.fg_accent)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn error_style(&self) -> Style {
        Style::default()
            .fg(self.error)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn success_style(&self) -> Style {
        Style::default()
            .fg(self.success)
    }
    
    pub fn muted_style(&self) -> Style {
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