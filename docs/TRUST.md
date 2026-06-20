# Trust and download verification

Muse is open source. The **only trusted download source** is the official [GitHub Releases](https://github.com/Satan2049/muse/releases) page for this repository.

Do not install Muse from third-party mirrors, random forum links, or unknown file shares.

---

## Verify SHA256 checksums

Each release includes a **`SHA256.txt`** file attached to the release assets. It lists one line per installer or archive:

```
<sha256-hex>  <filename>
```

The filename must match the asset you downloaded exactly.

### Windows (PowerShell)

```powershell
Get-FileHash -Algorithm SHA256 .\Muse_2.1.0_x64-setup.exe
```

Compare the `Hash` value (lowercase hex) to the line in `SHA256.txt` for that filename.

### macOS

```powershell
# If you use PowerShell on Windows-style paths, same as above.
# On macOS Terminal:
shasum -a 256 Muse_2.1.0_aarch64.dmg
```

Or:

```bash
sha256sum Muse_2.1.0_aarch64.dmg
```

### Linux

```bash
sha256sum Muse_2.1.0_amd64.AppImage
```

If the computed hash matches the official manifest, the file was not altered in transit.

---

## VirusTotal reports

New and **unsigned** desktop installers often trigger **false positives** on one or two engines — especially heuristic / machine-learning rules (for example Trapmine, SecureAge, Arctic Wolf). That does **not** automatically mean the file is malicious.

When reviewing a VirusTotal report, check:

| Signal | What to look for |
|--------|------------------|
| **Hash match** | The file hash on VirusTotal matches your download **and** the value in `SHA256.txt` from the official release |
| **Detection count** | Widespread detections across many reputable vendors are more concerning than one or two ML/heuristic flags |
| **Vendor reputation** | Prefer reports linked from our release notes; re-scan the **exact** file you downloaded from GitHub Releases |
| **Source** | Only trust files downloaded from [GitHub Releases](https://github.com/Satan2049/muse/releases) |

### Example reports (Windows v2.x installers)

These permalinks are provided for transparency. Re-scan your own download if the release version differs.

| Artifact | VirusTotal | Notes |
|----------|------------|-------|
| Portable `.exe` | [Analysis](https://www.virustotal.com/gui/file-analysis/Y2RmNzI3ZWQyMWJjMGY0YThmMWQ4MDczNGY0MTQ4YzU6MTc4MTk1NTI1MA==) | Trapmine: Malicious.moderate.ml.score (heuristic) |
| `.msi` installer | [Analysis](https://www.virustotal.com/gui/file-analysis/N2VhODZjMDMzNmJhNmJlYmZjMGM1YmQ0ZDBmYjM4NDM6MTc4MTk1NTM5MA==) | Trapmine: Malicious.moderate.ml.score (heuristic) |
| NSIS setup | [File report](https://www.virustotal.com/gui/file/a4b988531bba53cbc48bae6cdafc28c45ad8ec6e9c1a17fdf5690bf788d85c9d?nocache=1) | Arctic Wolf (Unsafe), SecureAge (Malicious) — common on unsigned NSIS builds |

If your hash matches the official release but VirusTotal shows a few heuristic hits, treat that as expected for unsigned open-source desktop software until code signing is added.

---

## Security vulnerabilities vs. download trust

- **Download verification** (this document) — confirm you have an unmodified release file.
- **Vulnerability reporting** — see [SECURITY.md](../SECURITY.md) for how to report security issues in Muse itself.

These are separate concerns. A verified download can still contain a bug; report bugs through GitHub Issues and security issues through the process in `SECURITY.md`.

---

## Maintainer: generating `SHA256.txt`

From the repository root after a release build:

```powershell
.\scripts\generate-sha256.ps1
```

Or point at a CI artifacts folder:

```powershell
.\scripts\generate-sha256.ps1 -FolderPath .\bundle -OutputPath .\SHA256.txt
```

Attach the resulting `SHA256.txt` to the GitHub Release (the release workflow merges platform manifests automatically when tags are pushed).
