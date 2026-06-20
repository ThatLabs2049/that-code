# Contributing to Muse

Thank you for interest in Muse. This project is open source and welcomes thoughtful contributions.

---

## Before you start

1. Read [README.md](../README.md) for project overview.
2. Read [PRODUCT.md](../PRODUCT.md) to understand the companion-first vision.
3. Check [ROADMAP.md](../ROADMAP.md) for current version scope — prefer v1.0.0 milestone work unless discussed.
4. Review [ARCHITECTURE.md](../ARCHITECTURE.md) and [AGENTS.md](../AGENTS.md) for technical context.

---

## Ways to contribute

- **Code** — features, fixes, tests aligned with roadmap
- **Documentation** — improve clarity, fix errors, add examples
- **Issues** — bug reports, UX feedback, design discussion
- **Review** — thoughtful PR reviews

---

## Development workflow

1. Fork and clone the repository.
2. Create a branch from `main`:

   ```bash
   git checkout -b feat/short-description
   ```

3. Follow [development.md](./development.md) for local setup.
4. Make focused changes — one concern per PR when possible.
5. Run tests and lint locally.
6. Open a pull request with a clear description and test plan.

---

## Code guidelines

Aligned with [`.cursor/rules/project.mdc`](../.cursor/rules/project.mdc):

- **Minimal scope** — do not refactor unrelated code in the same PR
- **Local-first** — respect SQLite and on-device storage assumptions
- **RTL-safe** — use logical CSS; test bidirectional text on UI changes
- **No dead code** — remove unused imports and functions
- **Dependencies** — justify new crates/npm packages in PR description
- **Agents** — prompt changes need companion + executor QA notes

### Rust

- Run `cargo fmt` and `cargo clippy` before submitting
- Add unit tests for orchestrator and parsing logic

### Frontend

- Match existing component patterns
- Follow visual design in PRODUCT.md §16
- Accessibility: labels, contrast, reduced motion

---

## Commit messages

Use clear, imperative subjects:

```
Add companion direct-response parsing
Fix RTL message bubble padding
Document TaskSpec schema in AGENTS.md
```

---

## Pull request checklist

- [ ] Change aligns with ROADMAP current milestone (or explain why not)
- [ ] Tests pass (`cargo test`, frontend lint/typecheck)
- [ ] Documentation updated if behavior or APIs changed
- [ ] UI changes checked for RTL and accessibility
- [ ] No secrets or API keys in diff

---

## Reporting bugs

Include:

- OS and Muse version (or commit hash)
- Steps to reproduce
- Expected vs actual behavior
- Screenshots if UI-related
- Redacted logs if executor/API related

---

## Feature requests

Explain:

- User problem (companion experience angle)
- Proposed behavior
- Which roadmap version it fits (or why it should be reprioritized)

---

## Architecture changes

Significant design changes should add or update an ADR in [DECISIONS.md](../DECISIONS.md) and be discussed in an issue first when possible.

---

## Code of conduct

Read [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md). Be respectful, constructive, and inclusive in all project spaces.

---

## License

By contributing, you agree your contributions will be licensed under the [MIT License](../LICENSE).

---

## Questions

Open a GitHub Discussion or Issue once the repository is public. For agent behavior questions, reference [AGENTS.md](../AGENTS.md).
