# Muse — Implementation Plan

Working plan after **v2.1.0**. Supersedes the v1.0 MVP phase list.

---

## Current state (v2.1.0)

Muse is a **local-first Tauri desktop app** with:

- **Companion personalities** — Luna, Sage, Spark; shared executor, different tone
- **Luna** (default) — warm chat, delegation, streamed plan + final replies
- **Executor** — bounded tool loop (25 steps) in a sandboxed workspace; **cancel** mid-run
- **Tools** — full workspace tool set + git tools + allowlisted commands; **schema-validated** tool calls; native function calling with JSON fallback
- **Verify loop** — auto-run tests/build after file edits (configurable command, max 3 retries)
- **Change review** — unified diff per run; revert file or revert all from UI
- **Local RAG** — full + incremental index; optional **auto-index** via file watcher
- **Long-term memory** — user CRUD in Settings; injected into companion + delegated tasks
- **Context pack** — project tree + git status injected into delegated tasks
- **Task queue** — sequential delegated jobs when enabled
- **MCP (minimal)** — stdio MCP server merges extra `mcp_*` tools into executor
- **Settings** — provider, models, workspace, RAG, verify, context pack, personalities, memory, MCP
- **UI refresh (1.6)** — warm ambient layout, glass surfaces, companion presence, status bar
- **i18n** — English + Persian, RTL-safe layout
- **Agent bench** — deterministic fixture tests in CI; optional live LLM eval (`MUSE_EVAL_LIVE=1`)

---

## How far from a serious coding agent?

**Short answer:** Muse ships **Tier 2** basics plus **Tier 3** workflow hooks (git, MCP, memory) — roughly **50–55%** of Cursor Agent / Devin-class capability for local repos. Still not production-grade without richer MCP, eval in CI, and AST-level context.

### What we have (strengths)

| Area | Muse today |
|------|------------|
| Architecture | Clean dual-agent split; companion UX + hidden executor |
| Local-first | SQLite history, settings, RAG vectors, memories — no cloud required |
| Workspace sandbox | Path traversal blocked; relative paths only |
| Tool loop | Multi-step executor with progress events; native tools + JSON fallback |
| Personality | Multiple companions; Luna explains intent before acting |
| Memory | Workspace RAG + user long-term memory table |
| Git workflow | stage, commit, checkout branch (sandboxed) |
| Extensibility | Minimal MCP stdio client |

### Critical gaps vs serious coding agents

| Gap | Why it matters |
|-----|----------------|
| **Limited terminal** | Allowlisted one-shot commands only; no interactive shell |
| **Weak context** | RAG + context pack (tree/git); no AST or imports graph yet |
| **MCP maturity** | Single stdio server; no multi-server UI or robust recovery |
| **Single conversation** | No project threads or parallel sub-agents |
| **Model-dependent reliability** | Live eval optional; CI still mostly deterministic |

### Maturity tiers (target)

```
Tier 0 — Chat companion          ✅ Done (v1.0)
Tier 1 — Tool-using agent        ✅ Done (v1.5)
Tier 2 — Reliable local coder    ✅ Done (v2.0)
Tier 3 — IDE-class agent         Partial (v2.1 — git, MCP, memory)
Tier 4 — Multi-agent ecosystem   ROADMAP v3+
```

---

## Priority tracks — status

### P0 — Trust & reliability (v2.0) ✅

1. **Verify loop** — Done
2. **Change review** — Done
3. **Structured tools** — Done (schema + native function calling)
4. **Agent bench** — Done (fixture + CI)
5. **Live LLM eval** — Done (optional `MUSE_EVAL_LIVE=1`, 5 scenarios)

### P1 — Context & speed ✅

- **Incremental RAG** — Done (1.6)
- **Context pack** — Done (2.0)
- **Streaming + cancel** — Done (1.6)
- **Personalities (v1.1)** — Done (2.1)
- **Long-term memory (v1.2 core)** — Done (2.1)

### P2 — Power user features ✅

- **Git tools** — Done (2.1)
- **Command allowlist settings** — Done (2.1)
- **Task queue** — Done (2.1)
- **Auto-index RAG** — Done (2.1)
- **MCP server support (minimal)** — Done (2.1)

### P3 — Next (v2.2+)

- Richer MCP (multi-server, UI, error recovery)
- Memory extraction from chat
- AST / import graph context
- Plugin marketplace
- Scheduled live eval in CI with secret

---

## Phases (updated)

| Phase | Deliverable | Status |
|-------|-------------|--------|
| 0–8 | v1.0.0 MVP | Done |
| 1.5 | Executor tools + delegation UX + minimal RAG | Done |
| 1.5.1 | Connection test fixes | Done |
| 1.6 | P1 UX + incremental RAG + UI refresh | Done |
| 2.0 | Reliable local coding agent | Done |
| **2.1** | **Personalities, memory, git, queue, auto RAG, MCP** | **Done** |
| 2.2 | MCP polish, memory extraction, eval CI | Planned |

---

## Architecture notes (unchanged core)

```
User → Companion (personality) → TaskSpec + plan message
              ↓
         RAG + memory enrich (optional)
              ↓
         Executor tool loop (sandbox + MCP tools)
              ↓
         Companion format pass → user
```

Keep companion as the only default voice. Executor stays neutral and tool-focused.

---

## References

- [PRODUCT.md](../../PRODUCT.md)
- [ARCHITECTURE.md](../../ARCHITECTURE.md)
- [AGENTS.md](../../AGENTS.md)
- [ROADMAP.md](../../ROADMAP.md)
- [todo.md](./todo.md)
