<p align="center">
  <img src="public/muse-logo.svg" alt="Muse logo" width="72" height="72" />
</p>

<h1 align="center">Muse</h1>

<p align="center"><em>in the world of musing</em></p>

<p align="center"><strong>Characters should not only respond. Characters should live.</strong></p>

<p align="center">
  <a href="https://github.com/Satan2049/muse/actions/workflows/ci.yml"><img src="https://github.com/Satan2049/muse/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/Satan2049/muse/releases/latest"><img src="https://img.shields.io/github/v/release/Satan2049/muse?label=release" alt="Latest release" /></a>
  <a href="./LICENSE"><img src="https://img.shields.io/github/license/Satan2049/muse" alt="MIT License" /></a>
</p>

<p align="center">
  <a href="https://Satan2049.github.io/muse/">Landing page</a> ·
  <a href="https://github.com/Satan2049/muse/releases/latest">Download</a> ·
  <a href="./docs/TRUST.md">Verify downloads</a> ·
  <a href="./SECURITY.md">Security</a>
</p>

---

Muse is an open-source **desktop AI companion** built with Tauri. You chat with warm, emotionally intelligent characters — starting with **Luna** — while a separate execution agent handles planning, research, and workspace tools behind the scenes.

The result feels personal and engaging, not like a generic task assistant.

---

## Features

- **Luna & personalities** — Luna, Sage, and Spark; companion-first single chat thread
- **Dual-agent orchestration** — companion delegates; executor runs structured work
- **Local-first** — SQLite conversations, settings, and memory on your device
- **OpenAI-compatible APIs** — your provider, your models
- **English & Persian UI** — RTL-safe layout and bidirectional text
- **Settings** — API key, models, workspace, personalities, memory, executor options
- **Optional executor panel** — activity log for power users without a second chat voice
- **Desktop builds** — Windows (NSIS, MSI), macOS (DMG), Linux (deb, AppImage)

> Screenshots: capture from the app and add to [`docs/screenshots/`](./docs/screenshots/). Placeholder mockups appear on the [landing page](./docs/index.html) until then.

---

## Screenshots

| Chat (placeholder) | Settings (placeholder) |
|--------------------|------------------------|
| Branded preview on [docs/index.html](./docs/index.html) | Add `docs/screenshots/settings.png` when ready |

Maintainers: see [`docs/screenshots/README.md`](./docs/screenshots/README.md) for capture guidelines.

---

## Installation

1. Download the installer for your platform from **[GitHub Releases](https://github.com/Satan2049/muse/releases/latest)** — the only trusted source.
2. Open **`SHA256.txt`** on the release page and verify your download before installing. See **[docs/TRUST.md](./docs/TRUST.md)** for step-by-step instructions (PowerShell, macOS, Linux) and VirusTotal notes.
3. Run the installer and configure your API provider in Settings.

---

## Development

**Prerequisites:** [Node.js](https://nodejs.org/) (LTS), [Rust](https://www.rust-lang.org/tools/install), and [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```bash
npm install
npm run tauri dev
```

Full setup: **[docs/development.md](./docs/development.md)**

---

## Build

```bash
npm run tauri build
```

Installers and bundles are written to **`src-tauri/target/release/bundle/`**.

Generate a checksum manifest for release assets:

```powershell
.\scripts\generate-sha256.ps1
```

---

## Tech stack

| Layer | Technology |
|-------|------------|
| Desktop shell | [Tauri 2](https://tauri.app/) |
| Frontend | React, TypeScript |
| Backend | Rust (Tauri) |
| AI | OpenAI-compatible APIs |
| Storage | SQLite |

---

## Documentation

| Document | Purpose |
|----------|---------|
| [PRODUCT.md](./PRODUCT.md) | Vision, UX, Luna personality, visual design |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System design, stack, data flow |
| [AGENTS.md](./AGENTS.md) | Companion and Executor agent specifications |
| [ROADMAP.md](./ROADMAP.md) | Version plan |
| [DECISIONS.md](./DECISIONS.md) | Architectural decision records |
| [CHANGELOG.md](./CHANGELOG.md) | Release history |
| [docs/TRUST.md](./docs/TRUST.md) | Download verification & VirusTotal transparency |
| [docs/contributing.md](./docs/contributing.md) | Contributor guide |
| [docs/development.md](./docs/development.md) | Local development |
| [docs/index.html](./docs/index.html) | GitHub Pages landing (enable Pages → `/docs`) |

---

## Contributing

Contributions are welcome. Start with **[CONTRIBUTING.md](./CONTRIBUTING.md)** (points to the full guide) and read **[CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md)** before opening a pull request.

---

## Security

Report vulnerabilities privately via **[SECURITY.md](./SECURITY.md)** (GitHub Security Advisories preferred). Download verification is covered separately in **[docs/TRUST.md](./docs/TRUST.md)**.

---

## Publishing a release (maintainers)

1. Update [CHANGELOG.md](./CHANGELOG.md)
2. Bump version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
3. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`
4. The [release workflow](.github/workflows/release.yml) builds draft assets for Windows, macOS, and Linux, merges **`SHA256.txt`**, and attaches it to the release
5. Review the draft on GitHub and publish

---

## License

[MIT](./LICENSE) — Copyright (c) 2026 Muse Contributors

---

## Project structure

```
/
├── README.md
├── SECURITY.md
├── CONTRIBUTING.md
├── AGENTS.md
├── ARCHITECTURE.md
├── PRODUCT.md
├── docs/
│   ├── index.html          # GitHub Pages landing
│   ├── TRUST.md
│   └── contributing.md
├── scripts/
│   └── generate-sha256.ps1
├── src/                    # React frontend
└── src-tauri/              # Rust backend
```
