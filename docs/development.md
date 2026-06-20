# Development Guide

How to set up, build, and work on Muse locally.

---

## Repository layout

```
/
├── src/                 # React + TypeScript frontend
├── src-tauri/           # Rust backend (Tauri)
├── .github/workflows/   # CI and release
├── docs/                # Supplementary documentation
├── CHANGELOG.md
└── …                    # PRODUCT.md, ARCHITECTURE.md, etc.
```

---

## Initial setup

```bash
git clone https://github.com/<org>/muse.git
cd muse
npm ci
```

Ensure Rust is installed:

```bash
rustc --version
cargo --version
```

---

## Development server

```bash
npm run tauri dev
```

---

## Quality checks (run before PR)

```bash
npm run build          # TypeScript + Vite
npm run test:rust      # cargo test
npm run lint:rust      # cargo clippy -D warnings
```

---

## Production build

```bash
npm run tauri build
```

Artifacts: `src-tauri/target/release/bundle/`

---

## Backend (Rust)

| Directory | Purpose |
|-----------|---------|
| `commands/` | Tauri IPC handlers |
| `orchestrator/` | Dual-agent pipeline |
| `agents/` | Companion and executor |
| `ai/` | HTTP client and prompts |
| `db/` | SQLite persistence |

```bash
npm run test:rust
npm run lint:rust
```

---

## Frontend (React)

```bash
npm run build
npm run typecheck
```

---

## Local AI stack (v2.0)

Muse works with any **OpenAI-compatible** chat API. Recommended local setup:

| Role | Example | Notes |
|------|---------|--------|
| Chat (Luna + executor) | [Ollama](https://ollama.com) `http://localhost:11434/v1` | No API key required for localhost |
| Embeddings (RAG) | Ollama `nomic-embed-text` or same provider as chat | Enable RAG in Settings after picking a workspace |
| Models | `llama3`, `qwen2.5-coder`, `gpt-4o-mini` (cloud) | Use a faster model for executor if latency matters |

**Quick Ollama path**

1. Install Ollama and pull models: `ollama pull llama3.2` and `ollama pull nomic-embed-text`
2. In Muse Settings → API base URL: `http://localhost:11434/v1`
3. Companion model: `llama3.2` (or your pull name) — Executor: same or a coder-tuned model
4. Pick a **project folder** under Workspace
5. Optional: **Index workspace** for RAG; enable **verify loop** and **context pack**

**Function calling:** OpenAI, Azure OpenAI, and recent Ollama models can use native executor `tool_calls`. Providers that reject the `tools` parameter fall back to JSON tool actions automatically.

**Agent bench (developers):** deterministic smoke tests against `src-tauri/tests/fixtures/rust-calc`:

```bash
cargo test bench:: --manifest-path src-tauri/Cargo.toml
```

**Live LLM eval (optional, not CI):** five read-only executor scenarios against the same fixture:

```bash
# PowerShell
$env:MUSE_EVAL_LIVE="1"
$env:MUSE_API_BASE="http://localhost:11434/v1"
$env:MUSE_MODEL="llama3.2"
cargo test live_eval --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture
```

Set `MUSE_API_KEY` for cloud providers. CI can opt in with a secret and `MUSE_EVAL_LIVE=1` on a scheduled job.

---

## CI

[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) runs on push/PR to `main`:

- `npm ci` + `npm run build`
- `cargo test` + `cargo clippy -D warnings` (Ubuntu)

---

## Releases

[`.github/workflows/release.yml`](../.github/workflows/release.yml) runs when a `v*` tag is pushed. It builds draft GitHub Release assets for Windows, macOS, and Linux using [tauri-action](https://github.com/tauri-apps/tauri-action).

Bundle targets include **installers** (NSIS, MSI, deb, dmg) and **portable** artifacts (standalone binary / AppImage).

### Workspace (v1.5)

Pick a **project folder** in Settings → Workspace before asking Luna to read or write files. The executor only accesses that folder; paths are validated in Rust.

---

## Related

- [getting-started.md](./getting-started.md)
- [contributing.md](./contributing.md)
- [ARCHITECTURE.md](../ARCHITECTURE.md)
