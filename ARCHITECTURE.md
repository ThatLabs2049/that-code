# Muse — Architecture

## Overview

Muse is a Tauri desktop application with a React/TypeScript frontend and a Rust backend. AI orchestration runs through OpenAI-compatible APIs. Conversation history persists in local SQLite.

The core architectural pattern is **dual-agent orchestration**: a high-temperature Companion agent handles user-facing conversation; a low-temperature Executor agent handles structured planning and task completion when needed.

---

## System diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Tauri Desktop App                        │
├─────────────────────────────────────────────────────────────┤
│  React UI (src/)                                             │
│  ├── ChatScreen                                              │
│  ├── Settings                                                │
│  └── Optional ExecutorActivityPanel                          │
├─────────────────────────────────────────────────────────────┤
│  Rust Backend (src-tauri/)                                   │
│  ├── Commands (IPC)                                          │
│  ├── Orchestrator                                            │
│  ├── CompanionService                                        │
│  ├── ExecutorService                                         │
│  ├── AI Client (OpenAI-compatible)                           │
│  └── SQLite (conversations, messages, settings)              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    OpenAI-compatible API
                    (OpenAI, Azure, local proxy, etc.)
```

---

## Request flow

```
User message
    │
    ▼
┌───────────────────┐
│  Chat UI          │  invoke: send_message
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│  Orchestrator     │  persist user message
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│  Companion Agent  │  analyze intent, respond or delegate
└─────────┬─────────┘
          │
          ├── no execution needed ──► companion reply ──► UI
          │
          └── execution needed
                    │
                    ▼
          ┌───────────────────┐
          │  TaskSpec builder │  structured objective JSON
          └─────────┬─────────┘
                    │
                    ▼
          ┌───────────────────┐
          │  Executor Agent   │  plan, tools, structured output
          └─────────┬─────────┘
                    │
                    ▼
          ┌───────────────────┐
          │  Companion Agent  │  format result for user
          └─────────┬─────────┘
                    │
                    ▼
               UI (single thread)
```

---

## Tech stack

| Layer | Technology | Notes |
|-------|------------|-------|
| Shell | Tauri 2.x | Cross-platform desktop |
| Frontend | React 18+, TypeScript | Vite bundler (typical Tauri setup) |
| Backend | Rust | Tauri commands, orchestration, persistence |
| AI | OpenAI-compatible HTTP API | Configurable base URL and models |
| Database | SQLite | Via `rusqlite` or Tauri plugin |
| IPC | Tauri `invoke` | Frontend ↔ Rust |

---

## Project layout

```
/
├── README.md
├── AGENTS.md
├── ARCHITECTURE.md
├── PRODUCT.md
├── ROADMAP.md
├── DECISIONS.md
├── docs/
│   ├── getting-started.md
│   ├── development.md
│   └── contributing.md
├── src/                    # React frontend
│   ├── components/
│   ├── hooks/
│   ├── lib/
│   ├── styles/
│   └── App.tsx
└── src-tauri/              # Rust backend
    ├── src/
    │   ├── main.rs
    │   ├── lib.rs
    │   ├── commands/       # Tauri IPC handlers
    │   ├── orchestrator/   # Dual-agent pipeline
    │   ├── agents/         # Companion + Executor
    │   ├── ai/             # HTTP client, prompts
    │   └── db/             # SQLite schema + queries
    ├── Cargo.toml
    └── tauri.conf.json
```

---

## Frontend architecture

### Responsibilities

- Render chat UI and settings
- Invoke Tauri commands for send/receive
- Subscribe to streaming events (if implemented)
- Apply visual design tokens (see PRODUCT.md §16)
- Handle RTL via `dir` attribute and logical CSS properties

### Key components (planned)

| Component | Purpose |
|-----------|---------|
| `ChatScreen` | Main conversation view |
| `MessageList` | Scrollable message history |
| `MessageBubble` | User vs companion styling |
| `Composer` | Input and send |
| `SettingsPanel` | API config, preferences |
| `ExecutorPanel` | Optional collapsed executor activity |

### State

- Conversation messages loaded from Rust on mount
- Optimistic UI for user sends (optional v1)
- Settings from Rust/local storage

---

## Backend architecture

### Modules

| Module | Responsibility |
|--------|----------------|
| `commands` | Tauri IPC entry points |
| `orchestrator` | Routes messages through companion/executor pipeline |
| `agents::companion` | Companion prompts, intent classification, formatting |
| `agents::executor` | Task execution, planning, tool calls |
| `ai::client` | OpenAI-compatible HTTP client |
| `ai::prompts` | System prompts and templates |
| `db` | Migrations, CRUD for conversations and messages |

### Tauri commands (planned)

| Command | Input | Output |
|---------|-------|--------|
| `send_message` | `{ conversation_id, content }` | Stream or final companion message |
| `list_conversations` | — | Conversation summaries |
| `get_messages` | `{ conversation_id }` | Message list |
| `get_settings` | — | App settings |
| `update_settings` | Settings payload | OK / error |
| `clear_history` | `{ conversation_id? }` | OK |

Exact signatures will be defined during implementation.

---

## Data model

### SQLite schema (v1)

```sql
-- conversations
CREATE TABLE conversations (
    id          TEXT PRIMARY KEY,
    title       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

-- messages (user-visible thread)
CREATE TABLE messages (
    id               TEXT PRIMARY KEY,
    conversation_id  TEXT NOT NULL REFERENCES conversations(id),
    role             TEXT NOT NULL,  -- 'user' | 'companion'
    content          TEXT NOT NULL,
    created_at       TEXT NOT NULL
);

-- executor_runs (optional visibility)
CREATE TABLE executor_runs (
    id               TEXT PRIMARY KEY,
    conversation_id  TEXT NOT NULL REFERENCES conversations(id),
    message_id       TEXT REFERENCES messages(id),
    task_spec        TEXT NOT NULL,  -- JSON
    result           TEXT,           -- JSON or text
    status           TEXT NOT NULL,  -- 'pending' | 'running' | 'done' | 'error'
    created_at       TEXT NOT NULL,
    completed_at     TEXT
);

-- settings (key-value)
CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

---

## AI integration

### OpenAI-compatible client

- Configurable `base_url`, `api_key`, and model names
- Separate model config for companion (higher temperature) and executor (lower temperature)
- Standard chat completions API

### Task specification (companion → executor)

Structured JSON passed between agents. Example shape:

```json
{
  "objective": "Create a 30-day freelancing starter plan",
  "context": "User is new to freelancing, unsure where to begin",
  "constraints": ["actionable steps", "beginner-friendly"],
  "expected_output": "step-by-step plan with weekly milestones"
}
```

Exact schema defined in [AGENTS.md](./AGENTS.md).

---

## Orchestration logic

1. **Persist** incoming user message.
2. **Companion pass** — send recent history + system prompt; companion returns either:
   - Direct reply (no execution), or
   - Reply stub + `TaskSpec` for executor.
3. **Executor pass** (if needed) — run with task spec; capture result and optional activity log.
4. **Companion format pass** — companion receives executor output; produces final user-facing message.
5. **Persist** companion message (and executor run record if applicable).
6. **Return** to frontend.

Intent classification may be explicit (companion outputs structured flag) or implicit (presence of `TaskSpec` in companion response). See [DECISIONS.md](./DECISIONS.md).

---

## Security

- API keys stored locally (OS keychain preferred; fallback encrypted or plain local settings for MVP)
- No telemetry by default
- Executor tools (future) run with user consent and sandboxing TBD
- Input sanitization before persistence and API calls

---

## Build and run

### Prerequisites

- Node.js LTS
- Rust stable
- Platform Tauri dependencies

### Commands

```bash
npm install
npm run tauri dev      # development
npm run tauri build    # production bundle
```

See [docs/development.md](./docs/development.md).

---

## Deployment

- **Windows:** `.msi` / NSIS installer
- **macOS:** `.dmg` / `.app`
- **Linux:** `.deb`, AppImage, or distro-specific packages

Distribution via GitHub Releases for open-source builds.

---

## Testing strategy

| Layer | Approach |
|-------|----------|
| Rust | Unit tests for orchestrator, task spec parsing, DB |
| Frontend | Component tests for chat UI |
| Integration | Mock AI client for pipeline tests |
| E2E | Optional; manual QA for v1 |

---

## Related documents

- [AGENTS.md](./AGENTS.md) — agent behavior and prompts
- [PRODUCT.md](./PRODUCT.md) — UX and scope
- [DECISIONS.md](./DECISIONS.md) — architectural choices
- [ROADMAP.md](./ROADMAP.md) — implementation phases
