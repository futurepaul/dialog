use std::env;

#[derive(Debug, Clone)]
pub struct DialogConfig {
    pub mode: ServiceMode,
    pub relay_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceMode {
    Mock,
    Real,
}

impl Default for DialogConfig {
    fn default() -> Self {
        Self {
            mode: ServiceMode::Mock,
            relay_url: "ws://localhost:8080".to_string(),
        }
    }
}

impl DialogConfig {
    pub fn from_env() -> Self {
        let mode = match env::var("DIALOG_MODE").as_deref() {
            Ok("real") => ServiceMode::Real,
            Ok("mock") | _ => ServiceMode::Mock,
        };

        let relay_url = env::var("DIALOG_RELAY_URL")
            .unwrap_or_else(|_| "ws://localhost:8080".to_string());

        Self {
            mode,
            relay_url,
        }
    }

    pub fn builder() -> DialogConfigBuilder {
        DialogConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct DialogConfigBuilder {
    mode: Option<ServiceMode>,
    relay_url: Option<String>,
}

impl DialogConfigBuilder {
    pub fn mode(mut self, mode: ServiceMode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn relay_url(mut self, url: impl Into<String>) -> Self {
        self.relay_url = Some(url.into());
        self
    }

    pub fn build(self) -> DialogConfig {
        let default = DialogConfig::default();
        DialogConfig {
            mode: self.mode.unwrap_or(default.mode),
            relay_url: self.relay_url.unwrap_or(default.relay_url),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DialogConfig::default();
        assert_eq!(config.mode, ServiceMode::Mock);
        assert_eq!(config.relay_url, "ws://localhost:8080");
    }

    #[test]
    fn test_builder() {
        let config = DialogConfig::builder()
            .mode(ServiceMode::Real)
            .relay_url("ws://custom.relay")
            .build();

        assert_eq!(config.mode, ServiceMode::Real);
        assert_eq!(config.relay_url, "ws://custom.relay");
    }
}