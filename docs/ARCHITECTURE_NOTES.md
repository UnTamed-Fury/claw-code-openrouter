# Architecture Notes — Claw Code

> These notes were created by studying the codebase to understand its structure, design decisions, and operational philosophy.

## What Is Claw Code?

Claw Code is a **Rust CLI agent harness** — a port/reimplementation of the Claude Code CLI, built to be "clawable": machine-controllable, event-native, and designed for autonomous coding agents (called "claws") rather than human-first terminal interaction.

The canonical implementation lives in `rust/`, with a companion Python/reference workspace in `src/` and `tests/`.

---

## Layered Architecture

The codebase is organized as a **9-crate Rust workspace** with clean dependency layering:

```
Layer 6:  rusty-claude-cli (binary: `claw`)
           ↓
Layer 5:  compat-harness    mock-anthropic-service
           ↓                      ↓
Layer 4:  tools ──────────────────┘
           ↓
Layer 3:  api          commands
           ↓              ↓
Layer 2:  runtime ←───────┘
           ↓
Layer 1:  plugins    telemetry
```

### Bottom-Up Crate Breakdown

#### Layer 1: Foundation Crates

**`telemetry`** — Observability layer
- `TelemetryEvent` enum: HTTP request lifecycle, analytics, traces
- `TelemetrySink` trait with `MemoryTelemetrySink` and `JsonlTelemetrySink`
- `SessionTracer` for per-session trace records
- Client identity, Anthropic request profiles (betas, extra body, headers)
- **No internal dependencies** — pure leaf crate

**`plugins`** — Plugin system (~3460 lines in `lib.rs` + `hooks.rs`)
- Plugin trait hierarchy: `BuiltinPlugin`, `BundledPlugin`, `ExternalPlugin`
- `PluginRegistry` and `PluginManager` for lifecycle management
- Hook runner: `PreToolUse`, `PostToolUse`, `PostToolUseFailure`
- Plugin install/update/uninstall operations
- **No internal dependencies** — pure leaf crate

#### Layer 2: Core Runtime

**`runtime`** — The largest and most central crate (~39 modules)

This crate owns:
- **Session management** (`session.rs`): `Session`, `ConversationMessage`, `ContentBlock`, persistence with rotation (256KB chunks, max 3 rotated files)
- **Conversation loop** (`conversation.rs`): `ConversationRuntime<C, T>` coordinates the model loop, tool execution, hooks, and session updates via generic `ApiClient` and `ToolExecutor` traits
- **Bash execution** (`bash.rs`, `bash_validation.rs`): Command execution with timeout/background/sandbox support
- **File operations** (`file_ops.rs`): `read_file`, `write_file`, `edit_file`, `glob_search`, `grep_search` with binary detection, size limits, workspace boundary validation
- **Permission system** (`permissions.rs`, `permission_enforcer.rs`): `PermissionMode` (read-only/workspace-write/danger-full-access), tool gating, file write boundary checks, bash read-only heuristics
- **MCP protocol** (`mcp.rs`, `mcp_client.rs`, `mcp_stdio.rs`, `mcp_tool_bridge.rs`): Full MCP lifecycle — stdio spawn, JSON-RPC, tool/resource discovery, server manager, degraded-mode reporting
- **OAuth** (`oauth.rs`): PKCE flow, token exchange, credential persistence
- **Sandbox detection** (`sandbox.rs`): Container environment detection, filesystem isolation, Linux sandbox commands
- **Policy engine** (`policy_engine.rs`): `PolicyRule`, `PolicyCondition`, `PolicyAction`, `LaneContext` — governs merge/recovery/escalation decisions
- **Worker boot** (`worker_boot.rs`): State machine `Spawning → TrustRequired → ReadyForPrompt → Running → Finished/Failed`, trust-gate detection, prompt-misdelivery recovery
- **Lane events** (`lane_events.rs`): Typed `LaneEvent` enum (`lane.started`, `lane.green`, `lane.failed`, etc.), commit provenance tracking
- **Recovery recipes** (`recovery_recipes.rs`): Automatic recovery for 7 failure scenarios (trust prompts, misdelivery, stale branches, compile errors, MCP handshake, plugin startup, provider failures)
- **Stale branch detection** (`stale_branch.rs`): Branch freshness checking and policy application
- **Task/Team/Cron registries**: In-memory lifecycle management
- **LSP client** (`lsp_client.rs`): Diagnostics, hover, definition, references, completion, symbols, formatting
- **Config loading** (`config.rs`): MCP server configs, OAuth, permissions, hooks, plugins, feature flags
- **System prompts** (`prompt.rs`): Prompt building with context files, dynamic boundary markers

#### Layer 3: API + Commands

**`api`** — HTTP client layer
- `ProviderClient` trait with implementations for:
  - Anthropic (`AnthropicClient`)
  - OpenAI-compatible (`OpenAiCompatClient` for XAI/Grok and generic OpenAI)
- Model alias resolution, context window metadata
- Prompt caching with `PromptCache`
- Request preflight validation
- Streaming response parsing (SSE)
- Depends on: `runtime`, `telemetry`

**`commands`** — Slash command surface (~7900 lines)
- 100+ slash commands: `/help`, `/status`, `/compact`, `/mcp`, `/plugins`, `/skills`, `/agents`, etc.
- Command dispatch and routing
- Plugin/skill/agent command handlers
- Depends on: `plugins`, `runtime`

#### Layer 4: Tools

**`tools`** — Tool registry (~5311 lines in `lib.rs` + `lane_completion.rs`)
- **40 built-in tool specs**: bash, read_file, write_file, edit_file, glob_search, grep_search, WebFetch, WebSearch, TodoWrite, Skill, Agent, ToolSearch, NotebookEdit, Sleep, SendUserMessage, Config, EnterPlanMode, ExitPlanMode, StructuredOutput, REPL, PowerShell, and more
- `GlobalToolRegistry` with plugin tool integration and permission enforcement
- Lane completion detection (`lane_completion.rs`): auto-marks lanes complete when tests are green + code pushed + no blockers
- Task packet format with `TaskScope` resolution (workspace/module/single-file/custom)
- Depends on: `api`, `commands`, `plugins`, `runtime`

#### Layer 5: Testing + Compatibility

**`compat-harness`** — Upstream compatibility verification
- Parses upstream TypeScript source (`src/commands.ts`, `src/tools.ts`, `src/entrypoints/cli.tsx`)
- Extracts command/tool/bootstrap manifests for parity checking
- Feature-gated entries and internal-only block detection
- Depends on: `commands`, `tools`, `runtime`

**`mock-anthropic-service`** — Deterministic test harness
- Mock HTTP server simulating Anthropic API
- 12 parity test scenarios: streaming text, read/write file, grep, multi-tool, bash, plugin, auto-compact, cost reporting
- Supports both streaming (SSE) and non-streaming responses
- Depends on: `api`

#### Layer 6: CLI Binary

**`rusty-claude-cli`** — The `claw` binary (~9476 lines in `main.rs` + 3 modules)
- CLI argument parsing with `CliAction` enum
- REPL loop with `rustyline`
- Slash command dispatch
- OAuth login/logout flow
- Doctor/init commands
- Session resume
- Prompt execution with model/provider resolution
- Tool execution loop
- Markdown rendering with spinner (`render.rs`)
- Plugin/MCP/skill management
- Depends on: everything above

---

## Key Design Patterns

### 1. Trait-Based Abstraction

The conversation runtime is generic over its dependencies:

```rust
pub struct ConversationRuntime<C, T> {
    session: Session,
    api_client: C,      // trait: ApiClient
    tool_executor: T,   // trait: ToolExecutor
    // ...
}
```

This enables:
- Testing with mock clients/ executors
- Swapping providers without changing runtime logic
- Clean separation of concerns

### 2. Global Registries via `OnceLock`

Tool registries are lazily initialized once per process:

```rust
fn global_tool_registry() -> &'static GlobalToolRegistry {
    static REGISTRY: OnceLock<GlobalToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(GlobalToolRegistry::new)
}
```

Same pattern for: LSP registry, MCP registry, team registry, cron registry, task registry, worker registry.

### 3. Session Persistence with Rotation

Sessions are persisted as JSONL files with automatic rotation:
- Rotate after 256KB
- Keep max 3 rotated files
- Located in `.claw/sessions/`
- Support compaction, forking, and resumption

### 4. Permission Enforcement Pipeline

Every tool has a `required_permission` field. The `PermissionEnforcer` checks:
- Tool-level permissions
- File write boundaries (workspace containment)
- Bash read-only heuristics
- User overrides via `--permission-mode` or `--allowedTools`

### 5. Worker State Machine

Workers go through explicit lifecycle states:

```
Spawning → TrustRequired → ReadyForPrompt → Running → Finished/Failed
```

Each transition emits a typed `WorkerEvent`. This enables:
- Deterministic startup handshakes
- Trust prompt auto-resolution for allowlisted repos
- Prompt misdelivery detection and recovery
- Machine-readable status (not scraped terminal text)

### 6. Lane Event Schema

Typed events flow through the system:

```
lane.started → lane.ready → lane.green → lane.commit.created → lane.merge.ready → lane.finished
                                      ↓
                              lane.blocked / lane.red / lane.failed
```

Events carry structured data: `LaneFailureClass`, `LaneCommitProvenance`, `LaneEventBlocker`.

### 7. Recovery Recipes

7 failure scenarios have automatic recovery:
1. Trust prompt unresolved → auto-accept for allowlisted repos
2. Prompt misdelivery → redirect to agent
3. Stale branch → rebase
4. Compile red cross-crate → clean build
5. MCP handshake failure → retry with timeout
6. Partial plugin startup → restart plugin
7. Provider failure → escalate to human

Each recipe has max attempts and escalation policy.

---

## The Conversation Loop Flow

```
1. Load session (new or resumed)
2. Build system prompt (context files, dynamic boundary)
3. Create ConversationRuntime with:
   - ApiClient (Anthropic/OpenAI-compatible)
   - ToolExecutor (GlobalToolRegistry)
   - PermissionPolicy
   - HookRunner
4. Send user message to session
5. Loop (up to max_iterations):
   a. Stream assistant response
   b. For each TextDelta → render to terminal
   c. For each ToolUse → execute tool, append result
   d. Check auto-compaction threshold
   e. If MessageStop → break
6. Persist session
7. Return TurnSummary
```

---

## Config Resolution Order

```
1. ~/.claw.json
2. ~/.config/claw/settings.json
3. <repo>/.claw.json
4. <repo>/.claw/settings.json
5. <repo>/.claw/settings.local.json
```

Later entries override earlier ones. Config covers: MCP servers, OAuth, permissions, hooks, plugins, feature flags.

---

## Model Support

| CLI Alias | Actual Model |
|-----------|-------------|
| `opus` | `claude-opus-4-6` |
| `sonnet` | `claude-sonnet-4-5-20251213` (via alias resolution) |
| `haiku` | `claude-haiku-4-5-20251213` |

Providers:
- Anthropic (primary)
- XAI/Grok (via OpenAI-compatible client)
- Generic OpenAI-compatible (via `ANTHROPIC_BASE_URL`)

---

## What Makes This "Clawable"

The ROADMAP defines "clawable" as:
- **Deterministic to start** — explicit worker states, not guessing
- **Machine-readable failures** — `LaneFailureClass` enum, not scraped text
- **Recoverable without human** — 7 automatic recovery recipes
- **Branch/test aware** — stale branch detection before blaming tests
- **Plugin/MCP lifecycle aware** — degraded-mode reporting per server
- **Event-first** — typed `LaneEvent` schema, not log prose
- **Autonomous** — policy engine drives next steps without babysitting

---

## Repository Statistics (at checkpoint `ee31e00`)

- 292 commits on main / 293 across all branches
- 9 crates in workspace
- 48,599 tracked Rust LOC
- 2,568 test LOC
- 3 authors
- Date range: 2026-03-31 → 2026-04-03

---

## Philosophy Summary

From `PHILOSOPHY.md`:

> "Claw Code is not just a codebase. It is a public demonstration of what happens when a human provides clear direction, multiple coding agents coordinate in parallel, notification routing is pushed out of the agent context window, planning/execution/review loops are automated, and the human does not sit in a terminal micromanaging every step."

The product is not the code — it's the **coordination system** that produced the code. The repository is the artifact; the philosophy is the system behind it.
