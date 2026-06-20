# Architecture Overview

This page is a short entry point. The canonical architecture document is **[ARCHITECTURE.md](../../ARCHITECTURE.md)** at the repository root.

---

## Quick reference

### Dual-agent flow

```
User → Companion → [TaskSpec → Executor] → Companion → User
```

Bracketed steps run only when the companion delegates.

### Stack

- **Tauri** — desktop shell
- **React + TypeScript** — UI
- **Rust** — orchestration, SQLite, AI client
- **OpenAI-compatible API** — LLM provider

### Key documents

| Topic | Document |
|-------|----------|
| System design | [ARCHITECTURE.md](../../ARCHITECTURE.md) |
| Agent contracts | [AGENTS.md](../../AGENTS.md) |
| Decisions | [DECISIONS.md](../../DECISIONS.md) |
| Development | [development.md](../development.md) |

---

For diagrams, data models, module layout, and IPC commands, see the full [ARCHITECTURE.md](../../ARCHITECTURE.md).
