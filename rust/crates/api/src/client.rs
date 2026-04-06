use crate::error::ApiError;
use crate::prompt_cache::{PromptCache, PromptCacheRecord, PromptCacheStats};
use crate::providers::anthropic::{self, AnthropicClient, AuthSource};
use crate::providers::gemini;
use crate::providers::openai_compat::{self, OpenAiCompatClient, OpenAiCompatConfig};
use crate::providers::openrouter::OpenRouterClient;
use crate::providers::qwen;
use crate::providers::{self, ProviderKind};
use crate::types::{MessageRequest, MessageResponse, StreamEvent};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum ProviderClient {
    OpenRouter(OpenRouterClient),
    Anthropic(AnthropicClient),
    Xai(OpenAiCompatClient),
    OpenAi(OpenAiCompatClient),
    Gemini(OpenAiCompatClient),
    Qwen(OpenAiCompatClient),
}

impl ProviderClient {
    pub fn from_model(model: &str) -> Result<Self, ApiError> {
        Self::from_model_with_anthropic_auth(model, None)
    }

    /// Create a provider client for the given model.
    ///
    /// Auth priority (OpenRouter-first):
    /// 1. Model-based routing (if model ID maps to a specific provider)
    /// 2. OpenRouter client (if OPENROUTER_API_KEY is set)
    /// 3. Anthropic client with optional auth fallback
    pub fn from_model_with_anthropic_auth(
        model: &str,
        anthropic_auth: Option<AuthSource>,
    ) -> Result<Self, ApiError> {
        let resolved_model = providers::resolve_model_alias(model);
        match providers::detect_provider_kind(&resolved_model) {
            ProviderKind::OpenRouter => {
                Ok(Self::OpenRouter(OpenRouterClient::from_env()?))
            }
            ProviderKind::Anthropic => {
                let client = match anthropic_auth {
                    Some(auth) => AnthropicClient::from_auth(auth),
                    None => AnthropicClient::from_env()?,
                };
                Ok(Self::Anthropic(client.with_base_url(anthropic::read_base_url())))
            }
            ProviderKind::Xai => Ok(Self::Xai(OpenAiCompatClient::from_env(
                OpenAiCompatConfig::xai(),
            )?)),
            ProviderKind::OpenAi => Ok(Self::OpenAi(OpenAiCompatClient::from_env(
                OpenAiCompatConfig::openai(),
            )?)),
            ProviderKind::Gemini => {
                Ok(Self::Gemini(gemini::create_client_from_env()?))
            }
            ProviderKind::Qwen => {
                Ok(Self::Qwen(qwen::create_client_from_env()?))
            }
        }
    }

    #[must_use]
    pub const fn provider_kind(&self) -> ProviderKind {
        match self {
            Self::OpenRouter(_) => ProviderKind::OpenRouter,
            Self::Anthropic(_) => ProviderKind::Anthropic,
            Self::Xai(_) => ProviderKind::Xai,
            Self::OpenAi(_) => ProviderKind::OpenAi,
            Self::Gemini(_) => ProviderKind::Gemini,
            Self::Qwen(_) => ProviderKind::Qwen,
        }
    }

    #[must_use]
    pub fn with_prompt_cache(self, prompt_cache: PromptCache) -> Self {
        match self {
            Self::Anthropic(client) => Self::Anthropic(client.with_prompt_cache(prompt_cache)),
            other => other,
        }
    }

    #[must_use]
    pub fn prompt_cache_stats(&self) -> Option<PromptCacheStats> {
        match self {
            Self::Anthropic(client) => client.prompt_cache_stats(),
            Self::OpenRouter(_) | Self::Xai(_) | Self::OpenAi(_) | Self::Gemini(_) | Self::Qwen(_) => None,
        }
    }

    #[must_use]
    pub fn take_last_prompt_cache_record(&self) -> Option<PromptCacheRecord> {
        match self {
            Self::Anthropic(client) => client.take_last_prompt_cache_record(),
            Self::OpenRouter(_) | Self::Xai(_) | Self::OpenAi(_) | Self::Gemini(_) | Self::Qwen(_) => None,
        }
    }

    pub async fn send_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageResponse, ApiError> {
        match self {
            Self::OpenRouter(client) => client.inner().send_message(request).await,
            Self::Anthropic(client) => client.send_message(request).await,
            Self::Xai(client) | Self::OpenAi(client) | Self::Gemini(client) | Self::Qwen(client) => {
                client.send_message(request).await
            }
        }
    }

    pub async fn stream_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageStream, ApiError> {
        match self {
            Self::OpenRouter(client) => client
                .inner()
                .stream_message(request)
                .await
                .map(MessageStream::OpenAiCompat),
            Self::Anthropic(client) => client
                .stream_message(request)
                .await
                .map(MessageStream::Anthropic),
            Self::Xai(client) | Self::OpenAi(client) | Self::Gemini(client) | Self::Qwen(client) => {
                client
                    .stream_message(request)
                    .await
                    .map(MessageStream::OpenAiCompat)
            }
        }
    }
}

#[derive(Debug)]
pub enum MessageStream {
    Anthropic(anthropic::MessageStream),
    OpenAiCompat(openai_compat::MessageStream),
}

impl MessageStream {
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::Anthropic(stream) => stream.request_id(),
            Self::OpenAiCompat(stream) => stream.request_id(),
        }
    }

    pub async fn next_event(&mut self) -> Result<Option<StreamEvent>, ApiError> {
        match self {
            Self::Anthropic(stream) => stream.next_event().await,
            Self::OpenAiCompat(stream) => stream.next_event().await,
        }
    }
}

pub use anthropic::{
    oauth_token_is_expired, resolve_saved_oauth_token, resolve_startup_auth_source, OAuthTokenSet,
};

#[must_use]
pub fn read_base_url() -> String {
    anthropic::read_base_url()
}

#[must_use]
pub fn read_xai_base_url() -> String {
    openai_compat::read_base_url(OpenAiCompatConfig::xai())
}

#[must_use]
pub fn read_gemini_base_url() -> String {
    gemini::read_base_url()
}

#[must_use]
pub fn read_qwen_base_url() -> String {
    qwen::read_base_url()
}

#[cfg(test)]
mod tests {
    use crate::providers::{detect_provider_kind, resolve_model_alias, ProviderKind};

    #[test]
    fn resolves_existing_and_grok_aliases() {
        assert_eq!(resolve_model_alias("opus"), "claude-opus-4-6");
        assert_eq!(resolve_model_alias("grok"), "grok-3");
        assert_eq!(resolve_model_alias("grok-mini"), "grok-3-mini");
        assert_eq!(resolve_model_alias("or-free"), "openrouter/free");
        assert_eq!(resolve_model_alias("or-gemini"), "google/gemini-2.5-pro");
    }

    #[test]
    fn provider_detection_prefers_model_family() {
        assert_eq!(detect_provider_kind("grok-3"), ProviderKind::Xai);
        assert_eq!(
            detect_provider_kind("claude-sonnet-4-6"),
            ProviderKind::Anthropic
        );
    }

    #[test]
    fn openrouter_detected_from_provider_prefix() {
        assert_eq!(
            detect_provider_kind("anthropic/claude-sonnet-4.6"),
            ProviderKind::OpenRouter
        );
        assert_eq!(
            detect_provider_kind("google/gemini-2.5-pro"),
            ProviderKind::OpenRouter
        );
        assert_eq!(
            detect_provider_kind("openai/gpt-5"),
            ProviderKind::OpenRouter
        );
        assert_eq!(
            detect_provider_kind("qwen/qwen-max"),
            ProviderKind::OpenRouter
        );
    }
}
