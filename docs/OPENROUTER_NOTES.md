# OpenRouter API Study Notes

> Comparison of Anthropic, OpenAI-compatible, and OpenRouter APIs for integration planning.

---

## 1. Anthropic Messages API (Current Implementation)

### Endpoint
- **Base URL:** `https://api.anthropic.com`
- **Endpoint:** `POST /v1/messages`
- **Token Counting:** `POST /v1/messages/count_tokens`

### Authentication
- **Header:** `x-api-key: <API_KEY>` (NOT Bearer)
- **Version Header:** `anthropic-version: 2023-06-01`
- **Optional:** `anthropic-beta: <beta-features>`
- **Content-Type:** `application/json`
- **Env vars:** `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`

### Request Format
```json
{
  "model": "claude-sonnet-4-6",
  "max_tokens": 64000,
  "messages": [
    {"role": "user", "content": "Hello"}
  ],
  "system": "You are a helpful assistant.",
  "tools": [
    {
      "name": "get_weather",
      "description": "Get weather for a city",
      "input_schema": {
        "type": "object",
        "properties": {"city": {"type": "string"}},
        "required": ["city"]
      }
    }
  ],
  "tool_choice": {"type": "auto"},
  "stream": false,
  "temperature": 1.0
}
```

### Response Format (non-streaming)
```json
{
  "id": "msg_01abc123",
  "type": "message",
  "role": "assistant",
  "content": [
    {"type": "text", "text": "Hello! How can I help?"}
  ],
  "model": "claude-sonnet-4-6",
  "stop_reason": "end_turn",
  "stop_sequence": null,
  "usage": {
    "input_tokens": 10,
    "output_tokens": 15,
    "cache_creation_input_tokens": 0,
    "cache_read_input_tokens": 0
  }
}
```

### Streaming Response (SSE)
```
event: message_start
data: {"type":"message_start","message":{"id":"...","type":"message","role":"assistant","content":[],"model":"...","stop_reason":null,"usage":{"input_tokens":10,"output_tokens":0}}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"input_tokens":10,"output_tokens":15}}

event: message_stop
data: {"type":"message_stop"}
```

### Tool Use Format (in response)
```json
{
  "type": "tool_use",
  "id": "toolu_01abc",
  "name": "get_weather",
  "input": {"city": "London"}
}
```

### Tool Result Format (in next request)
```json
{
  "role": "user",
  "content": [
    {
      "type": "tool_result",
      "tool_use_id": "toolu_01abc",
      "content": [{"type": "text", "text": "The weather is sunny"}],
      "is_error": false
    }
  ]
}
```

### Key Characteristics
- System prompt is a **top-level field** (not in messages)
- Content blocks are **typed** (`text`, `tool_use`, `tool_result`, `image`, etc.)
- Tool use has **unique IDs** prefixed with `toolu_`
- Streaming uses **Anthropic-specific SSE event types**
- Has **prompt caching** with `cache_control`
- Has **thinking/reasoning** blocks

---

## 2. OpenAI Chat Completions API (OpenAI-Compatible Client)

### Endpoint
- **Base URL:** `https://api.openai.com/v1`
- **Endpoint:** `POST /chat/completions`

### Authentication
- **Header:** `Authorization: Bearer <API_KEY>`
- **Content-Type:** `application/json`
- **Env vars:** `OPENAI_API_KEY`, `OPENAI_BASE_URL`

### Request Format
```json
{
  "model": "gpt-4o",
  "max_tokens": 4096,
  "messages": [
    {"role": "system", "content": "You are helpful."},
    {"role": "user", "content": "Hello"}
  ],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get weather",
        "parameters": {
          "type": "object",
          "properties": {"city": {"type": "string"}}
        }
      }
    }
  ],
  "tool_choice": "auto",
  "stream": false
}
```

### Response Format
```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "gpt-4o",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello!",
        "tool_calls": []
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 5,
    "total_tokens": 15
  }
}
```

### Streaming Response (SSE)
```
data: {"id":"chatcmpl-abc","object":"chat.completion.chunk","created":123,"model":"gpt-4o","choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":null}]}

data: {"id":"chatcmpl-abc","object":"chat.completion.chunk","created":123,"model":"gpt-4o","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-abc","object":"chat.completion.chunk","created":123,"model":"gpt-4o","choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"usage":{"prompt_tokens":10,"completion_tokens":5}}

data: [DONE]
```

### Tool Use Format (in response)
```json
{
  "tool_calls": [
    {
      "id": "call_abc123",
      "type": "function",
      "function": {
        "name": "get_weather",
        "arguments": "{\"city\": \"London\"}"
      }
    }
  ]
}
```

### Tool Result Format
```json
{
  "role": "tool",
  "tool_call_id": "call_abc123",
  "content": "The weather is sunny"
}
```

### Key Characteristics
- System prompt is a **message with role "system"**
- Tools are wrapped in `{"type": "function", "function": {...}}`
- Tool calls have **string arguments** (not parsed JSON objects)
- Tool results use **role "tool"** with `tool_call_id`
- Streaming has **delta** format with `choices[].delta`
- Finish reasons: `stop`, `tool_calls`, `length`, `content_filter`
- Uses `stream_options: {"include_usage": true}` for streaming usage

---

## 3. OpenRouter API (Target)

### Endpoint
- **Base URL:** `https://openrouter.ai/api/v1`
- **Chat Endpoint:** `POST /chat/completions`
- **Models Endpoint:** `GET /models`

### Authentication
- **Header:** `Authorization: Bearer <OPENROUTER_API_KEY>`
- **Optional Headers:**
  - `HTTP-Referer: <your-site-url>` (for leaderboard attribution)
  - `X-OpenRouter-Title: <your-app-name>` (for leaderboard attribution)
- **Content-Type:** `application/json`
- **Env vars:** `OPENROUTER_API_KEY`, `OPENROUTER_BASE_URL` (custom)

### Model IDs
Model IDs use **provider/model** format:
- `anthropic/claude-sonnet-4.6`
- `anthropic/claude-opus-4-6`
- `openai/gpt-5.2`
- `google/gemini-3.1-pro-preview`
- `qwen/qwen3.6-plus:free`
- `x-ai/grok-4.20`
- `mistralai/mistral-small-2603`
- `openrouter/free` (free tier)

### Request Format
**OpenRouter uses the OpenAI Chat Completions API format** — it's a drop-in proxy:
```json
{
  "model": "anthropic/claude-sonnet-4.6",
  "max_tokens": 64000,
  "messages": [
    {"role": "system", "content": "You are helpful."},
    {"role": "user", "content": "Hello"}
  ],
  "tools": [...],
  "tool_choice": "auto",
  "stream": false,
  "temperature": 1.0
}
```

### Response Format
**Same as OpenAI** — `ChatCompletionResponse`:
```json
{
  "id": "gen-abc123",
  "model": "anthropic/claude-sonnet-4.6",
  "choices": [
    {
      "message": {
        "role": "assistant",
        "content": "Hello!",
        "tool_calls": []
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 15,
    "total_tokens": 25
  }
}
```

### Streaming
**Same as OpenAI** — SSE with `data:` prefix:
```
data: {"id":"gen-abc","object":"chat.completion.chunk","model":"anthropic/claude-sonnet-4.6","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"gen-abc","object":"chat.completion.chunk","model":"anthropic/claude-sonnet-4.6","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

### Supported Parameters (OpenRouter-specific)
From the `/v1/models` endpoint, models support various parameters:
- `tools` / `tool_choice` — tool calling (model-dependent)
- `parallel_tool_calls` — parallel tool execution
- `include_reasoning` / `reasoning` — thinking tokens
- `structured_outputs` — JSON mode
- `min_p` — sampling parameter
- `verbosity` — output verbosity
- `web_search`, `input_cache_read`, `input_cache_write` — pricing features
- `provider` — force specific provider
- `models` — model fallback list

### Error Format
```json
{
  "error": {
    "code": 402,
    "message": "Insufficient credits",
    "metadata": {
      "request_id": "req_abc123"
    }
  }
}
```

### Key Characteristics
- **OpenAI-compatible format** — same request/response schema
- **Bearer token auth** (like OpenAI, NOT like Anthropic)
- **Model IDs include provider prefix** (`anthropic/`, `openai/`, etc.)
- **Single endpoint** for all models — smart routing handles the rest
- **Automatic provider fallback** if primary provider fails
- **No prompt caching** (OpenRouter doesn't support Anthropic's cache_control)
- **No thinking blocks** in response (normalized to text)
- Supports **hundreds of models** from dozens of providers
- Has **free tier** models (`:free` suffix)

---

## 4. API Mapping: What Needs to Change

### OpenRouter → OpenAI-Compatible (Minimal Changes)

OpenRouter uses the **exact same format as OpenAI's Chat Completions API**. This means the existing `OpenAiCompatClient` can be reused with minimal changes:

| Aspect | Current OpenAiCompatClient | OpenRouter | Change Needed |
|--------|--------------------------|------------|---------------|
| Auth | `Bearer <key>` | `Bearer <key>` | **None** — same |
| Endpoint | `/chat/completions` | `/chat/completions` | **None** — same |
| Base URL | `api.openai.com/v1` | `openrouter.ai/api/v1` | New default |
| Request body | OpenAI format | OpenAI format | **None** — same |
| Response | OpenAI format | OpenAI format | **None** — same |
| Streaming | OpenAI SSE | OpenAI SSE | **None** — same |
| Tool calls | OpenAI format | OpenAI format | **None** — same |
| Tool results | `role: "tool"` | `role: "tool"` | **None** — same |
| System prompt | `role: "system"` message | `role: "system"` message | **None** — same |
| Optional headers | None | `HTTP-Referer`, `X-OpenRouter-Title` | **Add optional headers** |
| Env vars | `OPENAI_API_KEY` | `OPENROUTER_API_KEY` | **New env var** |

### The One Critical Difference

The `OpenAiCompatClient` translates **from Anthropic's internal format** (`MessageRequest`) to **OpenAI's format**. This translation already exists and works perfectly for OpenRouter:

```rust
// Existing translation in openai_compat.rs:
// 1. System prompt → "system" role message  ✓
// 2. Anthropic content blocks → OpenAI messages  ✓
// 3. Anthropic tool format → OpenAI tool format  ✓
// 4. OpenAI response → Anthropic-style MessageResponse  ✓
```

**This means OpenRouter is essentially a config change on top of the existing OpenAI-compatible client.**

### What OpenRouter Adds

1. **New `ProviderKind::OpenRouter`** — distinct from OpenAI and XAI
2. **New env var:** `OPENROUTER_API_KEY` + `OPENROUTER_BASE_URL`
3. **New default base URL:** `https://openrouter.ai/api/v1`
4. **Optional attribution headers:** `HTTP-Referer`, `X-OpenRouter-Title`
5. **Model aliases** for popular OpenRouter models (e.g., `sonnet` → `anthropic/claude-sonnet-4.6`)
6. **Model registry entries** with provider prefixes

### OpenRouter-Specific Model Registry

```
"or-sonnet"    → "anthropic/claude-sonnet-4-6"
"or-opus"      → "anthropic/claude-opus-4-6"
"or-haiku"     → "anthropic/claude-haiku-3-5-20241022"
"or-gpt-4o"    → "openai/gpt-4o"
"or-gpt-5"     → "openai/gpt-5"
"or-gemini"    → "google/gemini-2.5-pro"
"or-grok"      → "x-ai/grok-3"
"or-qwen"      → "qwen/qwen3-235b-a22b"
"or-free"      → "openrouter/free"
```

Also: raw model IDs like `anthropic/claude-sonnet-4-6` should auto-detect as OpenRouter.

---

## 5. Implementation Plan

### Step 1: Add `OpenRouter` to `ProviderKind`
```rust
pub enum ProviderKind {
    Anthropic,
    Xai,
    OpenAi,
    OpenRouter,  // NEW
}
```

### Step 2: Create OpenRouter Config
Reuse `OpenAiCompatConfig` pattern:
```rust
impl OpenAiCompatConfig {
    pub const fn openrouter() -> Self {
        Self {
            provider_name: "OpenRouter",
            api_key_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: "https://openrouter.ai/api/v1",
        }
    }
}
```

### Step 3: Add OpenRouter Model Registry
Add entries to `MODEL_REGISTRY` with provider prefix mapping.

### Step 4: Update Provider Detection
`detect_provider_kind()` should check:
1. Model name/alias first
2. If model starts with `anthropic/`, `openai/`, `google/`, `qwen/`, `x-ai/`, `mistralai/` → OpenRouter
3. Fall back to env var detection (check `OPENROUTER_API_KEY`)

### Step 5: Add `ProviderClient::OpenRouter` variant
```rust
pub enum ProviderClient {
    Anthropic(AnthropicClient),
    Xai(OpenAiCompatClient),
    OpenAi(OpenAiCompatClient),
    OpenRouter(OpenAiCompatClient),  // NEW
}
```

### Step 6: Add CLI Model Flag Support
Update CLI to accept `--model or-sonnet` etc. and resolve to full IDs.

### Step 7: Optional Attribution Headers
Add optional `HTTP-Referer` and `X-OpenRouter-Title` headers to the request builder.

### Step 8: Model Token Limits
Add `model_token_limit` entries for OpenRouter models.

---

## 6. What OpenRouter Does NOT Support (vs Anthropic Direct)

| Feature | Anthropic Direct | OpenRouter |
|---------|-----------------|------------|
| Prompt caching (`cache_control`) | Yes | No |
| Thinking/reasoning blocks | Yes | Depends on model |
| Token counting endpoint | Yes (`/count_tokens`) | No |
| Beta features | Yes | No |
| Server tools (web_fetch, code_execution) | Yes | No |
| Eager input streaming | Yes | Depends |
| Multi-modal (images, documents) | Yes | Model-dependent |
| OAuth flow | Yes | No (API key only) |

**Impact:** When using OpenRouter, prompt caching and OAuth are unavailable. The CLI should handle this gracefully.

---

## 7. Summary

OpenRouter is the **easiest possible integration** because:
1. It uses the **OpenAI Chat Completions API format** (already fully implemented)
2. It uses **Bearer token auth** (already implemented)
3. The existing `OpenAiCompatClient` does **all the format translation** already
4. The only new code is: provider enum variant, env vars, model aliases, base URL

**Estimated changes:** ~100-150 lines across existing files, no new modules needed.
