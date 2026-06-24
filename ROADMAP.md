# ThatCode — Roadmap

> **Current release:** [v2.7.1](https://github.com/Satan2049/that-code/releases/tag/v2.7.1) (Windows).  
> Checklist: [`docs/temp/thatcode-todo.md`](./docs/temp/thatcode-todo.md).  
> Muse v2.x history below is kept for reference only.

---

## Shipped: **ThatCode v2.7.1** (Phases 0–7)

| Phase | Deliverable | Status |
|-------|-------------|--------|
| **0** | Decisions locked (`that-code` repo, `com.thatcode.app`) | **Done** |
| **1** | Rebrand metadata, docs, Windows release CI | **Done** |
| **2** | Single-agent backend | **Done** |
| **3** | Agent-first UI (chat + diffs) | **Done** |
| **4** | Windows ship (NSIS, MSI, portable, SHA256, VirusTotal) | **Done** |
| **5** | Codebase brain (RAG, @ context, rules) | **Done** |
| **6** | Agent loop parity (stream, plan, hunks) | **Done** |
| **7** | Power & depth (symbols, palette, git sidebar) | **Done** |
| **8** | macOS + Linux installers | **Next** |
| **9** | Autonomous developer (long missions) | **Later** |

---

## Phase 8 — Platform

- macOS + Linux installers (after Windows quality bar)
- Session export (markdown transcript + diffs)

---

## Phase 9 — Autonomous developer

Long missions with visible plan, self-correction on verify failures, checkpoints, and mission timeline. See [docs/temp/thatcode-todo.md](./docs/temp/thatcode-todo.md).

---

## Historical: Muse roadmap (frozen 2026-06-20)

> Muse companion / v2.2 track cancelled. Planning moved to ThatCode.

**v2.1** shipped (final Muse feature release). **ThatCode v2.7.1** supersedes the planned v3.0.0 semver label.

### Muse milestones (historical)

| Phase | Deliverable | Status |
|-------|-------------|--------|
| 0–8 | v1.0.0 MVP | Done |
| **1.5** | **Executor tools + responsive delegation** | **Done** |
| **1.6** | **Streaming, cancel, incremental RAG, UI refresh** | **Done** |
| **2.0** | **Reliable local coding agent** | **Done** |
| **2.1** | **Personalities, memory, git, queue, auto RAG, MCP** | **Done** |
| **2.2** | **Companion experience polish** | **Cancelled** → ThatCode |

### v2.0 feature checklist

- [x] Verify loop after mutating file tools
- [x] Change review panel with unified diff + revert actions
- [x] Context pack with Settings toggle
- [x] Schema-validated tools + native OpenAI function calling
- [x] Agent bench fixture + CI smoke tests

### v2.1 feature checklist

- [x] Personalities — Luna, Sage, Spark
- [x] Long-term memory
- [x] Git tools
- [x] Command allowlist
- [x] Task queue
- [x] Auto-index RAG
- [x] MCP client (minimal)

---

## How to use this roadmap

1. **Contributors:** Pick items from the current phase only unless agreed otherwise.
2. **Issues:** Tag with target version when relevant.
3. **Releases:** Update [CHANGELOG.md](./CHANGELOG.md), [SHA256.txt](./SHA256.txt), and [docs/TRUST.md](./docs/TRUST.md).
