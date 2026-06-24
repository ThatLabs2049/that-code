# Trust and download verification

ThatCode is open source. The **only trusted download source** is the official [GitHub Releases](https://github.com/Satan2049/that-code/releases) page for this repository.

Do not install ThatCode from third-party mirrors, random forum links, or unknown file shares.

---

## Verify SHA256 checksums

Each release includes a **`SHA256.txt`** file attached to the release assets (also in the repo root for [v2.7.1](../SHA256.txt)). It lists one line per installer or archive:

```
<sha256-hex>  <filename>
```

The filename must match the asset you downloaded exactly.

### v2.7.1 checksums

| File | SHA256 |
|------|--------|
| `ThatCode_2.7.1_x64-setup.exe` | `26754bc38d74085603d7ab2799c9c1336a19e1cae5936d6f926e045cf14be4ed` |
| `ThatCode_2.7.1_x64_en-US.msi` | `62881009afcfe2e1b1ac661117999b5a684b2e0eb653db61dda6fbc0bfdc65dd` |
| `ThatCode_2.7.1_x64-portable.zip` | `499f0c9b29199420424021cf2569258fdd965bb15624ac1413773335a2d791fb` |

### Windows (PowerShell)

```powershell
Get-FileHash -Algorithm SHA256 .\ThatCode_2.7.1_x64-setup.exe
```

Or for MSI:

```powershell
Get-FileHash -Algorithm SHA256 .\ThatCode_2.7.1_x64_en-US.msi
```

Compare the `Hash` value (lowercase hex) to the line in `SHA256.txt` for that filename.

Exact filenames depend on the Tauri bundle version — always match the asset name on the release page.

---

## VirusTotal reports

New and **unsigned** desktop installers often trigger **false positives** on one or two engines — especially heuristic / machine-learning rules. That does **not** automatically mean the file is malicious.

When reviewing a VirusTotal report, check:

| Signal | What to look for |
|--------|------------------|
| **Hash match** | The file hash on VirusTotal matches your download **and** the value in `SHA256.txt` from the official release |
| **Detection count** | Widespread detections across many reputable vendors are more concerning than one or two ML/heuristic flags |
| **Source** | Only trust files downloaded from [GitHub Releases](https://github.com/Satan2049/that-code/releases) |

### v2.7.1 scans (maintainer-submitted)

| Asset | VirusTotal | Notes |
|-------|------------|-------|
| NSIS (`.exe`) | [Report](https://www.virustotal.com/gui/file/26754bc38d74085603d7ab2799c9c1336a19e1cae5936d6f926e045cf14be4ed?nocache=1) | Arctic Wolf: Unsafe; SecureAge: Malicious — common unsigned-app heuristics |
| MSI | [Report](https://www.virustotal.com/gui/file/62881009afcfe2e1b1ac661117999b5a684b2e0eb653db61dda6fbc0bfdc65dd?nocache=1) | No security vendor flagged (0/72 at time of release) |
| Portable (`.zip`) | [Report](https://www.virustotal.com/gui/file/499f0c9b29199420424021cf2569258fdd965bb15624ac1413773335a2d791fb?nocache=1) | Trapmine: Suspicious.low.ml.score — ML heuristic only |

Re-scan after each release if your security team requires fresh reports. Legacy Muse v2.x scans remain in git history for reference.

---

## Security vulnerabilities vs. download trust

- **Download verification** (this document) — confirm you have an unmodified release file.
- **Vulnerability reporting** — see [SECURITY.md](../SECURITY.md).

---

## Maintainer: generating `SHA256.txt`

From the repository root after a Windows release build:

```powershell
.\scripts\generate-sha256.ps1
```

The release workflow attaches `SHA256.txt` automatically when a `v*` tag is pushed. Copy or verify against the committed manifest before publishing.
