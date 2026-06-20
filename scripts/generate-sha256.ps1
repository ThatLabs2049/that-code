<#
.SYNOPSIS
    Generates SHA256.txt checksum manifest for Muse release artifacts.

.DESCRIPTION
    Recursively scans a folder for release binaries and archives, computes
    SHA256 hashes, and writes a manifest in GNU-style format:

        <sha256-hex>  <filename>

    Filename in each line is the basename only (matches GitHub Release asset names).

.PARAMETER FolderPath
    Root folder to scan. Defaults to src-tauri/target/release/bundle relative
    to the repository root (or the current directory if that path is missing).

.PARAMETER OutputPath
    Destination file for the manifest. Defaults to SHA256.txt in the parent
    directory of FolderPath.

.EXAMPLE
    .\scripts\generate-sha256.ps1

.EXAMPLE
    .\scripts\generate-sha256.ps1 -FolderPath .\ci-artifacts\bundle -OutputPath .\SHA256.txt

.EXAMPLE
    pwsh ./scripts/generate-sha256.ps1 -FolderPath src-tauri/target/release/bundle
#>
[CmdletBinding()]
param(
    [Parameter()]
    [string] $FolderPath = "",

    [Parameter()]
    [string] $OutputPath = ""
)

$ErrorActionPreference = "Stop"

$releaseExtensions = @(
    ".exe",
    ".msi",
    ".zip",
    ".dmg",
    ".deb",
    ".rpm",
    ".appimage"
)

function Resolve-DefaultFolderPath {
    $candidates = @(
        (Join-Path -Path $PSScriptRoot -ChildPath "..\src-tauri\target\release\bundle"),
        (Join-Path -Path (Get-Location) -ChildPath "src-tauri\target\release\bundle"),
        (Get-Location).Path
    )

    foreach ($candidate in $candidates) {
        $resolved = (Resolve-Path -LiteralPath $candidate -ErrorAction SilentlyContinue)
        if ($resolved) {
            return $resolved.Path
        }
    }

    throw "No bundle folder found. Pass -FolderPath explicitly."
}

function Test-ReleaseArtifactName {
    param([string] $Name)

    $lower = $Name.ToLowerInvariant()

    foreach ($extension in $releaseExtensions) {
        if ($lower.EndsWith($extension)) {
            return $true
        }
    }

    if ($lower.EndsWith(".tar.gz") -or $lower.EndsWith(".app.tar.gz")) {
        return $true
    }

    return $false
}

if ([string]::IsNullOrWhiteSpace($FolderPath)) {
    $FolderPath = Resolve-DefaultFolderPath
} else {
    $FolderPath = (Resolve-Path -LiteralPath $FolderPath).Path
}

if (-not (Test-Path -LiteralPath $FolderPath -PathType Container)) {
    Write-Error "Folder not found: $FolderPath"
    exit 1
}

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $parent = Split-Path -Parent $FolderPath
    if ([string]::IsNullOrWhiteSpace($parent)) {
        $OutputPath = Join-Path -Path (Get-Location) -ChildPath "SHA256.txt"
    } else {
        $OutputPath = Join-Path -Path $parent -ChildPath "SHA256.txt"
    }
} else {
    $outputDirectory = Split-Path -Parent $OutputPath
    if (-not [string]::IsNullOrWhiteSpace($outputDirectory) -and -not (Test-Path -LiteralPath $outputDirectory)) {
        New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
    }
}

$files = Get-ChildItem -LiteralPath $FolderPath -Recurse -File |
    Where-Object { Test-ReleaseArtifactName -Name $_.Name } |
    Sort-Object -Property FullName

if ($files.Count -eq 0) {
    Write-Error "No hashable release artifacts found under: $FolderPath"
    exit 1
}

$lines = New-Object System.Collections.Generic.List[string]

foreach ($file in $files) {
    $hash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    $lines.Add("$hash  $($file.Name)")
}

$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllLines($OutputPath, $lines, $utf8NoBom)

Write-Host "Wrote $($files.Count) checksum(s) to $OutputPath"
foreach ($line in $lines) {
    Write-Host "  $line"
}

exit 0
