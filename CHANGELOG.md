# Changelog

All notable changes to Muse are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.1.0] - 2025-06-19

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
- **Dual-agent pipeline** — companion delegates structured tasks to a hidden executor
- **OpenAI-compatible API** — configurable base URL, models, and temperatures
- **Local persistence** — SQLite conversations, messages, and settings
- **Settings UI** — API key, models, test connection, executor visibility, clear history
- **Executor activity panel** — optional collapsed view when tasks are delegated
- **English and Persian UI** — RTL layout with `fa` locale
- **Accessibility** — skip link, focus trap, keyboard navigation, reduced motion
- **Desktop builds** — Windows, macOS, and Linux via Tauri 2

### Technical

- Tauri 2 + React 19 + TypeScript frontend
- Rust backend with `rusqlite`, `reqwest`, dual-agent orchestrator
- MIT License

[2.1.0]: https://github.com/Satan2049/muse/releases/tag/v2.1.0
[2.0.0]: https://github.com/Satan2049/muse/releases/tag/v2.0.0
[1.5.0]: https://github.com/Satan2049/muse/releases/tag/v1.5.0
[1.0.0]: https://github.com/Satan2049/muse/releases/tag/v1.0.0
