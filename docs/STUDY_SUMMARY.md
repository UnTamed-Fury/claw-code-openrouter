# Study Summary — Claw Code

> Key takeaways, study recommendations, and areas worth deeper investigation.

---

## Executive Summary

**Claw Code** is a Rust CLI agent harness — a port of Claude Code designed to be machine-controllable ("clawable"). It demonstrates autonomous software development coordinated by AI agents ("claws") with human direction via Discord.

### The Stack
- **Language:** Rust (2021 edition), 0 unsafe code
- **Workspace:** 9 crates, ~48.6K Rust LOC, ~2.6K test LOC
- **Binary:** `claw` CLI with REPL, one-shot prompts, and 100+ slash commands
- **Providers:** Anthropic Claude (primary), XAI/Grok, OpenAI-compatible
- **Testing:** Mock Anthropic service, compat-harness against upstream TypeScript, 12 parity scenarios

### The Philosophy
> "Humans set direction; claws perform the labor."

The product is not the code — it's the **coordination system** that produced it. The repository demonstrates autonomous development with:
- Clawhip (event/notification routing)
- OmO (multi-agent coordination)
- OmX (workflow automation)

---

## Key Insights

### 1. Clean Dependency Layering
The workspace has a clear bottom-up architecture with no cycles. Leaf crates (`telemetry`, `plugins`) have no internal dependencies, and everything flows up to the binary. This makes each crate independently testable and replaceable.

### 2. Trait-Based Abstraction
`ConversationRuntime<C, T>` is generic over `ApiClient` and `ToolExecutor`, enabling:
- Provider swapping (Anthropic → OpenAI-compatible)
- Mock testing without real API calls
- Clean separation of concerns

### 3. Global Singletons via `OnceLock`
All registries (tools, MCP, LSP, tasks, teams, crons, workers) use `OnceLock` for lazy, thread-safe, once-per-process initialization. Simple and effective for a CLI.

### 4. State Machines Everywhere
- **Workers:** `Spawning → TrustRequired → ReadyForPrompt → Running → Finished/Failed`
- **Lanes:** `Started → Ready → Green → CommitCreated → MergeReady → Finished`
- **Permissions:** `Allow / Den / Prompt`
- **MCP:** Config → Spawn → Initialize → Discover → Running/Degraded

This makes the system deterministic and machine-readable.

### 5. Recovery Before Escalation
7 failure scenarios have automatic recovery recipes that attempt once before escalating to a human. This is the core "clawable" philosophy — don't wake the human for recoverable errors.

### 6. Event-Native Design
Typed `LaneEvent` enum with 16 variants replaces scraped terminal text. Events carry structured data (`LaneFailureClass`, `LaneCommitProvenance`) enabling machine-driven monitoring and recovery.

### 7. Permission Enforcement
Every tool has a `required_permission` field. Permission checks happen at dispatch time with workspace boundary validation, symlink escape detection, and bash read-only heuristics.

### 8. Session Persistence with Rotation
Sessions are JSONL files with automatic rotation (256KB chunks, max 3 files), compaction support, and fork tracking. Located in `.claw/sessions/`.

---

## Areas Worth Deeper Study

### High Priority
1. **`conversation.rs`** (1,691 lines) — The core loop logic
2. **`main.rs`** (9,476 lines) — CLI entry point, REPL, command dispatch
3. **`tools/src/lib.rs`** (5,311 lines) — Tool registry and execution
4. **`commands/src/lib.rs`** (7,900 lines) — Slash command surface

### Medium Priority
5. **`runtime/src/lib.rs`** — Module re-exports, type definitions
6. **`runtime/src/mcp_stdio.rs`** — MCP process management
7. **`runtime/src/worker_boot.rs`** — Worker state machine
8. **`runtime/src/policy_engine.rs`** — Policy evaluation

### Lower Priority (but valuable)
9. **`runtime/src/session.rs`** — Session persistence
10. **`runtime/src/recovery_recipes.rs`** — Recovery automation
11. **`runtime/src/lane_events.rs`** — Event schema
12. **`api/src/client.rs`** — Provider client

---

## Study Strategy

### Phase 1: Understand the Flow (2-3 hours)
1. Read `PHILOSOPHY.md` — understand why this exists
2. Read `USAGE.md` — understand how to use it
3. Read `ARCHITECTURE_NOTES.md` — understand the structure
4. Read `FLOW_MAPS.md` — understand how subsystems interact
5. Build and run `claw doctor`

### Phase 2: Trace a Single Request (2-3 hours)
1. Start with `main.rs` → `parse_args()` → `CliAction::Prompt`
2. Follow to `LiveCli::run_turn_with_output()`
3. Trace into `ConversationRuntime::run_turn()`
4. Follow the `ApiClient::stream()` call
5. Trace tool execution through `GlobalToolRegistry::execute()`
6. Follow permission checking through `PermissionEnforcer`

### Phase 3: Study Key Modules (4-6 hours)
1. **Session system** — `session.rs` + persistence + rotation
2. **Worker boot** — `worker_boot.rs` state machine
3. **MCP lifecycle** — `mcp_stdio.rs` + `mcp_lifecycle_hardened.rs`
4. **Policy engine** — `policy_engine.rs` evaluation logic
5. **Recovery recipes** — `recovery_recipes.rs` automatic recovery

### Phase 4: Understand the Tests (2-3 hours)
1. Mock parity harness — 12 scenarios
2. Compat-harness — upstream parity checking
3. Integration tests — worker→recovery→policy flows
4. Output format contract — JSON parity across all commands

---

## Notable Design Decisions

### Why Rust?
- Performance and memory safety
- No GC pauses (important for agent reliability)
- Static binary (easy distribution)
- Type system catches bugs at compile time

### Why No Unsafe Code?
- `unsafe_code = "forbid"` at workspace level
- Security-critical system handling file I/O, network, subprocess execution
- Agents shouldn't execute memory-unsafe code

### Why JSONL for Sessions?
- Append-only (crash-safe)
- Line-oriented (easy to parse/rotate)
- Human-readable (debuggable)
- Compatible with streaming writes

### Why Global Singletons?
- CLI is single-process
- `OnceLock` provides safe lazy initialization
- Avoids threading registry through every function signature
- Trade-off: harder to test with multiple registries (mitigated by test isolation)

### Why Trait-Based Runtime?
- `ConversationRuntime<C, T>` enables mocking
- Provider-agnostic (Anthropic, XAI, OpenAI-compatible)
- Clean test boundaries
- Future-proof for new providers

---

## Current Gaps (from PARITY.md)

### Still Open
- [ ] End-to-end MCP runtime lifecycle beyond registry bridge
- [ ] Session compaction behavior matching upstream
- [ ] Token counting / cost tracking accuracy verification
- [ ] CI green on every commit (some flakes remain)

### Recently Closed
- [x] Output truncation for large stdout/file content
- [x] Config merge precedence with hook validation
- [x] Plugin install/enable/disable/uninstall flow
- [x] Bash validation lane on main
- [x] Resumed JSON parity for all slash commands
- [x] Doctor JSON output structure
- [x] Session state classification
- [x] Branch lock collision detection
- [x] Context-window preflight blocking

---

## Related Projects

| Project | Purpose |
|---------|---------|
| [clawhip](https://github.com/Yeachan-Heo/clawhip) | Event and notification router |
| [oh-my-openagent](https://github.com/code-yeongyu/oh-my-openagent) | Multi-agent coordination |
| [oh-my-codex](https://github.com/Yeachan-Heo/oh-my-codex) | Workflow layer (directives → structured execution) |
| [oh-my-claudecode](https://github.com/Yeachan-Heo/oh-my-claudecode) | Claude Code customization |

---

## Glossary

| Term | Definition |
|------|-----------|
| **Claw** | Autonomous coding agent that executes work |
| **Clawable** | Machine-controllable, event-native, recoverable without human |
| **Lane** | A unit of work being executed by a claw |
| **Worker** | A coding agent subprocess with explicit lifecycle states |
| **clawhip** | Event/notification routing system (separate from this repo) |
| **OmO** | Multi-agent coordination layer (oh-my-openagent) |
| **OmX** | Workflow automation layer (oh-my-codex) |
| **MCP** | Model Context Protocol — tool/resource protocol for LLMs |
| **Green Level** | Test pass quality (targeted → package → workspace → merge-ready) |
| **Parity** | Feature-for-feature match with upstream Claude Code |
| **Compat-Harness** | Automated upstream parity checker |
| **Doctor** | Built-in diagnostic command (`claw doctor` or `/doctor`) |
| **Trust Gate** | Security prompt asking user to trust a repo |
| **Prompt Misdelivery** | Prompt sent to shell instead of coding agent |
| **Stale Branch** | Branch missing recent main commits |
| **Recovery Recipe** | Automatic fix for known failure scenario |
| **Policy Engine** | Rules governing merge/recovery/escalation decisions |
| **Session** | Persisted conversation (JSONL, rotatable, resumable) |
| **Compaction** | Summarizing old messages to reduce context window |
| **Bootstrap Plan** | Initial setup sequence for new repos |

---

## File Locations Cheat Sheet

```
Top-level docs:
  README.md        — Project overview + quick start
  USAGE.md         — Task-oriented usage guide
  PHILOSOPHY.md    — Why this exists, how it's operated
  ROADMAP.md       — Active roadmap and cleanup backlog
  PARITY.md        — Rust port parity status
  CLAUDE.md        — Agent working instructions

New study notes (created during analysis):
  docs/ARCHITECTURE_NOTES.md  — Overall architecture breakdown
  docs/CRATE_STUDY_GUIDE.md   — Per-crate deep dive
  docs/FLOW_MAPS.md           — Subsystem interaction flows
  docs/STUDY_SUMMARY.md       — This file

Rust workspace:
  rust/Cargo.toml              — Workspace configuration
  rust/crates/runtime/         — Core runtime (largest crate)
  rust/crates/tools/           — Tool registry
  rust/crates/commands/        — Slash commands
  rust/crates/api/             — HTTP client layer
  rust/crates/plugins/         — Plugin system
  rust/crates/telemetry/       — Observability
  rust/crates/rusty-claude-cli/ — CLI binary
  rust/crates/compat-harness/  — Upstream parity checker
  rust/crates/mock-anthropic-service/ — Test harness

Python/reference:
  src/   — Python/reference source
  tests/ — Validation surfaces

Config:
  .claude.json               — Agent settings
  .claude/sessions/          — Session storage
  .github/workflows/         — CI configuration
```

---

## Next Steps

1. **Build it:** `cd rust && cargo build --workspace`
2. **Run tests:** `cargo test --workspace`
3. **Health check:** `./target/debug/claw doctor`
4. **Try it:** `./target/debug/claw prompt "summarize this repository"`
5. **Explore interactively:** `./target/debug/claw` then `/help`
6. **Study code:** Start with `conversation.rs` for the core loop logic
