# Development Guide

How to set up, build, and work on **ThatCode** locally.

**Release target (v1):** Windows — NSIS + MSI + portable zip on GitHub Releases.

---

## Repository layout

```
/
├── src/                 # React + TypeScript frontend
├── src-tauri/           # Rust backend (Tauri)
├── .github/workflows/   # CI (Ubuntu) + release (Windows)
├── docs/                # Product, trust, temp pivot plans
└── scripts/             # e.g. generate-sha256.ps1
```

---

## Initial setup

```bash
git clone https://github.com/Satan2049/that-code.git
cd that-code
npm ci
```

Ensure Rust is installed:

```bash
rustc --version
cargo --version
```

Install [Tauri prerequisites for Windows](https://tauri.app/start/prerequisites/).

---

## Development server

```bash
npm run tauri dev
```

---

## Quality checks (run before PR)

```bash
npm run build          # TypeScript + Vite
npm run typecheck
npm run test:rust      # cargo test
npm run lint:rust      # cargo clippy -D warnings
```

---

## Production build (Windows)

```bash
npm run tauri build
```

Artifacts: `src-tauri/target/release/bundle/` (NSIS + MSI in `tauri.conf.json`). Add portable zip to the release manually if needed.

```powershell
.\scripts\generate-sha256.ps1
```

Compare output with [SHA256.txt](../SHA256.txt) before tagging.

---

## Backend (Rust)

| Directory | Purpose |
|-----------|---------|
| `commands/` | Tauri IPC handlers |
| `orchestrator/` | Message pipeline (`run_turn`) |
| `agents/` | Executor tool loop, tier profiles |
| `tools/` | Workspace sandbox, verify, git |
| `ai/` | HTTP client and prompts |
| `db/` | SQLite persistence |

---

## Local AI stack

ThatCode works with any **OpenAI-compatible** chat API.

| Role | Example |
|------|---------|
| Chat | [Ollama](https://ollama.com) `http://localhost:11434/v1` |
| Embeddings (RAG) | Ollama `nomic-embed-text` |
| Cloud | OpenAI, Azure, compatible proxies |

**Agent bench (no API key):**

```bash
cargo test bench:: --manifest-path src-tauri/Cargo.toml
```

**Live LLM eval (optional):**

```bash
$env:THATCODE_EVAL_LIVE="1"
$env:THATCODE_API_BASE="http://localhost:11434/v1"
$env:THATCODE_MODEL="llama3.2"
cargo test live_eval --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture
```

Legacy `MUSE_EVAL_*` env names still work.

---

## CI

[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) — build + Rust tests on Ubuntu (push/PR).

---

## Releases

[`.github/workflows/release.yml`](../.github/workflows/release.yml) — **Windows only** when a `v*` tag is pushed. Attaches **`SHA256.txt`**.

---

## Planning

- [docs/temp/thatcode-todo.md](./temp/thatcode-todo.md) — phase checklist
- [ROADMAP.md](../ROADMAP.md) — milestones including Phase 9

---

## Related

- [ARCHITECTURE.md](../ARCHITECTURE.md)
- [CONTRIBUTING.md](../CONTRIBUTING.md)
