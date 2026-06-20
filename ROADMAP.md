# Muse — Roadmap

This roadmap tracks planned versions from MVP through the multi-agent ecosystem vision. Dates are tentative and will be updated as the project progresses.

---

## Current focus: v2.1 shipped → v2.2 in progress

**v2.1** adds personalities, memory, git workflow tools, task queue, auto RAG, and minimal MCP. **v2.2** polishes companion UX: onboarding, personality depth, markdown messages, tabbed settings, theme accents, empty states.

### Milestones

| Phase | Deliverable | Status |
|-------|-------------|--------|
| 0–8 | v1.0.0 MVP | Done |
| **1.5** | **Executor tools + responsive delegation** | **Done** |
| **1.6** | **Streaming, cancel, incremental RAG, UI refresh** | **Done** |
| **2.0** | **Reliable local coding agent** | **Done** |
| **2.1** | **Personalities, memory, git, queue, auto RAG, MCP** | **Done** |
| **2.2** | **Companion experience polish (UX, theme, personalities, UI)** | **In progress** |

### v1.0.0 feature checklist

- [x] Tauri desktop app (Windows, macOS, Linux)
- [x] Single chat screen
- [x] Luna companion personality
- [x] Dual-agent pipeline (companion → executor → companion)
- [x] Local conversation history (SQLite)
- [x] OpenAI-compatible API configuration
- [x] Basic planning pipeline (TaskSpec → executor → formatted reply)
- [x] Optional executor activity visibility
- [x] Dark-first UI per PRODUCT.md §16
- [x] GitHub Actions CI + release workflow

### v2.0 feature checklist

- [x] Verify loop after mutating file tools (auto-detect or configured command)
- [x] Change review panel with unified diff + revert actions
- [x] Context pack (project tree + git status) with Settings toggle
- [x] Schema-validated tools + native OpenAI function calling with JSON fallback
- [x] Agent bench fixture + CI smoke tests (no API key required)

### v2.1 feature checklist

- [x] Personalities — Luna, Sage, Spark with settings selector and localized greetings
- [x] Long-term memory — CRUD in Settings, injected into companion + delegated tasks
- [x] Git tools — `git_add`, `git_commit`, `git_checkout_branch`
- [x] Command allowlist — user-editable extra prefixes in Settings
- [x] Task queue — sequential delegated tasks when enabled
- [x] Auto-index RAG — file watcher on workspace changes
- [x] MCP client (minimal) — stdio server, `mcp_*` tools merged into executor
- [x] Live LLM eval harness — optional `MUSE_EVAL_LIVE=1` (5 read-only scenarios)

### v1.5 feature checklist

- [x] Luna holding message persisted and shown before executor finishes
- [x] Phase-aware progress (understanding → executing → formatting) via Tauri events
- [x] Live executor activity updates during tool loop
- [x] Workspace folder picker in Settings (sandboxed)
- [x] Executor tools: `list_dir`, `read_file`, `write_file`, `search_files`
- [x] Path sandbox with traversal blocking + tests
- [x] Optional skip of companion format pass for simple tool results
- [x] Portable + installer bundle targets (NSIS, MSI, AppImage, deb, dmg)

### Explicitly deferred from v1.0.0

- Voice
- Avatars
- ~~Multiple personalities~~ — shipped in v2.1 (Luna, Sage, Spark)
- ~~Long-term memory graph~~ — basic CRUD memory shipped in v2.1; graph/decay still future
- Plugin / tool marketplace
- Complex RAG
- Cloud sync
- Mobile apps

---

## v1.1 — Multiple personalities ✅ (shipped in v2.1)

**Goal:** Let users choose from a small set of companion characters.

### Delivered in v2.1

- Personality selector in settings
- Separate system prompts per personality (Luna, Sage, Spark)
- Shared executor (neutral task layer)
- Personality-specific greeting and tone rules

### Considerations

- Each personality needs consistent behavioral testing
- UI updates for personality indicator (no avatars required)

---

## v1.2 — Long-term memory ✅ (core shipped in v2.1)

**Goal:** Remember important facts and preferences across sessions.

### Delivered in v2.1

- User-visible memory list (read, add, delete)
- Companion and executor receive relevant memories
- Local-only storage (SQLite `memories` table)

### Still open

- Automatic memory extraction from conversations
- Memory edit UI (update via API only today)

### Considerations

- Privacy controls — user owns and can wipe memory
- Avoid over-storing sensitive data without consent

---

## v1.3 — Voice conversations

**Goal:** Speak with the companion naturally.

### Planned features

- Speech-to-text input
- Text-to-speech for companion responses
- Push-to-talk or continuous mode (TBD)
- Voice-optimized companion prompts

### Considerations

- Platform-specific audio APIs via Tauri
- Latency and streaming UX
- Accessibility: voice must not replace text-only path

---

## v1.4 — Local models

**Goal:** Run models locally for privacy and offline use.

### Planned features

- Ollama / llama.cpp / compatible local server integration
- Model selection per agent (companion vs executor)
- Graceful fallback when local model unavailable
- Performance guidance for hardware requirements

### Considerations

- Smaller local models may need simplified executor tasks
- Clear UX when local model quality differs from cloud

---

## v2.0 — Multi-agent ecosystem

**Goal:** Expand Muse into a platform for characters, memory, and tools.

### Planned features

- User-created personalities (prompt editor, traits, rules)
- Advanced memory system (graph, relationships, decay)
- Multi-agent coordination beyond companion + executor
- Tool marketplace — community and bundled tools
- Plugin architecture for third-party extensions
- Optional cloud sync (opt-in)

### Considerations

- Security sandbox for tools and plugins
- Moderation for shared personality marketplace
- Migration path from v1 SQLite schema

---

## How to use this roadmap

1. **Contributors:** Pick items from the current version milestone only unless agreed otherwise.
2. **Issues:** Tag with target version (`v1.0`, `v1.1`, etc.).
3. **PRs:** Reference roadmap items in description when applicable.
4. **Updates:** Edit this file when scope or priority changes; note rationale in [DECISIONS.md](./DECISIONS.md) if significant.

---

## Success metrics (v1.0.0)

Qualitative:

- Conversations feel companion-like, not assistant-like
- Executor delegation is invisible unless user opts in
- App installs and runs on all three desktop platforms

Quantitative (post-release):

- GitHub stars and issue engagement
- Successful install → first conversation completion rate
- Contributor PRs merged

---

## Related documents

- [PRODUCT.md](./PRODUCT.md) — scope and success criteria
- [ARCHITECTURE.md](./ARCHITECTURE.md) — technical design
- [DECISIONS.md](./DECISIONS.md) — why we build it this way
