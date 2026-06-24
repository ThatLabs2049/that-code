# ThatCode — Architecture

**ThatCode** is a local-first Tauri desktop app: React/TypeScript UI, Rust backend, OpenAI-compatible API, SQLite persistence.

**Status:** **v2.7.1** shipped (Phases 0–7). Phase 8–9 in [ROADMAP.md](./ROADMAP.md).  
**Historical:** Muse v2.x used a dual companion + executor pipeline (removed Phase 2). See [DECISIONS.md](./DECISIONS.md) ADR-014.

---

## System diagram

```
┌──────────────────────────────────────────────────────────────┐
│  Tauri 2.x (Windows v1: NSIS + MSI)                          │
├──────────────────────────────────────────────────────────────┤
│  React UI (src/)                                             │
│    ChatScreen · Composer (@file/@symbol) · Change review     │
│    Tool timeline · Settings · Command palette (Ctrl+K)       │
├──────────────────────────────────────────────────────────────┤
│  Rust (src-tauri/)                                           │
│    commands/ · orchestrator/ · agents/ · tools/ · rag/       │
│    db/ (SQLite) · mcp/ · workspace/                          │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
              OpenAI-compatible API (cloud or local, e.g. Ollama)
```

---

## Request flow

```
User message (+ optional @ attachments)
    │
    ▼
send_message (IPC)
    │
    ▼
Orchestrator::run_turn
    ├── casual chat? ──► direct chat completion (streamed)
    │
    └── agent task
            ├── enrich: context pack, memories, rules, RAG, attachments
            ├── optional: explore-then-implement (scout → editor)
            └── AgentTier → RunConfig (scout / editor phases, models)
                    │
                    ▼
            agents::executor (tool loop)
                    │
                    ▼
            Persist message + executor_run + file diffs
                    │
                    ▼
            UI: stream tokens, activity panel, change review
```

**Agent tiers:** `auto`, `quick`, `standard`, `deep`, `explain`. Standard uses fast model for scout (read-only) and strong model for editor when `auto_escalate` is on.

**Plan gate:** When `plan_before_edit` is enabled, scout pauses for user approval before editor phase (`respond_to_agent_plan`).

---

## Tech stack

| Layer | Technology |
|-------|------------|
| Shell | Tauri 2.x |
| Frontend | React, TypeScript, Vite |
| Backend | Rust |
| AI | OpenAI-compatible chat + embeddings |
| Database | SQLite (`rusqlite`) |
| IPC | Tauri `invoke` + events (`assistant-stream`, `executor-progress`) |

---

## Project layout

```
/
├── src/                      # React frontend
│   ├── components/           # Chat, settings, sidebar panels
│   ├── hooks/                # useChat, useRagStatus, useAppTheme
│   ├── lib/                  # IPC wrappers, i18n, settings types
│   └── styles/               # tokens.css, global.css
├── src-tauri/
│   ├── src/
│   │   ├── commands/         # Tauri IPC handlers
│   │   ├── orchestrator/     # run_turn, RAG/task enrichment
│   │   ├── agents/           # executor, profile (tiers), task specs
│   │   ├── tools/            # Sandbox, grep, edit, verify, git
│   │   ├── rag/              # Chunking, embeddings, index
│   │   ├── workspace/        # Path/symbol search, git status
│   │   ├── changes/          # Diffs, hunks, revert
│   │   ├── ai/               # HTTP client, prompts
│   │   └── db/               # Schema, messages, task queue
│   └── tauri.conf.json
└── docs/                     # TRUST, development, temp plans
```

---

## Frontend

| Area | Responsibility |
|------|----------------|
| `ChatScreen` | Layout, sidebar (git, task queue, activity, diffs) |
| `Composer` | Message input, tier select, `@` context picker, explore→edit |
| `useChat` | Messages, send/cancel, stream listeners, plan approval |
| `SettingsPanel` | API, workspace, agent models, RAG, MCP |
| `CommandPalette` | `Ctrl+K` shortcuts |

**Theme:** `themePreference` (`dark` / `light` / `system`) → `data-theme` on `<html>`; tokens in `src/styles/tokens.css`.

**i18n:** English + Persian (`fa`), RTL via `dir` on document root.

---

## Backend modules

| Module | Role |
|--------|------|
| `commands::chat` | `send_message`, `clear_history`, streaming |
| `commands::rag` | Index workspace, search codebase |
| `commands::workspace` | `search_workspace_paths`, `search_workspace_symbols`, git status |
| `commands::changes` | Diff, hunk reject, revert, open in editor |
| `orchestrator` | Route turn, enrich task, run agent or chat |
| `agents::executor` | Multi-step tool loop with progress callbacks |
| `agents::profile` | Tier → `RunConfig` (models, scout/editor limits) |
| `tools` | Sandboxed file ops + allowlisted `run_command` |
| `rag` | Local embeddings index in SQLite |
| `mcp` | Optional stdio MCP server for extra tools |
| `db` | Conversations, messages (`user` / `companion` role), settings blob |

---

## Key IPC commands

| Command | Purpose |
|---------|---------|
| `send_message` | User turn; optional `agentTier`, `attachments`, `exploreThenImplement` |
| `cancel_run` | Stop in-flight agent |
| `respond_to_agent_plan` | Approve/reject scout plan |
| `get_settings` / `update_settings` | Local preferences + API config |
| `index_workspace_rag` / `search_codebase` | RAG index and retrieval |
| `get_executor_run_changes` | File diffs for review |
| `list_queued_tasks` | Task queue sidebar |
| `get_workspace_git_status` | Branch + diff stat |

---

## Data model (SQLite)

- **conversations** — single active chat (v3)
- **messages** — `user` | `companion` (assistant content stored as `companion`)
- **executor_runs** — task spec, result JSON, file changes, status
- **settings** — JSON blob (`ai_settings` key)
- **rag_chunks** — embedding vectors + source paths
- **task_queue** — sequential delegated tasks per conversation
- **memories** — user notes injected into agent context

App data: `%APPDATA%/com.thatcode.app/` (Windows). DB filename `muse.db` (legacy name, unchanged for upgrade path).

---

## Agent tools (workspace sandbox)

Read-only: `list_dir`, `read_file`, `grep`, `search_files`, `file_info`  
Mutating: `write_file`, `edit_file`, `delete_file`, `create_dir`, `run_command`, git helpers  
Verify: post-edit test command (auto-detect or configured)  
Optional: MCP tools prefixed `mcp_*`

Commands run in the selected project folder only; path traversal blocked.

---

## Security & trust

- API keys stored locally in SQLite settings (device-only)
- No telemetry by default
- Change review + per-file / per-hunk revert before trusting edits
- Release binaries: SHA256 manifest — [docs/TRUST.md](docs/TRUST.md)

---

## Build & test

```bash
npm ci
npm run tauri dev          # development
npm run build && npm run test:rust && npm run lint:rust
npm run tauri build        # Windows NSIS + MSI
```

See [docs/development.md](docs/development.md).

---

## Related documents

| Document | Purpose |
|----------|---------|
| [README.md](README.md) | Install, features, screenshots |
| [DECISIONS.md](DECISIONS.md) | ADRs (incl. Muse → ThatCode pivot) |
| [ROADMAP.md](ROADMAP.md) | Phase milestones |
| [docs/temp/thatcode-todo.md](docs/temp/thatcode-todo.md) | Phase checklist |
| [docs/TRUST.md](docs/TRUST.md) | Verify downloads |
