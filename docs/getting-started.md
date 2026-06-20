# Getting Started with Muse

This guide helps you understand Muse and prepare for local development.

---

## What is Muse?

Muse is a desktop AI companion. You chat with **Luna**, a warm and emotionally intelligent character. When your request needs structured planning or deep task work, a hidden **executor agent** handles it — and Luna presents the result naturally.

You always see one conversation.

---

## Prerequisites

Before running Muse locally:

| Requirement | Link |
|-------------|------|
| Node.js (LTS) | https://nodejs.org/ |
| Rust | https://www.rust-lang.org/tools/install |
| Tauri prerequisites | https://tauri.app/start/prerequisites/ |

Install platform-specific dependencies listed in the Tauri docs for Windows, macOS, or Linux.

---

## API setup

Muse uses an **OpenAI-compatible** chat API.

1. Obtain an API key from your provider (OpenAI, Azure, or a compatible proxy).
2. After launching Muse, open **Settings**.
3. Enter your API key and base URL (if not using OpenAI's default).
4. Optionally set different models for companion and executor.

Keys are stored **locally** on your device.

### Compatible providers (expected)

- OpenAI
- Azure OpenAI (with compatible endpoint)
- Local proxies (LiteLLM, etc.)
- Future: Ollama and local servers (v1.4)

---

## First conversation

1. Launch Muse.
2. Read Luna's greeting.
3. Chat naturally — casual messages stay with Luna only.
4. Ask for a plan or research task to trigger the executor pipeline.
5. Optionally enable **executor visibility** in settings to see behind-the-scenes activity.

---

## Project documentation map

| Document | When to read |
|----------|--------------|
| [README.md](../README.md) | Project overview |
| [PRODUCT.md](../PRODUCT.md) | UX, Luna, visual design |
| [ARCHITECTURE.md](../ARCHITECTURE.md) | How the system is built |
| [AGENTS.md](../AGENTS.md) | Companion and executor behavior |
| [development.md](./development.md) | Build and run from source |
| [contributing.md](./contributing.md) | Submit changes |

---

## Troubleshooting

| Issue | Suggestion |
|-------|------------|
| API errors | Verify key and base URL in settings |
| Empty responses | Check network; review executor visibility for errors |
| Build failures | Confirm Rust and Tauri prerequisites |
| RTL layout issues | Report with locale and screenshot |

For bugs and features, open a GitHub issue once the repository is public.
