//! Qwen API client — Alibaba DashScope OpenAI-compatible endpoint.
//!
//! Qwen (via DashScope) provides an OpenAI-compatible API at:
//! `https://dashscope.aliyuncs.com/compatible-mode/v1`
//!
//! This means it works with the exact same request/response format as OpenAI,
//! just with a different base URL and API key.
//!
//! # Environment variables
//! - `QWEN_API_KEY` — required API key (DashScope API key)
//! - `QWEN_BASE_URL` — optional custom base URL
//!
//! # Model IDs
//! - `qwen-max` → Qwen Max
//! - `qwen-plus` → Qwen Plus
//! - Or through OpenRouter: `qwen/qwen3-235b-a22b`, `qwen/qwen-max`, `qwen/qwen-plus`

use crate::error::ApiError;
use crate::providers::openai_compat::{OpenAiCompatClient, OpenAiCompatConfig};

pub const DEFAULT_BASE_URL: &str = "https://dashscope.aliyuncs.com/compatible-mode/v1";

#[must_use]
pub fn has_api_key() -> bool {
    match std::env::var("QWEN_API_KEY") {
        Ok(value) if !value.is_empty() => true,
        _ => false,
    }
}

#[must_use]
pub fn read_base_url() -> String {
    std::env::var("QWEN_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

/// Create an OpenAI-compatible Qwen client from env.
pub fn create_client_from_env() -> Result<OpenAiCompatClient, ApiError> {
    let Some(api_key) = std::env::var("QWEN_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())
    else {
        return Err(ApiError::missing_credentials("Qwen", &["QWEN_API_KEY"]));
    };
    let config = OpenAiCompatConfig {
        provider_name: "Qwen",
        api_key_env: "QWEN_API_KEY",
        base_url_env: "QWEN_BASE_URL",
        default_base_url: DEFAULT_BASE_URL,
    };
    Ok(OpenAiCompatClient::new(api_key, config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    #[test]
    fn default_base_url_is_dashscope() {
        assert_eq!(
            DEFAULT_BASE_URL,
            "https://dashscope.aliyuncs.com/compatible-mode/v1"
        );
    }

    #[test]
    fn has_api_key_detects_env() {
        let _lock = env_lock();
        std::env::remove_var("QWEN_API_KEY");
        assert!(!has_api_key());

        std::env::set_var("QWEN_API_KEY", "test-key");
        assert!(has_api_key());

        std::env::remove_var("QWEN_API_KEY");
    }

    #[test]
    fn read_base_url_falls_back() {
        let _lock = env_lock();
        std::env::remove_var("QWEN_BASE_URL");
        assert_eq!(read_base_url(), DEFAULT_BASE_URL);
    }

    #[test]
    fn create_client_fails_without_key() {
        let _lock = env_lock();
        std::env::remove_var("QWEN_API_KEY");
        assert!(create_client_from_env().is_err());
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock")
    }
}
