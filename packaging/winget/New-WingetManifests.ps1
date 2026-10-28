#Requires -Version 5.1
<#
.SYNOPSIS
    Generate the three winget manifests for a snaptop release.

.DESCRIPTION
    The release workflow calls this after the portable ZIP has been built, so
    the SHA256 in the installer manifest always describes the exact bytes that
    were uploaded. It can also be run by hand:

        ./New-WingetManifests.ps1 -Version 0.1.0 -Sha256 (Get-FileHash snaptop-v0.1.0-arm64.zip).Hash

.EXAMPLE
    ./New-WingetManifests.ps1 -Version 0.1.0 -Sha256 ABCDEF... -OutDir ./out
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$Sha256,

    [ValidateSet('arm64', 'x64')]
    [string]$Architecture = 'arm64',

    [string]$InstallerUrl,

    [string]$OutDir = (Join-Path $PSScriptRoot 'out')
)

$ErrorActionPreference = 'Stop'

# .NET file APIs resolve relative paths against the process working directory,
# which PowerShell's `cd` does not change. Pin the output directory to an
# absolute path once, up front.
$OutDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutDir)

if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    throw "Version must look like 1.2.3 (got '$Version')."
}

$Sha256 = $Sha256.Trim().ToUpperInvariant()
if ($Sha256 -notmatch '^[0-9A-F]{64}$') {
    throw 'Sha256 must be 64 hexadecimal characters.'
}

if (-not $InstallerUrl) {
    $InstallerUrl = "https://github.com/1duo/snaptop/releases/download/v$Version/snaptop-v$Version-$Architecture.zip"
}

$versionManifest = @'
# yaml-language-server: $schema=https://aka.ms/winget-manifest.version.1.10.0.schema.json
PackageIdentifier: 1duo.snaptop
PackageVersion: {{VERSION}}
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.10.0
'@

$installerManifest = @'
# yaml-language-server: $schema=https://aka.ms/winget-manifest.installer.1.10.0.schema.json
PackageIdentifier: 1duo.snaptop
PackageVersion: {{VERSION}}
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
  - RelativeFilePath: snaptop.exe
    PortableCommandAlias: snaptop
Installers:
  - Architecture: {{ARCH}}
    InstallerUrl: {{URL}}
    InstallerSha256: {{SHA256}}
ManifestType: installer
ManifestVersion: 1.10.0
'@

$localeManifest = @'
# yaml-language-server: $schema=https://aka.ms/winget-manifest.defaultLocale.1.10.0.schema.json
PackageIdentifier: 1duo.snaptop
PackageVersion: {{VERSION}}
PackageLocale: en-US
Publisher: Yuduo Wu
PublisherUrl: https://github.com/1duo
PublisherSupportUrl: https://github.com/1duo/snaptop/issues
Author: Yuduo Wu
PackageName: snaptop
PackageUrl: https://github.com/1duo/snaptop
License: MIT
LicenseUrl: https://github.com/1duo/snaptop/blob/main/LICENSE
Copyright: Copyright (c) 2026 Yuduo Wu
ShortDescription: CPU, GPU, NPU and power monitor for Snapdragon laptops
Description: |
  snaptop is a fast, single-binary terminal dashboard for Snapdragon laptops.
  It shows CPU utilisation and per-core load, Adreno GPU utilisation and clocks,
  Hexagon NPU utilisation, memory, per-rail power from the Snapdragon energy
  meter, and the hottest thermal zone, with no driver or SDK required.
Moniker: snaptop
Tags:
  - snapdragon
  - qualcomm
  - npu
  - arm64
  - monitor
  - tui
  - system
ReleaseNotesUrl: https://github.com/1duo/snaptop/releases/tag/v{{VERSION}}
ManifestType: defaultLocale
ManifestVersion: 1.10.0
'@

$installer = $installerManifest.Replace('{{VERSION}}', $Version)
$installer = $installer.Replace('{{ARCH}}', $Architecture)
$installer = $installer.Replace('{{URL}}', $InstallerUrl)
$installer = $installer.Replace('{{SHA256}}', $Sha256)

$files = [ordered]@{
    '1duo.snaptop.yaml'              = $versionManifest.Replace('{{VERSION}}', $Version)
    '1duo.snaptop.installer.yaml'    = $installer
    '1duo.snaptop.locale.en-US.yaml' = $localeManifest.Replace('{{VERSION}}', $Version)
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$utf8 = New-Object System.Text.UTF8Encoding($false)
foreach ($name in $files.Keys) {
    [System.IO.File]::WriteAllText((Join-Path $OutDir $name), $files[$name], $utf8)
}

Write-Host "winget manifests written to $OutDir"
Get-ChildItem $OutDir | Select-Object Name, Length | Format-Table -AutoSize
