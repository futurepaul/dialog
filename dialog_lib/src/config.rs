use std::env;

#[derive(Debug, Clone)]
pub struct DialogConfig {
    pub relay_urls: Vec<String>,
}

impl Default for DialogConfig {
    fn default() -> Self {
        Self {
            relay_urls: vec![
                "ws://localhost:10547".to_string(),
                "ws://localhost:8080".to_string(),
                "ws://localhost:7777".to_string(),
            ],
        }
    }
}

impl DialogConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_env() -> Self {
        let relay_urls = if let Ok(urls) = env::var("DIALOG_RELAY_URLS") {
            urls.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            Self::default().relay_urls
        };

        Self {
            relay_urls,
        }
    }

    pub fn with_relay_url(relay_url: impl Into<String>) -> Self {
        Self {
            relay_urls: vec![relay_url.into()],
        }
    }

    pub fn with_relay_urls(relay_urls: Vec<String>) -> Self {
        Self {
            relay_urls,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DialogConfig::default();
        assert_eq!(config.relay_urls, vec![
            "ws://localhost:10547".to_string(),
            "ws://localhost:8080".to_string(),
            "ws://localhost:7777".to_string(),
        ]);
    }

    #[test]
    fn test_with_relay_url() {
        let config = DialogConfig::with_relay_url("ws://custom.relay");
        assert_eq!(config.relay_urls, vec!["ws://custom.relay".to_string()]);
    }
}