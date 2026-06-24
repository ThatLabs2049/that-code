# Changelog

All notable changes to ThatCode (formerly Muse) are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned

- macOS + Linux installers (Phase 8)
- Autonomous mission mode (Phase 9)

## [2.7.1] - 2026-06-24

First **ThatCode** Windows release after the Muse → single-agent pivot. Phases 0–7 are in this build.

### Added

- **ThatCode rebrand** — new identity, icons, and `com.thatcode.app` bundle id
- **Single coding agent** — one tool loop (companion/delegation removed)
- **Agent tiers** — auto, quick, standard, deep, explain with scout/editor routing
- **RAG** — local embeddings index, `@codebase`, `.thatcodeignore`, HNSW retrieval, index progress UI
- **@ context** — `@file`, `@folder`, `@symbol` attachments from composer
- **Project rules** — `.thatcode/rules.md` / `.cursorrules` injection
- **Plan-before-edit** — scout pauses for approval; persisted pending plans
- **Streaming replies** — ack + assistant stream events; final message returned in IPC result
- **Change review** — unified diffs, per-hunk reject, open-in-editor links
- **Command palette** — `Ctrl+K` shortcuts
- **Git sidebar** — status, branch, diff stats
- **Task queue** — visible sidebar queue for sequential agent tasks
- **MCP preset gallery** — optional stdio MCP tools (`mcp_*`)
- **Explore → implement** — read-only scout pass then editor phase
- **Portable zip** — alongside NSIS + MSI on GitHub Releases

### Changed

- Agent-first UI: chat + tool activity + changes sidebar
- Windows command runner resolves `npm.cmd` / `npx.cmd` for tool execution
- Casual chat uses direct completion even when a workspace is set
- Docs, landing page, and screenshots updated for ThatCode

### Fixed

- Final agent message missing after run completes (`assistantMessage` IPC + non-empty DB persist)
- Tool activity panel scroll and post-run visibility
- Agent parse fallbacks for models without native tool calling
- MCP spawn timeout; ack stream timeout; executor wall-clock limit
- Full-file rewrites blocked on existing files (must use `edit_file`)
- Plan/send race, MCP shell injection, pending plan restore on startup

### Security

- MCP server spawn via argv (no shell); session killed on drop
- Command allowlist + no shell pipes; timed-out child processes killed
- See [SHA256.txt](./SHA256.txt) and [docs/TRUST.md](./docs/TRUST.md) for release verification

## [2.1.0] - 2025-06-19

*Released as **Muse**.*

### Added

- **Personalities** — Luna, Sage, Spark with settings selector and localized greetings
- **Long-term memory** — SQLite CRUD, Settings UI, injected into companion + delegated tasks
- **Git tools** — `git_add`, `git_commit`, `git_checkout_branch` for the executor
- **Command allowlist** — user-editable extra prefixes in Settings
- **Task queue** — sequential delegated tasks (Settings toggle)
- **Auto-index RAG** — file watcher on workspace when enabled
- **MCP client (minimal)** — stdio MCP server spawns extra executor tools (`mcp_*`)
- **Live LLM eval** — optional `MUSE_EVAL_LIVE=1` harness (5 read-only scenarios on fixture repo)

## [2.0.0] - 2025-06-19

### Added

- **Verify loop** — auto-run `cargo test` / `npm test` (or configured command) after file edits; failures fed back to executor (max 3 retries)
- **Change review** — unified diff per executor run; revert one file or revert all from the chat UI
- **Context pack** — capped project tree + `git status --short` injected into delegated tasks (Settings toggle)
- **Structured tools** — OpenAI-style function definitions with schema validation; native `tool_calls` when the provider supports them, JSON fallback otherwise
- **Settings** — verify loop toggle, optional verify command, context pack toggle (English + Persian)
- **Agent bench fixture** — `src-tauri/tests/fixtures/rust-calc` with deterministic CI smoke tests (`cargo test bench::`)

### Changed

- Executor conversation history uses proper assistant/tool message roles for native function calling
- Version bumped to 2.0.0 across app manifests

## [1.5.0] - 2025-06-19

### Added

- **Responsive delegation** — Luna's holding message appears before executor finishes; phase progress via `executor-progress` events
- **Workspace** — user-selected project folder in Settings with native folder picker
- **Executor tools** — `list_dir`, `read_file`, `write_file`, `search_files` in a sandboxed agent loop (max 10 steps)
- **Live activity panel** — updates during tool execution when visibility is enabled
- **Settings** — workspace path, allow file overwrites toggle, executor model latency hint

### Changed

- Executor uses real tool loop instead of single-shot JSON simulation
- Bundle targets explicitly include NSIS, MSI, AppImage, deb, dmg (portable + installer)

## [1.0.0] - 2025-06-19

First public release of Muse — a local-first desktop AI companion with dual-agent architecture.

### Added

- **Luna** companion personality — warm, supportive, curious companion chat
- **Executor agent** — structured task pipeline with sandboxed tools
- **SQLite persistence** — conversations and settings on device
- **OpenAI-compatible API** — bring your own key and base URL
- **Dark UI** — ThatGPT-family calm aesthetic
- **Windows, macOS, Linux** — Tauri 2 desktop builds

[Unreleased]: https://github.com/Satan2049/that-code/compare/v2.7.1...HEAD
[2.7.1]: https://github.com/Satan2049/that-code/releases/tag/v2.7.1
[2.1.0]: https://github.com/Satan2049/that-code/releases/tag/v2.1.0
