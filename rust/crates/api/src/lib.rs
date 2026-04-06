mod client;
mod error;
mod prompt_cache;
mod providers;
mod sse;
mod types;

pub use client::{
    oauth_token_is_expired, read_base_url, read_xai_base_url, resolve_saved_oauth_token,
    resolve_startup_auth_source, MessageStream, OAuthTokenSet, ProviderClient,
};
pub use error::ApiError;
pub use prompt_cache::{
    CacheBreakEvent, PromptCache, PromptCacheConfig, PromptCachePaths, PromptCacheRecord,
    PromptCacheStats,
};
pub use providers::anthropic::{AnthropicClient, AnthropicClient as ApiClient, AuthSource};
pub use providers::gemini::{
    has_api_key as gemini_has_api_key, read_base_url as _gemini_base_url,
};
pub use providers::openai_compat::{OpenAiCompatClient, OpenAiCompatConfig};
pub use providers::openrouter::{
    has_api_key as openrouter_has_api_key, read_base_url as openrouter_base_url,
    OpenRouterClient, OpenRouterConfig,
};
pub use providers::qwen::{
    has_api_key as qwen_has_api_key, read_base_url as _qwen_base_url,
};
pub use providers::{
    detect_provider_kind, max_tokens_for_model, resolve_model_alias, ProviderKind,
};
pub use sse::{parse_frame, SseParser};
pub use types::{
    ContentBlockDelta, ContentBlockDeltaEvent, ContentBlockStartEvent, ContentBlockStopEvent,
    InputContentBlock, InputMessage, MessageDelta, MessageDeltaEvent, MessageRequest,
    MessageResponse, MessageStartEvent, MessageStopEvent, OutputContentBlock, StreamEvent,
    ToolChoice, ToolDefinition, ToolResultContentBlock, Usage,
};

pub use telemetry::{
    AnalyticsEvent, AnthropicRequestProfile, ClientIdentity, JsonlTelemetrySink,
    MemoryTelemetrySink, SessionTraceRecord, SessionTracer, TelemetryEvent, TelemetrySink,
    DEFAULT_ANTHROPIC_VERSION,
};
