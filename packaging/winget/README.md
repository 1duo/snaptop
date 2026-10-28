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
`New-WingetManifests.ps1`, validates them, and attaches both to the release. The
hash can never drift from the asset.

Submit once per release:

```powershell
winget install Microsoft.WingetCreate
wingetcreate submit --token <GITHUB_PAT> .\out
```

That opens a PR against `microsoft/winget-pkgs`; after it merges, the install
command works for everyone.

winget downloads the asset anonymously, so the release cannot live in a private
repository. If the source ever moves back to private, publish the ZIP from a
public release repo and pass `-InstallerUrl` to the script.
