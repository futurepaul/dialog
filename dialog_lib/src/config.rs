use std::env;

#[derive(Debug, Clone)]
pub struct DialogConfig {
    pub relay_url: String,
}

impl Default for DialogConfig {
    fn default() -> Self {
        Self {
            relay_url: "ws://localhost:10547".to_string(),
        }
    }
}

impl DialogConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_env() -> Self {
        let relay_url = env::var("DIALOG_RELAY_URL")
            .unwrap_or_else(|_| "ws://localhost:10547".to_string());

        Self {
            relay_url,
        }
    }

    pub fn with_relay_url(relay_url: impl Into<String>) -> Self {
        Self {
            relay_url: relay_url.into(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DialogConfig::default();
        assert_eq!(config.relay_url, "ws://localhost:10547");
    }

    #[test]
    fn test_with_relay_url() {
        let config = DialogConfig::with_relay_url("ws://custom.relay");
        assert_eq!(config.relay_url, "ws://custom.relay");
    }
}