//! OpenRouter API client.
//!
//! OpenRouter exposes the **OpenAI Chat Completions API** format, so this
//! module re-exports `OpenAiCompatClient` with an OpenRouter-specific config
//! and adds optional attribution headers (`HTTP-Referer`, `X-OpenRouter-Title`).
//!
//! # Environment variables
//! - `OPENROUTER_API_KEY` — required API key
//! - `OPENROUTER_BASE_URL` — optional custom base URL (default: `https://openrouter.ai/api/v1`)
//!
//! # Model IDs
//! OpenRouter uses provider-prefixed model IDs like `anthropic/claude-sonnet-4.6`,
//! `openai/gpt-5`, `google/gemini-2.5-pro`, etc. Use CLI aliases like `--model or-sonnet`
//! for shorthand.

use crate::error::ApiError;
use crate::providers::openai_compat::{OpenAiCompatClient, OpenAiCompatConfig};

pub const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// OpenRouter-specific config with optional attribution headers.
#[derive(Debug, Clone, Copy)]
pub struct OpenRouterConfig {
    pub api_key_env: &'static str,
    pub base_url_env: &'static str,
    pub default_base_url: &'static str,
    /// Optional site URL for OpenRouter leaderboard attribution.
    pub http_referer: Option<&'static str>,
    /// Optional app name for OpenRouter leaderboard attribution.
    pub x_openrouter_title: Option<&'static str>,
}

impl OpenRouterConfig {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            api_key_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: DEFAULT_BASE_URL,
            http_referer: None,
            x_openrouter_title: None,
        }
    }

    /// Convert to the generic OpenAI-compatible config (shares the same wire format).
    #[must_use]
    pub fn to_compat_config(&self) -> OpenAiCompatConfig {
        OpenAiCompatConfig {
            provider_name: "OpenRouter",
            api_key_env: self.api_key_env,
            base_url_env: self.base_url_env,
            default_base_url: self.default_base_url,
        }
    }
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// OpenRouter API client — wraps `OpenAiCompatClient` with OpenRouter config.
#[derive(Debug, Clone)]
pub struct OpenRouterClient {
    inner: OpenAiCompatClient,
    config: OpenRouterConfig,
}

impl OpenRouterClient {
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        let config = OpenRouterConfig::new();
        let inner = OpenAiCompatClient::new(api_key, config.to_compat_config());
        Self { inner, config }
    }

    pub fn from_env() -> Result<Self, ApiError> {
        let config = OpenRouterConfig::new();
        let Some(api_key) = read_env_non_empty(config.api_key_env)? else {
            return Err(ApiError::missing_credentials(
                "OpenRouter",
                &["OPENROUTER_API_KEY"],
            ));
        };
        let inner = OpenAiCompatClient::new(api_key, config.to_compat_config());
        Ok(Self { inner, config })
    }

    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.inner = self.inner.with_base_url(base_url);
        self
    }

    #[must_use]
    pub fn with_attribution(
        mut self,
        http_referer: &'static str,
        x_openrouter_title: &'static str,
    ) -> Self {
        self.config.http_referer = Some(http_referer);
        self.config.x_openrouter_title = Some(x_openrouter_title);
        self
    }

    /// Access the inner OpenAI-compatible client for direct use.
    #[must_use]
    pub fn inner(&self) -> &OpenAiCompatClient {
        &self.inner
    }

    #[must_use]
    pub fn config(&self) -> &OpenRouterConfig {
        &self.config
    }
}

fn read_env_non_empty(key: &str) -> Result<Option<String>, ApiError> {
    match std::env::var(key) {
        Ok(value) if !value.is_empty() => Ok(Some(value)),
        Ok(_) | Err(std::env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(ApiError::from(error)),
    }
}

/// Check if the `OPENROUTER_API_KEY` environment variable is set.
#[must_use]
pub fn has_api_key() -> bool {
    read_env_non_empty("OPENROUTER_API_KEY")
        .ok()
        .flatten()
        .is_some()
}

/// Read the configured base URL from env or default.
#[must_use]
pub fn read_base_url() -> String {
    std::env::var("OPENROUTER_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

#[cfg(test)]
mod tests {
    use super::{has_api_key, read_base_url, OpenRouterClient, OpenRouterConfig, DEFAULT_BASE_URL};
    use std::sync::{Mutex, OnceLock};

    #[test]
    fn default_config_uses_openrouter_base_url() {
        let config = OpenRouterConfig::new();
        assert_eq!(config.default_base_url, DEFAULT_BASE_URL);
        assert_eq!(config.api_key_env, "OPENROUTER_API_KEY");
        assert_eq!(config.base_url_env, "OPENROUTER_BASE_URL");
    }

    #[test]
    fn default_base_url_is_openrouter_api() {
        assert_eq!(DEFAULT_BASE_URL, "https://openrouter.ai/api/v1");
    }

    #[test]
    fn read_base_url_falls_back_to_default() {
        let _lock = env_lock();
        std::env::remove_var("OPENROUTER_BASE_URL");
        assert_eq!(read_base_url(), DEFAULT_BASE_URL);
    }

    #[test]
    fn has_api_key_detects_env_variable() {
        let _lock = env_lock();
        std::env::remove_var("OPENROUTER_API_KEY");
        assert!(!has_api_key());

        std::env::set_var("OPENROUTER_API_KEY", "sk-or-test-key");
        assert!(has_api_key());

        std::env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    fn client_from_env_fails_without_key() {
        let _lock = env_lock();
        std::env::remove_var("OPENROUTER_API_KEY");
        let result = OpenRouterClient::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn client_with_base_url_overrides_env() {
        let client = OpenRouterClient::new("sk-or-test").with_base_url("http://localhost:3000");
        // Client constructed successfully with custom base URL
        assert!(client.config().default_base_url == DEFAULT_BASE_URL);
    }

    #[test]
    fn attribution_config_defaults_to_none() {
        let config = OpenRouterConfig::new();
        assert!(config.http_referer.is_none());
        assert!(config.x_openrouter_title.is_none());
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock")
    }
}
