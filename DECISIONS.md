# Muse — Architectural Decision Records

This document records significant technical and product decisions. Format: **ADR-NNN — Title**.

---

## ADR-001 — Dual-agent architecture (Companion + Executor)

**Status:** Accepted  
**Date:** 2025 (project inception)

### Context

A single LLM call struggles to be both emotionally engaging (high temperature, personality) and reliably structured (low temperature, planning, tools). Combining both in one agent produces either bland companions or unreliable task output.

### Decision

Split responsibilities into two agents:

1. **Companion** — user-facing, high temperature, personality (Luna in v1)
2. **Executor** — hidden, low temperature, structured task completion

The user sees one conversation; orchestration is internal.

### Consequences

- More API calls per delegated task (companion → executor → companion format)
- Clearer prompt design and testing boundaries
- Optional executor visibility without a second chat product surface

---

## ADR-002 — Tauri for desktop shell

**Status:** Accepted

### Context

Need a cross-platform desktop app with native feel, small bundle size, and Rust backend for orchestration and SQLite.

### Decision

Use **Tauri 2** with React/TypeScript frontend and Rust backend.

### Alternatives considered

| Option | Why not (for v1) |
|--------|------------------|
| Electron | Larger bundle; Node backend less ideal for Rust-native orchestration |
| Pure web app | Loses desktop integration; local-first story weaker |
| Flutter / .NET MAUI | Team stack preference for React + Rust |

### Consequences

- Rust learning curve for contributors
- Strong local-first and performance characteristics
- Platform-specific build requirements in CI

---

## ADR-003 — Local-first with SQLite

**Status:** Accepted

### Context

Conversations and settings should stay on the user's device by default. Privacy and offline-awareness are product values.

### Decision

Store conversations, messages, executor runs, and settings in **SQLite** via the Tauri Rust backend.

### Consequences

- No account system required for v1
- Backup/export features needed eventually
- Cloud sync deferred to v2.0 (opt-in)

---

## ADR-004 — OpenAI-compatible API layer

**Status:** Accepted

### Context

Users may use OpenAI, Azure OpenAI, local proxies (LiteLLM, etc.), or future local servers.

### Decision

Implement a single **OpenAI-compatible chat completions client** with configurable `base_url`, `api_key`, and models.

### Consequences

- Not locked to one vendor
- Some providers may have subtle API differences — document known-good providers
- Separate model/temperature config per agent

---

## ADR-005 — Single chat surface (no executor chat page)

**Status:** Accepted

### Context

Tools like Cursor expose agent thinking in dedicated UI. Muse targets companionship, not IDE workflow.

### Decision

One chat thread for the user. Executor activity is **optional** and shown in a lightweight collapsible panel — not a full second chat.

### Consequences

- Simpler UX aligned with product vision
- Executor output must be formatted by companion before display (default path)

---

## ADR-006 — Luna as sole v1 personality

**Status:** Accepted

### Context

Multiple personalities multiply prompt QA, UI, and settings complexity.

### Decision

Ship v1 with **one personality: Luna**. Multi-personality deferred to v1.1.

### Consequences

- Faster path to MVP
- Personality system should still be data-driven (prompts as config) to ease v1.1

---

## ADR-007 — English-first, RTL-safe

**Status:** Accepted

### Context

Primary audience is English-speaking; Persian and other RTL locales are supported requirements from project rules.

### Decision

- English-first copy and Luna prompts for v1
- UI must use logical CSS properties and `dir` support for RTL
- Test at least one RTL locale on touched screens

### Consequences

- Companion prompts may need localization pass in future versions
- Message bubbles and composer must not assume LTR-only layout

---

## ADR-008 — No avatars or voice in v1

**Status:** Accepted

### Context

Avatars and voice add asset pipelines, latency, and platform complexity.

### Decision

Exclude avatars and voice from v1. Personality is conveyed through language only. Voice in v1.3 per roadmap.

### Consequences

- Distinct message styling replaces visual character representation
- Companion identity relies on strong prompt design

---

## ADR-009 — Structured TaskSpec between agents

**Status:** Accepted

### Context

Free-text handoff from companion to executor is ambiguous and hard to validate.

### Decision

Companion emits a JSON **TaskSpec** (`objective`, `context`, `constraints`, `expected_output`) when delegating. Rust validates before executor call.

### Consequences

- Companion must be prompted for structured output in delegate mode
- Enables logging, testing, and future tool routing

---

## ADR-010 — Intent classification by companion (not separate classifier)

**Status:** Accepted

### Context

Could use a small classifier model or rules engine for routing.

### Decision

The **companion agent** decides direct vs delegate in the same pass (structured JSON response modes).

### Alternatives considered

- Separate routing model — extra latency and cost for v1
- Always invoke executor — breaks casual chat UX

### Consequences

- Companion prompt must be clear about when not to delegate
- Monitor over-delegation in QA

---

## ADR-011 — Open-source on GitHub

**Status:** Accepted

### Context

Project is intended as a public open-source companion app.

### Decision

Publish on GitHub with full architecture and agent documentation. Licensed under **MIT License**.

### Consequences

- Documentation-first contribution path
- Need CONTRIBUTING.md, CODE_OF_CONDUCT (future), and CI before community scale

---

## ADR-009 — Sandboxed workspace for executor tools (v1.5)

**Status:** Accepted  
**Date:** 2025-06-19

### Context

v1.0 executor only simulated steps via LLM JSON. Users need real file access for project tasks without granting system-wide access.

### Decision

- User picks **one workspace folder** in Settings (explicit consent)
- All executor file tools resolve paths under that root with canonicalization and `..` blocking
- No shell execution, no arbitrary network fetch in v1.5
- `write_file` overwrites require a Settings opt-in
- Tool calls logged in `executor_runs` / activity panel

### Consequences

- Windows-first path handling; macOS/Linux use same sandbox logic
- Companion must not delegate file tasks without workspace configured
- Additional Rust tests required for path sandbox

---

## ADR-010 — Responsive delegation via Tauri events (v1.5)

**Status:** Accepted  
**Date:** 2025-06-19

### Context

Delegated tasks felt slow: three sequential LLM calls with no feedback until completion.

### Decision

- Persist Luna's **holding message** immediately after intent, before executor runs
- Emit `executor-progress` Tauri events for phases: `holding`, `executing`, `formatting`
- Frontend shows phase-specific status and live activity log during the blocking `send_message` call
- Skip companion format pass when executor output is short and tool-based (template wrap)

### Consequences

- `send_message` may produce two companion messages (holding + final) on delegate path
- UI listens for events during invoke; direct chat unchanged

---

## Template for new ADRs

```markdown
## ADR-NNN — Title

**Status:** Proposed | Accepted | Superseded  
**Date:** YYYY-MM-DD

### Context
What is the issue?

### Decision
What did we decide?

### Alternatives considered
What else was evaluated?

### Consequences
What becomes easier or harder?
```

When adding ADRs, append to this file and reference in PR descriptions when relevant.
