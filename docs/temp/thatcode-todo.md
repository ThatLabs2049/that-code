# ThatCode — Build TODO

Action checklist for ThatCode. Long-term vision: autonomous missions (Phase 9) in [ROADMAP.md](../../ROADMAP.md).

**Platform v1:** Windows (NSIS + MSI + portable zip).  
**Current release:** **v2.7.1** — Phases 0–7 shipped.  
**North star:** *Devin-level autonomy on long tasks — Cursor-level clarity on every step. Local, BYO API.*

---

## Phase 8 — Platform (next)

- [ ] macOS + Linux installers (after Windows quality bar)
- [ ] Session export (markdown transcript + diffs)

---

## Phase 9 — Autonomous developer (Devin-like + Cursor clarity)

*Long missions: plan, decide, act, retry — with full visible audit trail. Not a black box.*

### 9.1 — Mission foundation

- [ ] Mission mode: user goal → decomposed plan → multi-step run
- [ ] Mission entity in DB (goal, plan, status, step count, blockers)
- [ ] Resume mission after app restart
- [ ] Step budget + wall-clock limit; graceful pause

### 9.2 — Think–act loop

- [ ] Replan between tool batches (not single TaskSpec per message)
- [ ] Progress narrative in UI (“Investigating…”, “Tests failed, retrying…”)
- [ ] Working memory per mission (hypotheses, attempts, files touched)

### 9.3 — Self-correction

- [ ] Verify/test failure → diagnose → patch → retry (bounded)
- [ ] Stream command/test output in activity timeline

### 9.4 — Trust & control

- [ ] Autonomy dial: supervised vs auto-run
- [ ] Risky-tool gates (push, delete, force) — require approval
- [ ] Checkpoints at milestones; rollback to checkpoint
- [ ] Mission timeline in sidebar (all steps, expandable)

### 9.5 — Scale out

- [ ] Subagent roles: explorer (read) / implementer (edit) / verifier (test)
- [ ] Mission inbox — queue missions; notify when blocked
- [ ] Multi-model routing (fast plan, strong edit)

**Phase 9 done when:** User assigns a multi-file bugfix mission; agent runs 10+ steps autonomously; user sees plan, context, diffs, and test output throughout; can pause or revert — all local.

---

## Quick reference — embeddings

Enable: **Settings → Power → Local memory (RAG)**

| Setting | Default |
|---------|---------|
| `rag_enabled` | `false` |
| `embedding_model` | `nomic-embed-text` |
| `embedding_base_url` | `http://localhost:11434/v1` |
| `rag_top_k` | `8` |
| `rag_auto_index` | `false` |

Manual: **Index workspace** / **Index changes** / test embeddings connection.

---

## Completed phases (archive)

<details>
<summary>Phases 0–7 + v2.7.1 release — done</summary>

- Phase 0 — decisions locked (ThatCode name, repo, tagline)
- Phase 1 — rebrand metadata, docs, CI, icons
- Phase 2 — single-agent backend
- Phase 3 — agent-first UI + screenshots
- Phase 4 — v2.7.1 Windows ship (NSIS, MSI, portable, SHA256, VirusTotal docs)
- Phase 5 — RAG, @ context, project rules
- Phase 6 — streaming, plan approval, hunk review, activity output
- Phase 7 — symbols, tiers, task queue, git sidebar, MCP gallery, Ctrl+K

</details>
