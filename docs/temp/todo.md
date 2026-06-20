# Muse — TODO

Updated after **v2.1** (P0 deferred + P1 + P2).

---

## Status summary

| Track | Status |
|-------|--------|
| **P0 v2.0** (verify, diff, tools, context pack) | Done |
| **P0 deferred** (LLM eval in CI with live API) | Done — optional `MUSE_EVAL_LIVE=1` harness (5 scenarios); CI opt-in via secret |
| **P1 v1.1** (personalities) | Done — Luna, Sage, Spark |
| **P1 v1.2** (long-term memory) | Done — CRUD + injection into companion + TaskSpec |
| **P2 git tools** | Done — `git_add`, `git_commit`, `git_checkout_branch` + allowlist prefixes |
| **P2 command allowlist settings** | Done — extra prefixes in Settings |
| **P2 task queue** | Done — sequential queue when enabled |
| **P2 file watcher RAG** | Done — auto-index on workspace changes |
| **P2 MCP client** | Done (minimal) — stdio MCP server, tools merged into executor |

---

## Manual test checklist

Run `npm run tauri dev` with a real workspace folder and API (Ollama or OpenAI).

### P0 — Reliable local coding agent

**Verify loop**

1. Settings → enable verify, leave command empty → delegate a small code edit
2. Confirm executor activity shows a `verify` step and `cargo test` / `npm test` runs
3. Break a test intentionally → agent receives stderr and retries (up to 3)

**Change review**

1. After a delegated edit → change review panel lists files + unified diff
2. Revert one file → only that file restores
3. Revert all → workspace matches pre-run state

**Context pack**

1. Toggle context pack off → save → delegate; TaskSpec should lack tree/git block (check executor activity objective/context if visible)
2. Toggle on → delegate; context includes capped tree + `git status --short`

**Structured tools / native calling**

1. OpenAI or tool-capable Ollama → executor uses function calls (watch activity / model behavior)
2. Provider without tools → falls back to JSON `action` responses without crashing

**Agent bench (dev)**

```bash
cargo test bench:: --manifest-path src-tauri/Cargo.toml
```

**Live LLM eval (optional)**

```bash
$env:MUSE_EVAL_LIVE="1"
$env:MUSE_API_BASE="http://localhost:11434/v1"
$env:MUSE_MODEL="llama3.2"
cargo test live_eval --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture
```

---

### P1 — Context & UX (regression)

**Streaming + cancel**

1. Delegate a task → plan streams in status bar
2. Final reply streams with cursor in message bubble
3. Click **Stop** during executor → Luna reports cancellation

**Incremental RAG**

1. Full index workspace → edit one file → **Index changes** only
2. Index/test embedding without saving Settings first

**UI / RTL**

1. Warm glass UI, Luna moon marker, status bar phases
2. Switch to Persian → layout RTL-safe; technical inputs stay LTR

---

### P1 — Personalities (v1.1)

1. Settings → Preferences → pick **Sage** or **Spark** → save
2. Reset chat (or fresh DB) → greeting matches personality
3. Companion message labels update (Sage / Spark / Luna)
4. Delegate a task → executor behavior unchanged; tone differs in companion replies

---

### P1 — Long-term memory (v1.2)

1. Settings → **Long-term memory** → add “Prefers Rust over Python”
2. Delegate a coding task → executor context includes `User memories` block
3. Ask a preference question in chat → companion system prompt includes memory
4. Delete memory → no longer appears in new delegations

---

### P2 — Power features

**Git tools**

1. In a git workspace, ask Luna to stage and commit with a message
2. Ask to create branch `feature/test-branch` → `git checkout -b` runs
3. Disallowed command (e.g. `rm -rf`) → tool error, no execution

**Command allowlist**

1. Settings → add prefix `make ` (one line) → save
2. Delegate “run make test” (if Makefile exists) → succeeds
3. Remove prefix → same command blocked

**Task queue**

1. Enable **sequential queue** in Settings
2. Queue a task via API (`queue_task`) or send two delegations in quick succession if UI allows
3. Tasks run FIFO; completed items marked in DB

**Auto-index RAG**

1. Enable RAG + **auto-index workspace changes**
2. Edit a file in workspace → wait ~3s → chunk count updates (or re-open RAG status)
3. Disable auto-index → edits no longer trigger background index

**MCP (optional)**

1. Install an MCP server (e.g. filesystem server)
2. Settings → enable MCP, set command → save
3. Delegate task using an MCP tool → executor calls `mcp_*` tool; result in activity log
4. Invalid/disconnected MCP command → activity shows MCP error; workspace tools still work

---

### Connection & settings smoke

1. **Test connection** shows model + latency (local Ollama without API key)
2. **Test embeddings** with unsaved form values
3. Save settings → reload app → values persist

---

## References

- [plan.md](./plan.md)
- [ROADMAP.md](../../ROADMAP.md)
- [development.md](../development.md)
