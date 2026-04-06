#![allow(clippy::cast_possible_truncation)]
use std::future::Future;
use std::pin::Pin;

use serde::Serialize;

use crate::error::ApiError;
use crate::types::{MessageRequest, MessageResponse};

pub mod anthropic;
pub mod gemini;
pub mod openai_compat;
pub mod openrouter;
pub mod qwen;

#[allow(dead_code)]
pub type ProviderFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ApiError>> + Send + 'a>>;

#[allow(dead_code)]
pub trait Provider {
    type Stream;

    fn send_message<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> ProviderFuture<'a, MessageResponse>;

    fn stream_message<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> ProviderFuture<'a, Self::Stream>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    OpenRouter,
    Anthropic,
    Xai,
    OpenAi,
    Gemini,
    Qwen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderMetadata {
    pub provider: ProviderKind,
    pub auth_env: &'static str,
    pub base_url_env: &'static str,
    pub default_base_url: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelTokenLimit {
    pub max_output_tokens: u32,
    pub context_window_tokens: u32,
}

const MODEL_REGISTRY: &[(&str, ProviderMetadata)] = &[
    // ─── OpenRouter (PRIMARY) ──────────────────────────────────────
    (
        "free",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-free",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-sonnet",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-opus",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-haiku",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-gpt-4o",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-gpt-5",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    // Gemini through OpenRouter
    (
        "or-gemini",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-gemini-flash",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-gemini-pro",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    // Qwen through OpenRouter
    (
        "or-qwen",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-qwen-max",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-qwen-plus",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-grok",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-mistral",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-llama",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    (
        "or-deepseek",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        },
    ),
    // ─── Anthropic (secondary) ─────────────────────────────────────
    (
        "opus",
        ProviderMetadata {
            provider: ProviderKind::Anthropic,
            auth_env: "ANTHROPIC_API_KEY",
            base_url_env: "ANTHROPIC_BASE_URL",
            default_base_url: anthropic::DEFAULT_BASE_URL,
        },
    ),
    (
        "sonnet",
        ProviderMetadata {
            provider: ProviderKind::Anthropic,
            auth_env: "ANTHROPIC_API_KEY",
            base_url_env: "ANTHROPIC_BASE_URL",
            default_base_url: anthropic::DEFAULT_BASE_URL,
        },
    ),
    (
        "haiku",
        ProviderMetadata {
            provider: ProviderKind::Anthropic,
            auth_env: "ANTHROPIC_API_KEY",
            base_url_env: "ANTHROPIC_BASE_URL",
            default_base_url: anthropic::DEFAULT_BASE_URL,
        },
    ),
    // ─── XAI / Grok (secondary) ────────────────────────────────────
    (
        "grok",
        ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        },
    ),
    (
        "grok-3",
        ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        },
    ),
    (
        "grok-mini",
        ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        },
    ),
    (
        "grok-3-mini",
        ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        },
    ),
    (
        "grok-2",
        ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        },
    ),
];

#[must_use]
pub fn resolve_model_alias(model: &str) -> String {
    let trimmed = model.trim();
    let lower = trimmed.to_ascii_lowercase();
    MODEL_REGISTRY
        .iter()
        .find_map(|(alias, metadata)| {
            (*alias == lower).then_some(match metadata.provider {
                // OpenRouter aliases (PRIMARY)
                ProviderKind::OpenRouter => match *alias {
                    "free" | "or-free" => "openrouter/free",
                    "or-sonnet" => "anthropic/claude-sonnet-4.6",
                    "or-opus" => "anthropic/claude-opus-4.6",
                    "or-haiku" => "anthropic/claude-3-5-haiku-20241022",
                    "or-gpt-4o" => "openai/gpt-4o",
                    "or-gpt-5" => "openai/gpt-5",
                    "or-gemini" => "google/gemini-2.5-pro",
                    "or-gemini-flash" => "google/gemini-2.5-flash",
                    "or-gemini-pro" => "google/gemini-2.5-pro",
                    "or-qwen" => "qwen/qwen3-235b-a22b",
                    "or-qwen-max" => "qwen/qwen-max",
                    "or-qwen-plus" => "qwen/qwen-plus",
                    "or-grok" => "x-ai/grok-3",
                    "or-mistral" => "mistralai/mistral-large-2411",
                    "or-llama" => "meta-llama/llama-4-maverick",
                    "or-deepseek" => "deepseek/deepseek-chat-v3",
                    _ => trimmed,
                },
                ProviderKind::Anthropic => match *alias {
                    "opus" => "claude-opus-4-6",
                    "sonnet" => "claude-sonnet-4-6",
                    "haiku" => "claude-haiku-4-5-20251213",
                    _ => trimmed,
                },
                ProviderKind::Xai => match *alias {
                    "grok" | "grok-3" => "grok-3",
                    "grok-mini" | "grok-3-mini" => "grok-3-mini",
                    "grok-2" => "grok-2",
                    _ => trimmed,
                },
                ProviderKind::Gemini => match *alias {
                    "gemini-flash" => "gemini-2.5-flash",
                    "gemini-pro" => "gemini-2.5-pro",
                    _ => trimmed,
                },
                ProviderKind::Qwen => match *alias {
                    "qwen-max" => "qwen-max",
                    "qwen-plus" => "qwen-plus",
                    _ => trimmed,
                },
                ProviderKind::OpenAi => trimmed,
            })
        })
        .map_or_else(|| trimmed.to_string(), ToOwned::to_owned)
}

#[must_use]
pub fn metadata_for_model(model: &str) -> Option<ProviderMetadata> {
    let canonical = resolve_model_alias(model);

    // OpenRouter: provider-prefixed model IDs (PRIMARY)
    if canonical.starts_with("anthropic/")
        || canonical.starts_with("openai/")
        || canonical.starts_with("google/")
        || canonical.starts_with("qwen/")
        || canonical.starts_with("x-ai/")
        || canonical.starts_with("mistralai/")
        || canonical.starts_with("openrouter/")
        || canonical.starts_with("meta-llama/")
        || canonical.starts_with("deepseek/")
    {
        return Some(ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openrouter::DEFAULT_BASE_URL,
        });
    }

    // Direct Gemini (Google native / OpenAI-compatible)
    if canonical.starts_with("gemini") {
        return Some(ProviderMetadata {
            provider: ProviderKind::Gemini,
            auth_env: "GEMINI_API_KEY",
            base_url_env: "GEMINI_BASE_URL",
            default_base_url: gemini::DEFAULT_BASE_URL,
        });
    }

    // Direct Qwen (DashScope OpenAI-compatible)
    if canonical.starts_with("qwen") {
        return Some(ProviderMetadata {
            provider: ProviderKind::Qwen,
            auth_env: "QWEN_API_KEY",
            base_url_env: "QWEN_BASE_URL",
            default_base_url: qwen::DEFAULT_BASE_URL,
        });
    }

    // Anthropic (secondary — direct API)
    if canonical.starts_with("claude") {
        return Some(ProviderMetadata {
            provider: ProviderKind::Anthropic,
            auth_env: "ANTHROPIC_API_KEY",
            base_url_env: "ANTHROPIC_BASE_URL",
            default_base_url: anthropic::DEFAULT_BASE_URL,
        });
    }

    // XAI / Grok (secondary)
    if canonical.starts_with("grok") {
        return Some(ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        });
    }
    None
}

#[must_use]
pub fn detect_provider_kind(model: &str) -> ProviderKind {
    if let Some(metadata) = metadata_for_model(model) {
        return metadata.provider;
    }
    // Auth-based detection: OpenRouter first (PRIMARY)
    if openrouter::has_api_key() {
        return ProviderKind::OpenRouter;
    }
    // Then other providers
    if openai_compat::has_api_key("XAI_API_KEY") {
        return ProviderKind::Xai;
    }
    if openai_compat::has_api_key("OPENAI_API_KEY") {
        return ProviderKind::OpenAi;
    }
    if gemini::has_api_key() {
        return ProviderKind::Gemini;
    }
    if qwen::has_api_key() {
        return ProviderKind::Qwen;
    }
    // Anthropic last (secondary)
    if anthropic::has_auth_from_env_or_saved().unwrap_or(false) {
        return ProviderKind::Anthropic;
    }
    // Default: OpenRouter (PRIMARY)
    ProviderKind::OpenRouter
}

#[must_use]
pub fn max_tokens_for_model(model: &str) -> u32 {
    model_token_limit(model).map_or_else(
        || {
            let canonical = resolve_model_alias(model);
            if canonical.contains("opus") {
                32_000
            } else {
                64_000
            }
        },
        |limit| limit.max_output_tokens,
    )
}

#[must_use]
pub fn model_token_limit(model: &str) -> Option<ModelTokenLimit> {
    let canonical = resolve_model_alias(model);
    match canonical.as_str() {
        // OpenRouter models
        "openrouter/free" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 131_072,
        }),
        "anthropic/claude-sonnet-4.6" | "anthropic/claude-opus-4.6" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 200_000,
        }),
        "anthropic/claude-3-5-haiku-20241022" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 200_000,
        }),
        "openai/gpt-4o" | "openai/gpt-5" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 200_000,
        }),
        "google/gemini-2.5-pro" | "google/gemini-2.5-flash" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 1_000_000,
        }),
        "x-ai/grok-3" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 131_072,
        }),
        "qwen/qwen3-235b-a22b" | "qwen/qwen-max" | "qwen/qwen-plus" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 131_072,
        }),
        "mistralai/mistral-large-2411" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 131_072,
        }),
        "meta-llama/llama-4-maverick" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 131_072,
        }),
        "deepseek/deepseek-chat-v3" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 131_072,
        }),
        // Direct Anthropic
        "claude-opus-4-6" => Some(ModelTokenLimit {
            max_output_tokens: 32_000,
            context_window_tokens: 200_000,
        }),
        "claude-sonnet-4-6" | "claude-haiku-4-5-20251213" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 200_000,
        }),
        // Direct XAI
        "grok-3" | "grok-3-mini" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 131_072,
        }),
        // Direct Gemini
        "gemini-2.5-pro" | "gemini-2.5-flash" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 1_000_000,
        }),
        // Direct Qwen
        "qwen-max" | "qwen-plus" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 131_072,
        }),
        _ => None,
    }
}

pub fn preflight_message_request(request: &MessageRequest) -> Result<(), ApiError> {
    let Some(limit) = model_token_limit(&request.model) else {
        return Ok(());
    };

    let estimated_input_tokens = estimate_message_request_input_tokens(request);
    let estimated_total_tokens = estimated_input_tokens.saturating_add(request.max_tokens);
    if estimated_total_tokens > limit.context_window_tokens {
        return Err(ApiError::ContextWindowExceeded {
            model: resolve_model_alias(&request.model),
            estimated_input_tokens,
            requested_output_tokens: request.max_tokens,
            estimated_total_tokens,
            context_window_tokens: limit.context_window_tokens,
        });
    }

    Ok(())
}

fn estimate_message_request_input_tokens(request: &MessageRequest) -> u32 {
    let mut estimate = estimate_serialized_tokens(&request.messages);
    estimate = estimate.saturating_add(estimate_serialized_tokens(&request.system));
    estimate = estimate.saturating_add(estimate_serialized_tokens(&request.tools));
    estimate = estimate.saturating_add(estimate_serialized_tokens(&request.tool_choice));
    estimate
}

fn estimate_serialized_tokens<T: Serialize>(value: &T) -> u32 {
    serde_json::to_vec(value)
        .ok()
        .map_or(0, |bytes| (bytes.len() / 4 + 1) as u32)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::error::ApiError;
    use crate::types::{
        InputContentBlock, InputMessage, MessageRequest, ToolChoice, ToolDefinition,
    };

    use super::{
        detect_provider_kind, max_tokens_for_model, model_token_limit, preflight_message_request,
        resolve_model_alias, ProviderKind,
    };

    #[test]
    fn resolves_grok_aliases() {
        assert_eq!(resolve_model_alias("grok"), "grok-3");
        assert_eq!(resolve_model_alias("grok-mini"), "grok-3-mini");
        assert_eq!(resolve_model_alias("grok-2"), "grok-2");
    }

    #[test]
    fn detects_provider_from_model_name_first() {
        assert_eq!(detect_provider_kind("grok"), ProviderKind::Xai);
        assert_eq!(
            detect_provider_kind("claude-sonnet-4-6"),
            ProviderKind::Anthropic
        );
        assert_eq!(
            detect_provider_kind("or-free"),
            ProviderKind::OpenRouter
        );
    }

    #[test]
    fn keeps_existing_max_token_heuristic() {
        assert_eq!(max_tokens_for_model("opus"), 32_000);
        assert_eq!(max_tokens_for_model("grok-3"), 64_000);
    }

    #[test]
    fn returns_context_window_metadata_for_supported_models() {
        assert_eq!(
            model_token_limit("claude-sonnet-4-6")
                .expect("claude-sonnet-4-6 should be registered")
                .context_window_tokens,
            200_000
        );
        assert_eq!(
            model_token_limit("grok-mini")
                .expect("grok-mini should resolve to a registered model")
                .context_window_tokens,
            131_072
        );
    }

    #[test]
    fn preflight_blocks_requests_that_exceed_the_model_context_window() {
        let request = MessageRequest {
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 64_000,
            messages: vec![InputMessage {
                role: "user".to_string(),
                content: vec![InputContentBlock::Text {
                    text: "x".repeat(600_000),
                }],
            }],
            system: Some("Keep the answer short.".to_string()),
            tools: Some(vec![ToolDefinition {
                name: "weather".to_string(),
                description: Some("Fetches weather".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": { "city": { "type": "string" } },
                }),
            }]),
            tool_choice: Some(ToolChoice::Auto),
            stream: true,
        };

        let error = preflight_message_request(&request)
            .expect_err("oversized request should be rejected before the provider call");

        match error {
            ApiError::ContextWindowExceeded {
                model,
                estimated_input_tokens,
                requested_output_tokens,
                estimated_total_tokens,
                context_window_tokens,
            } => {
                assert_eq!(model, "claude-sonnet-4-6");
                assert!(estimated_input_tokens > 136_000);
                assert_eq!(requested_output_tokens, 64_000);
                assert!(estimated_total_tokens > context_window_tokens);
                assert_eq!(context_window_tokens, 200_000);
            }
            other => panic!("expected context-window preflight failure, got {other:?}"),
        }
    }

    #[test]
    fn preflight_skips_unknown_models() {
        let request = MessageRequest {
            model: "unknown-model".to_string(),
            max_tokens: 64_000,
            messages: vec![InputMessage {
                role: "user".to_string(),
                content: vec![InputContentBlock::Text {
                    text: "x".repeat(600_000),
                }],
            }],
            system: None,
            tools: None,
            tool_choice: None,
            stream: false,
        };

        preflight_message_request(&request)
            .expect("models without context metadata should skip the guarded preflight");
    }

    #[test]
    fn resolves_openrouter_aliases() {
        assert_eq!(resolve_model_alias("or-free"), "openrouter/free");
        assert_eq!(resolve_model_alias("free"), "openrouter/free");
        assert_eq!(resolve_model_alias("or-sonnet"), "anthropic/claude-sonnet-4.6");
        assert_eq!(resolve_model_alias("or-opus"), "anthropic/claude-opus-4.6");
        assert_eq!(resolve_model_alias("or-gpt-5"), "openai/gpt-5");
        assert_eq!(resolve_model_alias("or-gemini"), "google/gemini-2.5-pro");
        assert_eq!(resolve_model_alias("or-gemini-flash"), "google/gemini-2.5-flash");
        assert_eq!(resolve_model_alias("or-qwen"), "qwen/qwen3-235b-a22b");
        assert_eq!(resolve_model_alias("or-qwen-max"), "qwen/qwen-max");
        assert_eq!(resolve_model_alias("or-mistral"), "mistralai/mistral-large-2411");
        assert_eq!(resolve_model_alias("or-llama"), "meta-llama/llama-4-maverick");
        assert_eq!(resolve_model_alias("or-deepseek"), "deepseek/deepseek-chat-v3");
    }

    #[test]
    fn detects_openrouter_from_model_prefix() {
        assert_eq!(
            detect_provider_kind("anthropic/claude-sonnet-4.6"),
            ProviderKind::OpenRouter
        );
        assert_eq!(
            detect_provider_kind("openai/gpt-5"),
            ProviderKind::OpenRouter
        );
        assert_eq!(
            detect_provider_kind("google/gemini-2.5-pro"),
            ProviderKind::OpenRouter
        );
        assert_eq!(
            detect_provider_kind("qwen/qwen-max"),
            ProviderKind::OpenRouter
        );
        assert_eq!(
            detect_provider_kind("meta-llama/llama-4-maverick"),
            ProviderKind::OpenRouter
        );
        assert_eq!(
            detect_provider_kind("deepseek/deepseek-chat-v3"),
            ProviderKind::OpenRouter
        );
        assert_eq!(detect_provider_kind("or-free"), ProviderKind::OpenRouter);
        assert_eq!(detect_provider_kind("or-sonnet"), ProviderKind::OpenRouter);
    }

    #[test]
    fn openrouter_models_have_context_window_metadata() {
        assert_eq!(
            model_token_limit("or-free")
                .expect("or-free should be registered")
                .context_window_tokens,
            131_072
        );
        assert_eq!(
            model_token_limit("or-gemini")
                .expect("or-gemini should be registered")
                .context_window_tokens,
            1_000_000
        );
        assert_eq!(
            model_token_limit("or-qwen")
                .expect("or-qwen should be registered")
                .context_window_tokens,
            131_072
        );
        assert_eq!(
            model_token_limit("anthropic/claude-sonnet-4.6")
                .expect("provider-prefixed model should be registered")
                .context_window_tokens,
            200_000
        );
        assert_eq!(
            model_token_limit("or-mistral")
                .expect("or-mistral should be registered")
                .context_window_tokens,
            131_072
        );
        assert_eq!(
            model_token_limit("or-llama")
                .expect("or-llama should be registered")
                .context_window_tokens,
            131_072
        );
        assert_eq!(
            model_token_limit("or-deepseek")
                .expect("or-deepseek should be registered")
                .context_window_tokens,
            131_072
        );
    }
}
