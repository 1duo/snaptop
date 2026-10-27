# winget packaging

`snaptop` is distributed as a **portable ZIP**: the release asset contains
`snaptop.exe`, `LICENSE` and `README.md`, and the manifest registers
`snaptop.exe` as the `snaptop` command. winget copies it into its package store
and puts a shim on `PATH`, so upgrades and uninstalls are managed for you.

## User-facing command

```powershell
winget install 1duo.snaptop
winget upgrade  1duo.snaptop
winget uninstall 1duo.snaptop
```

## How a release is produced

`.github/workflows/release.yml` runs on a `v*` tag and:

1. builds `snaptop.exe` for `aarch64-pc-windows-msvc`;
2. zips it as `snaptop-v<version>-arm64.zip`;
3. computes the SHA256 of that exact ZIP;
4. calls `New-WingetManifests.ps1` to emit the three manifests;
5. validates them with `winget validate`;
6. attaches the ZIP and the manifests to the GitHub release.

Nothing is hand-edited, so the hash can never drift from the asset.

## Publishing to the winget community repository

Once per release, submit the generated manifests:

```powershell
winget install Microsoft.WingetCreate
wingetcreate submit --token <YOUR_GITHUB_PAT> .\out
```

`wingetcreate` opens a pull request against `microsoft/winget-pkgs`. After it
is merged (usually a few days), `winget install 1duo.snaptop` works for
everyone. The workflow's job ends at the release; submission is a maintainer
action because it requires a token with `public_repo` scope.

## Private source, public artifact

winget must be able to download `InstallerUrl` anonymously, so the release
asset cannot live in a private repository. If the source stays private, publish
the ZIP from a small public repository (for example `1duo/snaptop-releases`) and
pass `-InstallerUrl` to the script.
