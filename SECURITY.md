# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| **2.7.1** ([latest release](https://github.com/Satan2049/that-code/releases/tag/v2.7.1)) | Yes |
| Older releases | No — upgrade to v2.7.1 |
| Development builds from `main` | Best-effort only; not a supported distribution channel |

## Reporting a vulnerability

If you believe you have found a security vulnerability in ThatCode, please report it **privately** so we can investigate before public disclosure.

**Preferred:** [GitHub Security Advisories](https://github.com/Satan2049/that-code/security/advisories/new) (Private vulnerability report).

**Alternative:** Email **mohammad161186@gmail.com** with a clear description, steps to reproduce, and impact assessment.

Please allow reasonable time for a fix before public discussion. We will acknowledge receipt and keep you updated on remediation.

## What to report

- Remote code execution, sandbox escapes, or path traversal in workspace tools
- SQL injection or unsafe deserialization in the Rust backend
- Authentication or secret-handling flaws in the application (not user misconfiguration)
- Cross-site or injection issues in the Tauri webview surface
- Dependency vulnerabilities with a demonstrated exploit path in ThatCode

## What not to report here

- **API keys stored in local Settings** — ThatCode is local-first; keys stay on your device by design. Rotate compromised keys with your provider.
- **Third-party API or model provider outages** — report those to the provider.
- **Social engineering or phishing** using the ThatCode name outside our official GitHub Releases page.
- **General product feedback or feature requests** — use [GitHub Issues](https://github.com/Satan2049/that-code/issues) instead.

## Download verification

To verify that a release file matches the official build, see [docs/TRUST.md](./docs/TRUST.md). That guide covers SHA256 checksums ([SHA256.txt](./SHA256.txt) for v2.7.1) and [VirusTotal](https://www.virustotal.com) transparency — it is **not** a substitute for vulnerability reporting.

## Safe harbor

We appreciate responsible disclosure. Good-faith security research on supported releases will not be pursued as a policy violation.
