# Contributing to ThatCode

Thank you for your interest in ThatCode. This project is open source and welcomes focused contributions.

Please read our [Code of Conduct](./CODE_OF_CONDUCT.md) before participating. By contributing, you agree your contributions are licensed under the [MIT License](./LICENSE).

**Status:** ThatCode v3 (single coding agent) — Phases 0–7 complete. See [docs/temp/thatcode-todo.md](./docs/temp/thatcode-todo.md) before large changes.

---

## Before you start

1. [README.md](./README.md) — overview and install
2. [ARCHITECTURE.md](./ARCHITECTURE.md) — stack and request flow
3. [ROADMAP.md](./ROADMAP.md) — milestones
4. [docs/development.md](./docs/development.md) — clone, build, test

---

## Development workflow

1. Fork and clone `https://github.com/Satan2049/that-code.git`
2. Branch from `main`: `git checkout -b feat/short-description`
3. Make focused changes; match existing style
4. Run quality checks before opening a PR:

```bash
npm run typecheck
npm run build
npm run test:rust
npm run lint:rust
```

5. Open a PR with a clear description and test plan

---

## Scope guidelines

- **Windows v1** — release target is NSIS + MSI; test UI on Win11 when possible
- **Minimal diffs** — avoid drive-by refactors unrelated to the PR
- **No new dependencies** unless justified in the PR
- **Agent-first** — UI changes should keep diffs, tool activity, and trust visible

---

## Reporting issues

Use GitHub Issues. Include OS version, ThatCode version, steps to reproduce, and relevant logs (no API keys).

---

## Security

See [SECURITY.md](./SECURITY.md) for responsible disclosure.
