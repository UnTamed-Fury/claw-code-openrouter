//! Gemini API client — Google's OpenAI-compatible endpoint.
//!
//! Gemini provides an OpenAI-compatible API at:
//! `https://generativelanguage.googleapis.com/v1beta/openai/`
//!
//! This means it works with the exact same request/response format as OpenAI,
//! just with a different base URL and API key.
//!
//! # Environment variables
//! - `GEMINI_API_KEY` — required API key (Google AI Studio key)
//! - `GEMINI_BASE_URL` — optional custom base URL
//!
//! # Model IDs
//! - `gemini-2.5-pro` → Google Gemini 2.5 Pro
//! - `gemini-2.5-flash` → Google Gemini 2.5 Flash
//! - Or through OpenRouter: `google/gemini-2.5-pro`, `google/gemini-2.5-flash`

use crate::error::ApiError;
use crate::providers::openai_compat::{OpenAiCompatClient, OpenAiCompatConfig};

pub const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";

#[must_use]
pub fn has_api_key() -> bool {
    match std::env::var("GEMINI_API_KEY") {
        Ok(value) if !value.is_empty() => true,
        _ => false,
    }
}

#[must_use]
pub fn read_base_url() -> String {
    std::env::var("GEMINI_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

/// Create an OpenAI-compatible Gemini client from env.
pub fn create_client_from_env() -> Result<OpenAiCompatClient, ApiError> {
    let Some(api_key) = std::env::var("GEMINI_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())
    else {
        return Err(ApiError::missing_credentials("Gemini", &["GEMINI_API_KEY"]));
    };
    let config = OpenAiCompatConfig {
        provider_name: "Gemini",
        api_key_env: "GEMINI_API_KEY",
        base_url_env: "GEMINI_BASE_URL",
        default_base_url: DEFAULT_BASE_URL,
    };
    Ok(OpenAiCompatClient::new(api_key, config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    #[test]
    fn default_base_url_is_google() {
        assert_eq!(
            DEFAULT_BASE_URL,
            "https://generativelanguage.googleapis.com/v1beta/openai"
        );
    }

    #[test]
    fn has_api_key_detects_env() {
        let _lock = env_lock();
        std::env::remove_var("GEMINI_API_KEY");
        assert!(!has_api_key());

        std::env::set_var("GEMINI_API_KEY", "test-key");
        assert!(has_api_key());

        std::env::remove_var("GEMINI_API_KEY");
    }

    #[test]
    fn read_base_url_falls_back() {
        let _lock = env_lock();
        std::env::remove_var("GEMINI_BASE_URL");
        assert_eq!(read_base_url(), DEFAULT_BASE_URL);
    }

    #[test]
    fn create_client_fails_without_key() {
        let _lock = env_lock();
        std::env::remove_var("GEMINI_API_KEY");
        assert!(create_client_from_env().is_err());
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock")
    }
}
