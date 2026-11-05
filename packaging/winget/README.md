# winget packaging

Released as a portable ZIP: `snaptop.exe` plus the license. winget keeps it in
its package store and puts `snaptop` on `PATH`.

```powershell
winget install 1duo.snaptop
winget upgrade 1duo.snaptop
winget uninstall 1duo.snaptop
```

`.github/workflows/release.yml` runs on a `v*` tag: builds ARM64, zips it,
computes the SHA256, generates the three manifests with
`New-WingetManifests.ps1`, validates them, and attaches the ZIP and manifests
to the release. The hash can never drift from the asset.

Submit once per release, from the repository root:

```powershell
winget install Microsoft.WingetCreate
wingetcreate submit --token <GITHUB_PAT> .\packaging\winget\out
```

That opens a PR against `microsoft/winget-pkgs`; after it merges, the install
command works for everyone.

winget installs the architecture-matched Microsoft Visual C++ Redistributable
(`Microsoft.VCRedist.2015+.arm64` / `.x64`) through the installer manifest's
`Dependencies`. If you extract the ZIP and run `snaptop.exe` by hand, install
that VC++ runtime first: without it the process exits immediately with
`0xC0000135` (`STATUS_DLL_NOT_FOUND`), a loader failure that happens before the
CLI parses any arguments.

winget downloads the asset anonymously, so the release cannot live in a private
repository. If the source ever moves back to private, publish the ZIP from a
public release repo and pass `-InstallerUrl` to the script.
