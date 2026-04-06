# Crate Study Guides

> Deep-dive notes on each crate in the `rust/` workspace, organized by dependency layer.

---

## Layer 1: `telemetry`

**Path:** `rust/crates/telemetry/`
**Lines:** ~1 file
**Dependencies:** None (internal)

### Purpose
Observability and tracing layer for the claw CLI. Captures HTTP request lifecycle events, analytics, and session traces.

### Key Types

- **`TelemetryEvent`** — Enum covering all event types:
  - HTTP request start/complete/failed
  - Analytics events
  - Session trace records

- **`TelemetrySink`** — Trait for consuming events:
  - `MemoryTelemetrySink` — In-memory buffer (testing)
  - `JsonlTelemetrySink` — Persistent JSONL file output

- **`SessionTracer`** — Per-session trace recorder with start/end timing

- **`ClientIdentity`** — Anthropic client identification (headers, betas, version)

- **`AnthropicRequestProfile`** — Request configuration: betas, extra body, headers

### Why It Matters
This crate decouples observability from the runtime. Any crate can emit `TelemetryEvent`s without knowing where they go. The `TelemetrySink` trait enables swapping storage backends without changing emission sites.

---

## Layer 1: `plugins`

**Path:** `rust/crates/plugins/`
**Lines:** ~3,460 (lib.rs) + hooks.rs
**Dependencies:** None (internal)

### Purpose
Plugin system with installable, enableable, disableable, and uninstallable plugins. Supports builtin, bundled, and external plugins.

### Key Types

- **`BuiltinPlugin`** — Compiled-in plugins
- **`BundledPlugin`** — Shipped with distribution
- **`ExternalPlugin`** — User-installed from external sources
- **`PluginRegistry`** — Registry of all loaded plugins
- **`PluginManager`** — Lifecycle manager (install/uninstall/enable/disable)
- **`PluginTool`** — Tool definition exposed by a plugin
- **`PluginHooks`** — `PreToolUse`, `PostToolUse`, `PostToolUseFailure` hooks

### Hook System (`hooks.rs`)
Hooks execute shell commands with:
- Environment variable injection
- Stdin payload delivery
- Progress reporting

### Why It Matters
Plugins extend the tool surface without modifying core code. The hook system enables pre/post-processing of tool execution (logging, validation, augmentation).

---

## Layer 2: `runtime`

**Path:** `rust/crates/runtime/`
**Lines:** 39 modules, largest crate
**Dependencies:** `plugins`, `telemetry`

### Purpose
The central nervous system. Owns session persistence, permission evaluation, prompt assembly, MCP plumbing, file operations, conversation loop, sandbox detection, OAuth, policy evaluation, and worker/task registries.

### Module Map

#### Session & Conversation
| Module | Key Types | Purpose |
|--------|-----------|---------|
| `session.rs` | `Session`, `ConversationMessage`, `ContentBlock`, `MessageRole` | Persistent conversation state with rotation (256KB chunks, max 3 files) |
| `conversation.rs` | `ConversationRuntime<C, T>`, `ApiClient`, `ToolExecutor`, `AssistantEvent`, `RuntimeError`, `ToolError` | Generic conversation loop with streaming |
| `compact.rs` | `CompactionConfig`, `CompactionResult` | Session compaction with token estimation and threshold logic |

#### I/O Operations
| Module | Key Types | Purpose |
|--------|-----------|---------|
| `bash.rs` | `BashCommandInput`, `BashCommandOutput`, `execute_bash` | Command execution with timeout/background/sandbox |
| `bash_validation.rs` | Validation logic | Read-only gating, destructive command warnings, path validation |
| `file_ops.rs` | `read_file`, `write_file`, `edit_file`, `GlobSearchOutput`, `GrepSearchOutput` | File operations with binary detection, size limits, workspace boundary checks |

#### Permissions & Security
| Module | Key Types | Purpose |
|--------|-----------|---------|
| `permissions.rs` | `PermissionMode`, `PermissionPolicy`, `PermissionOutcome`, `PermissionPrompter` | Permission modes (read-only/workspace-write/danger), policy evaluation |
| `permission_enforcer.rs` | `PermissionEnforcer` | Tool-level permission checks, file write boundary enforcement, bash read-only heuristics |
| `sandbox.rs` | `SandboxStatus`, `ContainerEnvironment`, `LinuxSandboxCommand` | Container detection, filesystem isolation, sandbox command building |

#### MCP (Model Context Protocol)
| Module | Key Types | Purpose |
|--------|-----------|---------|
| `mcp.rs` | Server signatures, name normalization, config hashing | MCP server identity and configuration |
| `mcp_client.rs` | `McpClientBootstrap`, transports (stdio/remote/sdk/managed-proxy) | MCP client lifecycle |
| `mcp_stdio.rs` | `McpServerManager`, JSON-RPC types, tool/resource discovery | Stdio-based MCP process management |
| `mcp_tool_bridge.rs` | `McpToolRegistry` | Bridge between MCP servers and tool dispatch |
| `mcp_lifecycle_hardened.rs` | `McpLifecycleValidator`, `McpDegradedReport`, `McpFailedServer` | Structured degraded-startup reporting |

#### Worker & Task Management
| Module | Key Types | Purpose |
|--------|-----------|---------|
| `worker_boot.rs` | `WorkerRegistry`, `WorkerStatus`, `WorkerEvent`, `WorkerFailure` | Worker state machine for reliable boot |
| `task_registry.rs` | `TaskRegistry` | In-memory task lifecycle (create/get/list/stop/update/output) |
| `team_cron_registry.rs` | `TeamRegistry`, `CronRegistry` | Team and cron job management |
| `task_packet.rs` | `TaskPacket`, `TaskScope` | Structured task format with validation |

#### Policy & Events
| Module | Key Types | Purpose |
|--------|-----------|---------|
| `policy_engine.rs` | `PolicyEngine`, `PolicyRule`, `LaneContext`, `GreenLevel` | Policy evaluation for merge/recovery/escalation |
| `lane_events.rs` | `LaneEvent`, `LaneEventName`, `LaneCommitProvenance` | Typed event schema for lane lifecycle |
| `recovery_recipes.rs` | `RecoveryRecipe`, `FailureScenario`, `RecoveryStep` | Automatic recovery for known failures |
| `stale_branch.rs` | `BranchFreshness`, `StaleBranchPolicy` | Branch freshness detection and policy |
| `green_contract.rs` | Green level contract | Test pass/fail contract definition |
| `summary_compression.rs` | Text compression | Summary text compression for status |

#### Infrastructure
| Module | Key Types | Purpose |
|--------|-----------|---------|
| `config.rs` | `RuntimeConfig`, `ConfigLoader`, `McpServerConfig`, `OAuthConfig` | Configuration loading with precedence |
| `prompt.rs` | `SystemPromptBuilder`, `ContextFile`, `ProjectContext` | System prompt assembly |
| `oauth.rs` | PKCE flow, token exchange, credential persistence | OAuth authentication |
| `hooks.rs` | `HookRunner`, `HookEvent`, `HookAbortSignal` | Pre/post tool-use hook execution |
| `remote.rs` | `RemoteSessionContext`, `UpstreamProxyBootstrap` | Remote session handling |
| `sse.rs` | `IncrementalSseParser`, `SseEvent` | Server-sent events parsing |
| `usage.rs` | `UsageTracker`, `TokenUsage`, `ModelPricing` | Token counting and cost estimation |
| `bootstrap.rs` | `BootstrapPhase`, `BootstrapPlan` | Bootstrap phase definitions |
| `json.rs` | JSON utilities | Internal JSON parsing helpers |

#### Test-Only Modules
| Module | Purpose |
|--------|---------|
| `session_control.rs` | Session control tests |
| `trust_resolver.rs` | Trust resolution tests |

### Why It Matters
This is the heart of the system. Almost every other crate depends on types defined here. The `ConversationRuntime` is the central orchestrator, and every module feeds into how it behaves.

---

## Layer 3: `api`

**Path:** `rust/crates/api/`
**Lines:** 9 modules
**Dependencies:** `runtime`, `telemetry`

### Purpose
HTTP client layer for LLM providers. Abstracts away provider-specific details behind a common interface.

### Key Modules

| Module | Purpose |
|--------|---------|
| `client.rs` | `ProviderClient` trait, OAuth token handling, message streams, auth source resolution |
| `types.rs` | API types: `MessageRequest`, `MessageResponse`, `InputMessage`, `OutputContentBlock`, `ToolDefinition`, `StreamEvent` |
| `providers/mod.rs` | `Provider` trait, `ProviderKind` (Anthropic/Xai/OpenAi), model alias resolution, context window metadata |
| `providers/anthropic.rs` | Anthropic API client implementation |
| `providers/openai_compat.rs` | OpenAI-compatible client for XAI/Grok and generic OpenAI |
| `prompt_cache.rs` | Prompt cache with config, paths, records, stats, break events |
| `sse.rs` | SSE frame parsing for streaming responses |
| `error.rs` | `ApiError` type |

### Key Abstractions

```rust
// Provider-agnostic client
pub trait ProviderClient {
    async fn create_message(&self, request: MessageRequest) -> Result<MessageResponse, ApiError>;
    async fn stream_message(&self, request: MessageRequest) -> Result<MessageStream, ApiError>;
}

// Provider detection
pub fn detect_provider_kind(base_url: &str) -> ProviderKind;

// Model alias resolution
pub fn resolve_model_alias(alias: &str) -> &str;
```

### Why It Matters
Swapping providers (Anthropic → XAI → OpenAI-compatible) requires zero changes to the runtime or tools. The conversation loop talks to `ApiClient` trait, not a specific provider.

---

## Layer 3: `commands`

**Path:** `rust/crates/commands/`
**Lines:** ~7,900 (single file)
**Dependencies:** `plugins`, `runtime`

### Purpose
Defines the full slash-command surface (~100+ commands) and their routing/dispatch logic.

### Key Types

| Type | Purpose |
|------|---------|
| `SlashCommandSpec` | Command metadata: name, aliases, summary, argument hint, resume support |
| `CommandManifestEntry` | Registry entry with source (Builtin/InternalOnly/FeatureGated) |
| `CommandRegistry` | Collection of command manifests |
| `SkillSlashDispatch` | Skill command routing: `Local` or `Invoke(String)` |

### Slash Commands (examples)
- `/help` — Show available commands
- `/status` — Show session status
- `/compact` — Compact session history
- `/mcp` — Inspect MCP servers
- `/plugins` — Manage plugins
- `/skills` — Show available skills
- `/agents` — Show agent status
- `/cost` — Show token usage
- `/config` — Inspect config
- `/model` — Switch model
- `/permissions` — Switch permission mode
- `/clear` — Start fresh session
- `/resume` — Load saved session
- `/init` — Create CLAUDE.md
- `/export` — Export session
- `/diff` — Show session diff
- `/doctor` — Run diagnostics

### Why It Matters
This crate defines the user-facing command surface. The `resume_supported` flag on each spec determines whether the command works in resumed sessions.

---

## Layer 4: `tools`

**Path:** `rust/crates/tools/`
**Lines:** ~5,311 (lib.rs) + lane_completion.rs
**Dependencies:** `api`, `commands`, `plugins`, `runtime`

### Purpose
Tool registry and execution surface. Defines all tools the model can call and routes them to implementations.

### Built-in Tools (40 specs)

**Core Execution:**
- `bash` — Shell command execution
- `read_file` — File reading
- `write_file` — File writing
- `edit_file` — File editing (structured patches)
- `glob_search` — Glob pattern file search
- `grep_search` — Content search

**Web:**
- `WebFetch` — URL content fetching
- `WebSearch` — Web search

**Productivity:**
- `TodoWrite` — Task list management
- `Skill` — Skill invocation
- `Agent` — Agent spawning
- `ToolSearch` — Tool discovery search
- `NotebookEdit` — Jupyter notebook editing
- `Sleep` — Delay execution
- `SendUserMessage` — User messaging

**System:**
- `Config` — Config inspection
- `EnterPlanMode` — Plan mode entry
- `ExitPlanMode` — Plan mode exit
- `StructuredOutput` — Structured response formatting
- `REPL` — REPL interaction
- `PowerShell` — PowerShell execution

**Registry-Backed (no longer stubs):**
- `Task*` — Task lifecycle (create/get/list/stop/update/output)
- `Team*` — Team lifecycle (create/delete)
- `Cron*` — Cron lifecycle (create/delete/list)
- `LSP` — Language server protocol (symbols/references/diagnostics/definition/hover)
- `MCP` — Model Context Protocol (resources/auth/tools)

### Key Types

| Type | Purpose |
|------|---------|
| `ToolSpec` | Tool definition: name, description, input schema, required permission |
| `GlobalToolRegistry` | Central registry with plugin tools, runtime tools, and permission enforcer |
| `RuntimeToolDefinition` | Runtime-added tool with permission requirements |
| `ToolManifestEntry` | Registry entry with source (Base/Conditional) |

### Lane Completion (`lane_completion.rs`)
Auto-detects when a lane is complete:
- Session finished + tests green + push complete → policy closeout

### Why It Matters
This is where the model's tool calls become action. The registry pattern means tools can be added/removed without recompiling, and permission enforcement happens at dispatch time.

---

## Layer 5: `compat-harness`

**Path:** `rust/crates/compat-harness/`
**Lines:** 1 file
**Dependencies:** `commands`, `tools`, `runtime`

### Purpose
Parses upstream TypeScript source to extract command/tool manifests for parity verification. Ensures the Rust port matches the original feature surface.

### How It Works
1. Reads `src/commands.ts`, `src/tools.ts`, `src/entrypoints/cli.tsx`
2. Parses import statements
3. Detects feature-gated entries
4. Identifies internal-only blocks
5. Extracts bootstrap phase plan
6. Compares against Rust implementations

### Why It Matters
This is automated honesty. It proves (or disproves) that the Rust port covers the same surface as the upstream. Without this, parity claims would be manual and error-prone.

---

## Layer 5: `mock-anthropic-service`

**Path:** `rust/crates/mock-anthropic-service/`
**Lines:** lib.rs + main.rs
**Dependencies:** `api`

### Purpose
Deterministic HTTP server simulating the Anthropic API for testing without real API calls.

### Test Scenarios (12)
1. `streaming_text` — Streaming text response
2. `read_file_roundtrip` — File read operation
3. `grep_chunk_assembly` — Chunked grep output
4. `write_file_allowed` — Permitted file write
5. `write_file_denied` — Denied file write
6. `multi_tool_turn_roundtrip` — Multi-tool assistant turn
7. `bash_stdout_roundtrip` — Bash execution
8. `bash_permission_prompt_approved` — Approved bash permission
9. `bash_permission_prompt_denied` — Denied bash permission
10. `plugin_tool_roundtrip` — Plugin tool execution
11. `auto_compact` — Automatic session compaction
12. `cost_reporting` — Cost reporting

### Why It Matters
Enables CI testing without API keys or rate limits. Scenarios are deterministic and reproducible. The parity harness runs against this to prove behavioral correctness.

---

## Layer 6: `rusty-claude-cli`

**Path:** `rust/crates/rusty-claude-cli/`
**Lines:** ~9,476 (main.rs) + init.rs + input.rs + render.rs
**Dependencies:** All of the above

### Purpose
The `claw` binary — ties everything together with REPL, argument parsing, rendering, and session management.

### Key Modules

| Module | Purpose |
|--------|---------|
| `main.rs` | CLI entry point: argument parsing, REPL loop, slash command dispatch, OAuth, doctor/init, session resume, prompt execution, tool execution, Markdown rendering, plugin/MCP/skill management |
| `init.rs` | Repository initialization (create CLAUDE.md) |
| `input.rs` | Input handling for REPL |
| `render.rs` | Terminal rendering: Markdown streaming, spinner, `TerminalRenderer` |

### CLI Actions (from `CliAction` enum)
- `Help` / `HelpTopic` — Help display
- `Version` — Version info
- `Doctor` — Diagnostics
- `Init` — Repo initialization
- `Login` / `Logout` — OAuth authentication
- `Prompt` — One-shot prompt execution
- `Repl` — Interactive REPL
- `ResumeSession` — Load saved session
- `Status` / `Sandbox` — Status snapshots
- `Agents` / `Mcp` / `Skills` / `Plugins` — Inventory commands
- `PrintSystemPrompt` — System prompt dump
- `DumpManifests` / `BootstrapPlan` — Debug output

### Output Format Support
All commands support `--output-format json` for machine-readable output, enabling claw-driven automation.

### Why It Matters
This is the user-facing binary. It orchestrates all other crates into a coherent CLI experience. The `--output-format json` flag on every command is what makes the system "clawable" rather than "human-only."

---

## Cross-Cutting Concerns

### Error Handling
- Each crate defines its own error types
- `RuntimeError` and `ToolError` in runtime for conversation-level errors
- `ApiError` in api for HTTP-level errors
- `SessionError` for persistence errors
- Errors are `Clone + PartialEq + Eq` for testability

### Concurrency
- `Arc<Mutex<T>>` for shared mutable state (registries)
- `OnceLock` for lazy singleton initialization
- `test_env_lock()` for test serialization
- No `unsafe` code (forbidden at workspace level)

### Testing Strategy
- Unit tests within each crate
- Integration tests in `rust/crates/rusty-claude-cli/tests/`
- Mock parity harness with 12 scenarios
- Compat-harness against upstream TypeScript
- Workspace-parallel test execution with isolated temp resources
