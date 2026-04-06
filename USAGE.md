# Claw Code Usage

This guide covers the current Rust workspace under `rust/` and the `claw` CLI binary. If you are brand new, make the doctor health check your first run: start `claw`, then run `/doctor`.

## Quick-start health check

Run this before prompts, sessions, or automation:

```bash
cd rust
cargo build --workspace
./target/debug/claw
# first command inside the REPL
/doctor
```

`/doctor` is the built-in setup and preflight diagnostic. Once you have a saved session, you can rerun it with `./target/debug/claw --resume latest /doctor`.

## Prerequisites

- Rust toolchain with `cargo`
- One of:
  - `ANTHROPIC_API_KEY` for direct API access
  - `claw login` for OAuth-based auth
- Optional: `ANTHROPIC_BASE_URL` when targeting a proxy or local service

## Install / build the workspace

```bash
cd rust
cargo build --workspace
```

The CLI binary is available at `rust/target/debug/claw` after a debug build. Make the doctor check above your first post-build step.

## Quick start

### First-run doctor check

```bash
cd rust
./target/debug/claw
/doctor
```

### Interactive REPL

```bash
cd rust
./target/debug/claw
```

### One-shot prompt

```bash
cd rust
./target/debug/claw prompt "summarize this repository"
```

### Shorthand prompt mode

```bash
cd rust
./target/debug/claw "explain rust/crates/runtime/src/lib.rs"
```

### JSON output for scripting

```bash
cd rust
./target/debug/claw --output-format json prompt "status"
```

## Model and permission controls

```bash
cd rust
./target/debug/claw --model sonnet prompt "review this diff"
./target/debug/claw --permission-mode read-only prompt "summarize Cargo.toml"
./target/debug/claw --permission-mode workspace-write prompt "update README.md"
./target/debug/claw --allowedTools read,glob "inspect the runtime crate"
```

Supported permission modes:

- `read-only`
- `workspace-write`
- `danger-full-access`

Model aliases currently supported by the CLI:

**Anthropic (direct):**
- `opus` → `claude-opus-4-6`
- `sonnet` → `claude-sonnet-4-6`
- `haiku` → `claude-haiku-4-5-20251213`

**OpenRouter (multi-provider gateway):**
- `or-sonnet` → `anthropic/claude-sonnet-4.6`
- `or-opus` → `anthropic/claude-opus-4.6`
- `or-haiku` → `anthropic/claude-3-5-haiku-20241022`
- `or-gpt-4o` → `openai/gpt-4o`
- `or-gpt-5` → `openai/gpt-5`
- `or-gemini` → `google/gemini-2.5-pro`
- `or-grok` → `x-ai/grok-3`
- `or-qwen` → `qwen/qwen3-235b-a22b`
- `or-free` → `openrouter/free`

You can also pass raw provider-prefixed IDs directly (e.g., `--model anthropic/claude-sonnet-4.6`).

## Authentication

### API key

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

### OAuth

```bash
cd rust
./target/debug/claw login
./target/debug/claw logout
```

### OpenRouter

[OpenRouter](https://openrouter.ai) provides access to hundreds of models (Anthropic, OpenAI, Google, etc.) through a single API key.

```bash
export OPENROUTER_API_KEY="sk-or-..."
cd rust
./target/debug/claw --model or-sonnet "explain this code"
./target/debug/claw --model or-gpt-5 "write a test"
./target/debug/claw --model or-gemini "review this PR"
./target/debug/claw --model anthropic/claude-sonnet-4.6 "summarize the repo"
```

You can also use raw provider-prefixed model IDs directly — any model ID containing `/` (like `anthropic/...`, `openai/...`, `google/...`) auto-detects as OpenRouter.

## Common operational commands

```bash
cd rust
./target/debug/claw status
./target/debug/claw sandbox
./target/debug/claw agents
./target/debug/claw mcp
./target/debug/claw skills
./target/debug/claw system-prompt --cwd .. --date 2026-04-04
```

## Session management

REPL turns are persisted under `.claw/sessions/` in the current workspace.

```bash
cd rust
./target/debug/claw --resume latest
./target/debug/claw --resume latest /status /diff
```

Useful interactive commands include `/help`, `/status`, `/cost`, `/config`, `/session`, `/model`, `/permissions`, and `/export`.

## Config file resolution order

Runtime config is loaded in this order, with later entries overriding earlier ones:

1. `~/.claw.json`
2. `~/.config/claw/settings.json`
3. `<repo>/.claw.json`
4. `<repo>/.claw/settings.json`
5. `<repo>/.claw/settings.local.json`

## Mock parity harness

The workspace includes a deterministic Anthropic-compatible mock service and parity harness.

```bash
cd rust
./scripts/run_mock_parity_harness.sh
```

Manual mock service startup:

```bash
cd rust
cargo run -p mock-anthropic-service -- --bind 127.0.0.1:0
```

## Verification

```bash
cd rust
cargo test --workspace
```

## Workspace overview

Current Rust crates:

- `api`
- `commands`
- `compat-harness`
- `mock-anthropic-service`
- `plugins`
- `runtime`
- `rusty-claude-cli`
- `telemetry`
- `tools`
