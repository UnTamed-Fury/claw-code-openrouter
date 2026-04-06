# Conceptual Flow Maps

> How the major subsystems interact during real usage.

---

## 1. User Prompt Flow (REPL)

```
User types in terminal
        ↓
┌───────────────────────────────────────────────────────────────┐
│ rusty-claude-cli (main.rs)                                    │
│                                                               │
│  1. Parse CLI args → CliAction::Repl                          │
│  2. Load config (5-level precedence)                          │
│  3. Resolve auth source (API key or OAuth)                    │
│  4. Create Session (new or resumed from .claw/sessions/)      │
│  5. Build ApiClient (AnthropicClient or OpenAiCompatClient)   │
│  6. Create GlobalToolRegistry                                 │
│  7. Create PermissionEnforcer                                 │
│  8. Load system prompt                                        │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ ConversationRuntime<AnthropicClient, GlobalToolRegistry>       │
│                                                               │
│  Loop (max_iterations):                                       │
│    1. ApiClient.stream(request) → SSE stream                  │
│    2. For each event:                                         │
│       - TextDelta → render to terminal (Markdown streaming)   │
│       - ToolUse → execute tool via GlobalToolRegistry         │
│       - Usage → update UsageTracker                           │
│       - MessageStop → break loop                              │
│    3. Check auto-compaction threshold                         │
│    4. Append messages to Session                              │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Tool Execution (GlobalToolRegistry)                           │
│                                                               │
│  1. Look up tool by name                                      │
│  2. Check permission (PermissionEnforcer)                     │
│     - If denied → return error                                │
│     - If allowed → execute                                    │
│  3. Execute implementation:                                   │
│     - bash → runtime::execute_bash()                          │
│     - read_file → runtime::read_file()                        │
│     - write_file → runtime::write_file()                      │
│     - edit_file → runtime::edit_file()                        │
│     - MCP → global_mcp_registry().dispatch()                  │
│     - LSP → global_lsp_registry().dispatch()                  │
│     - etc.                                                    │
│  4. Return result as JSON string                              │
└───────────────────────────────────────────────────────────────┘
        ↓
Session persisted to .claw/sessions/<id>.jsonl
```

---

## 2. Worker Boot Flow

```
Claw requests worker start
        ↓
┌───────────────────────────────────────────────────────────────┐
│ WorkerRegistry                                                 │
│                                                               │
│  1. Create Worker with status = Spawning                      │
│  2. Emit WorkerEvent::Spawning                                │
│  3. Spawn subprocess (coding agent)                           │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Trust Gate                                                     │
│                                                               │
│  1. Detect trust prompt in subprocess output                  │
│  2. status = TrustRequired                                    │
│  3. Emit WorkerEvent::TrustRequired                           │
│  4. Check if repo is allowlisted:                             │
│     - Yes → auto-resolve → TrustResolved                      │
│     - No → wait for manual approval                           │
│  5. Emit WorkerEvent::TrustResolved                           │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Ready Handshake                                                │
│                                                               │
│  1. Wait for "ready_for_prompt" signal                        │
│  2. status = ReadyForPrompt                                   │
│  3. Emit WorkerEvent::ReadyForPrompt                          │
│  4. Prompt is now safe to send                                │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Prompt Delivery                                                │
│                                                               │
│  1. Send prompt to worker                                     │
│  2. Emit WorkerEvent::PromptDelivery                          │
│  3. Detect if delivered to correct target:                    │
│     - Shell → PromptMisdelivery → auto-recover               │
│     - Agent → Running                                        │
│  4. status = Running                                          │
│  5. Emit WorkerEvent::Running                                 │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Completion / Failure                                           │
│                                                               │
│  On success:                                                  │
│    status = Finished                                          │
│    Emit WorkerEvent::Finished                                 │
│                                                               │
│  On failure:                                                  │
│    status = Failed                                            │
│    Record WorkerFailure (kind + message)                      │
│    Emit WorkerEvent::Failed                                   │
│    → Bridge to RecoveryRecipes                                │
└───────────────────────────────────────────────────────────────┘
```

---

## 3. Lane Event Flow

```
Lane starts
        ↓
┌───────────────────────────────────────────────────────────────┐
│ LaneEvent::Started(status=Running)                            │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Worker boots → LaneEvent::Ready(status=Ready)                 │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Prompt delivered → work begins                                │
│                                                               │
│  On success path:                                             │
│    LaneEvent::Green(status=Green)                             │
│    LaneEvent::CommitCreated(status=Completed)                 │
│      └─ provenance: branch/worktree/canonical-commit/lineage  │
│    LaneEvent::MergeReady(status=Completed)                    │
│    LaneEvent::Finished(status=Completed)                      │
│                                                               │
│  On failure path:                                             │
│    LaneEvent::Red(status=Red)                                 │
│      └─ failure_class: Compile/Test/McpStartup/etc.           │
│    LaneEvent::Blocked(status=Blocked)                         │
│      └─ blocker: { failure_class, detail }                    │
│    LaneEvent::Failed(status=Failed)                           │
│      → RecoveryRecipes attempts auto-recovery                 │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Policy Engine evaluates LaneContext                            │
│                                                               │
│  If green + scoped_diff + review_passed → MergeToDev          │
│  If stale_branch → MergeForward before broad tests            │
│  If startup_blocked → RecoverOnce, then Escalate              │
│  If lane_completed → CloseoutLane + CleanupSession            │
└───────────────────────────────────────────────────────────────┘
```

---

## 4. Recovery Flow

```
Failure detected (WorkerFailure / LaneEvent::Failed)
        ↓
┌───────────────────────────────────────────────────────────────┐
│ FailureScenario::from_worker_failure_kind()                   │
│                                                               │
│  Maps:                                                        │
│    TrustGate → TrustPromptUnresolved                          │
│    PromptDelivery → PromptMisdelivery                         │
│    Protocol → McpHandshakeFailure                             │
│    Provider → ProviderFailure                                 │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ recipe_for(scenario) → RecoveryRecipe                         │
│                                                               │
│  Recipe contains:                                             │
│    - steps: Vec<RecoveryStep>                                 │
│    - max_attempts: u32                                        │
│    - escalation_policy: EscalationPolicy                      │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ attempt_recovery(recipe, context)                             │
│                                                               │
│  1. Check attempt count < max_attempts                        │
│  2. Execute steps in order:                                   │
│     - AcceptTrustPrompt → auto-clear trust gate              │
│     - RedirectPromptToAgent → fix misdelivery                │
│     - RebaseBranch → fix stale branch                        │
│     - CleanBuild → fix cross-crate compile errors            │
│     - RetryMcpHandshake → retry with timeout                 │
│     - RestartPlugin → restart failed plugin                  │
│     - RestartWorker → full worker restart                    │
│  3. Emit RecoveryEvent::RecoveryAttempted                     │
│  4. Return RecoveryResult:                                    │
│     - Recovered { steps_taken }                               │
│     - PartialRecovery { recovered, remaining }                │
│     - EscalationRequired { reason }                           │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ On success: retry original operation                         │
│ On escalation: EscalationPolicy::AlertHuman                  │
└───────────────────────────────────────────────────────────────┘
```

---

## 5. Permission Check Flow

```
Model requests tool use
        ↓
┌───────────────────────────────────────────────────────────────┐
│ GlobalToolRegistry.execute(tool_name, input)                  │
│                                                               │
│  1. Find ToolSpec by name                                     │
│  2. Get required_permission from spec                         │
│  3. Check against allowed_tools set                           │
│  4. If PermissionEnforcer exists:                             │
│     a. check_permission(tool_name, context)                   │
│     b. For file tools: check_file_write(path)                 │
│        - Validate workspace boundary                          │
│        - Check read-only mode                                 │
│        - Detect symlink escape                                │
│     c. For bash: check_bash(command)                          │
│        - Read-only mode heuristics                            │
│        - Destructive command warning                          │
│        - Prompt-mode without confirmation                     │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ PermissionOutcome                                              │
│                                                               │
│  - Allow → execute tool                                       │
│  - Deny { reason } → return error to model                    │
│  - Prompt { message } → ask user (if interactive)             │
│     - User approves → execute                                 │
│     - User denies → return error                              │
└───────────────────────────────────────────────────────────────┘
```

---

## 6. Session Lifecycle Flow

```
┌───────────────────────────────────────────────────────────────┐
│ Session Creation                                               │
│                                                               │
│  1. Session::new() → generates session_id, timestamps          │
│  2. Optional: with_persistence_path(path)                      │
│  3. Optional: with_fork(parent_id, branch_name)               │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Message Accumulation                                           │
│                                                               │
│  1. append_message(role, blocks)                              │
│  2. Each message has: role, content blocks, optional usage    │
│  3. ContentBlock variants:                                    │
│     - Text { text }                                           │
│     - ToolUse { id, name, input }                             │
│     - ToolResult { tool_use_id, tool_name, output, is_error } │
│  4. Auto-save after each append (if persistence path set)     │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Session Persistence                                            │
│                                                               │
│  1. Serialize to JSONL format                                 │
│  2. Check file size > 256KB                                   │
│     - If yes: rotate (rename current, start new)              │
│     - Keep max 3 rotated files                                │
│  3. Write atomic (temp file + rename)                         │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Session Resumption                                             │
│                                                               │
│  1. Load latest or specific session file                      │
│  2. Parse JSONL entries                                       │
│  3. Reconstruct Session with messages                         │
│  4. Continue conversation from checkpoint                     │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Session Compaction                                             │
│                                                               │
│  1. Check token count > threshold (default 100K)              │
│  2. Generate summary of removed messages                      │
│  3. Replace old messages with summary                         │
│  4. Update compaction metadata                                │
└───────────────────────────────────────────────────────────────┘
```

---

## 7. MCP Lifecycle Flow

```
┌───────────────────────────────────────────────────────────────┐
│ MCP Bootstrap                                                  │
│                                                               │
│  1. Load MCP server configs (stdio/remote/sdk/managed-proxy)  │
│  2. For each server:                                          │
│     a. Validate config                                       │
│     b. Spawn stdio process (or connect remote)                │
│     c. Send initialize handshake                              │
│     d. Receive InitializeResult                               │
│     e. Discover tools: list_tools → McpTool[]                │
│     f. Discover resources: list_resources → McpResource[]     │
│     g. Register in McpToolRegistry                           │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Degraded Startup Detection                                     │
│                                                               │
│  1. McpLifecycleValidator checks each phase:                  │
│     - Config validation                                       │
│     - Spawn/connect                                           │
│     - Initialize handshake                                    │
│     - Tool/resource discovery                                 │
│  2. Classify failures:                                        │
│     - McpStartup → server failed to start                    │
│     - McpHandshake → initialize failed                        │
│     - McpConfig → config validation error                     │
│     - Partial → some servers up, some down                    │
│  3. Generate McpDegradedReport:                               │
│     - failed_servers: Vec<McpFailedServer>                    │
│     - recovery_recommendations: Vec<String>                   │
│     - healthy_servers: still usable                           │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ MCP Tool Dispatch                                              │
│                                                               │
│  1. Model calls MCP tool                                      │
│  2. GlobalToolRegistry routes to McpToolRegistry              │
│  3. McpToolBridge dispatches to server via JSON-RPC           │
│  4. Server responds                                           │
│  5. Result returned to model                                  │
│                                                               │
│  If server is down:                                           │
│    → Structured error with failure classification             │
│    → Not opaque "something went wrong"                        │
└───────────────────────────────────────────────────────────────┘
```

---

## 8. Config Loading Flow

```
┌───────────────────────────────────────────────────────────────┐
│ ConfigLoader::discover()                                       │
│                                                               │
│  Loads in order (later overrides earlier):                    │
│    1. ~/.claw.json                                            │
│    2. ~/.config/claw/settings.json                            │
│    3. <repo>/.claw.json                                       │
│    4. <repo>/.claw/settings.json                              │
│    5. <repo>/.claw/settings.local.json                        │
│                                                               │
│  Merge validation:                                            │
│    - Hook validation before deep-merge                        │
│    - Malformed entries fail with source-path context          │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ RuntimeConfig                                                  │
│                                                               │
│  Contains:                                                    │
│    - MCP server configs (stdio/remote/sdk/websocket/managed)  │
│    - OAuth config                                             │
│    - Permission rules                                         │
│    - Hook definitions                                         │
│    - Plugin settings                                          │
│    - Feature flags                                            │
└───────────────────────────────────────────────────────────────┘
```

---

## 9. CLI Command Dispatch Flow

```
User runs: claw [args]
        ↓
┌───────────────────────────────────────────────────────────────┐
│ parse_args(&args) → CliAction                                 │
│                                                               │
│  Matches:                                                     │
│    claw --help         → HelpTopic                            │
│    claw --version      → Version                              │
│    claw doctor         → Doctor                               │
│    claw init           → Init                                 │
│    claw login          → Login                                │
│    claw logout         → Logout                               │
│    claw "prompt"       → Prompt (shorthand)                   │
│    claw prompt "text"  → Prompt                               │
│    claw status         → Status                               │
│    claw sandbox        → Sandbox                              │
│    claw agents         → Agents                               │
│    claw mcp            → Mcp                                  │
│    claw skills         → Skills                               │
│    claw plugins        → Plugins                              │
│    claw --resume X /Y  → ResumeSession                        │
│    claw (no args)      → Repl                                 │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Execute action                                                 │
│                                                               │
│  Each action has independent setup:                           │
│    - Doctor: full preflight diagnostic                        │
│    - Prompt: one-shot API call + render                       │
│    - Repl: interactive loop                                   │
│    - Resume: load session + run commands                      │
│    - Status/Sandbox/Agents/etc: snapshot output               │
│                                                               │
│  Output format:                                               │
│    - Text (default): human-readable                           │
│    - JSON (--output-format json): machine-readable            │
└───────────────────────────────────────────────────────────────┘
```

---

## 10. Stale Branch Detection Flow

```
Before running broad tests
        ↓
┌───────────────────────────────────────────────────────────────┐
│ check_freshness(branch, main)                                 │
│                                                               │
│  1. Get current branch HEAD                                   │
│  2. Get main branch HEAD                                      │
│  3. Calculate:                                                │
│     - commits_behind: how many main commits are missing       │
│     - commits_ahead: local commits not in main                │
│     - last_common_ancestor                                   │
│     - time_since_divergence                                   │
│  4. Return BranchFreshness                                    │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ BranchFreshness                                                │
│                                                               │
│  - Fresh: recently diverged, few commits behind               │
│  - Stale: diverged long ago, many commits behind              │
│    → Emit branch.stale_against_main event                     │
│    → Suggest rebase/merge-forward                              │
└───────────────────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────────────────┐
│ apply_policy(freshness, policy)                                │
│                                                               │
│  If stale and policy.auto_rebase:                             │
│    → Attempt rebase before tests                              │
│    → Prevents misclassifying stale-branch failures as new bugs │
│                                                               │
│  If fresh:                                                    │
│    → Proceed with tests normally                              │
└───────────────────────────────────────────────────────────────┘
```
