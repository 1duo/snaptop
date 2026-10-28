param(
  [string] $FramesDir = (Join-Path $PSScriptRoot 'frames'),
  [string] $Out = (Join-Path $PSScriptRoot 'demo.gif'),
  [string] $Gifsicle = '',
  [int] $Width = 1600
)

$ErrorActionPreference = 'Stop'

$chrome = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
if (-not (Test-Path $chrome)) {
  throw "Chrome not found at $chrome"
}
if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
  throw 'ffmpeg is required to encode the GIF'
}
if (-not $Gifsicle) {
  $command = Get-Command gifsicle -ErrorAction SilentlyContinue
  if ($command) {
    $Gifsicle = $command.Source
  }
}
if (-not $Gifsicle -or -not (Test-Path $Gifsicle)) {
  throw 'gifsicle is required to optimize the GIF; pass -Gifsicle <path>'
}

$frames = Get-ChildItem $FramesDir -Filter 'frame-*.html' | Sort-Object Name
if ($frames.Count -ne 60) {
  throw "Expected 60 frames in $FramesDir, found $($frames.Count)"
}

$profile = Join-Path $env:TEMP 'chrome-snaptop-gif'
foreach ($frame in $frames) {
  $png = Join-Path $FramesDir ([IO.Path]::GetFileNameWithoutExtension($frame.Name) + '.png')
  & $chrome --headless=new --disable-gpu --hide-scrollbars --no-first-run `
    --no-default-browser-check --user-data-dir=$profile `
    --default-background-color=00000000 --force-device-scale-factor=2 `
    --window-size=1000,640 --screenshot=$png `
    "file:///$($frame.FullName -replace '\\','/')" 2>&1 | Out-Null
  if ($LASTEXITCODE -ne 0 -or -not (Test-Path $png)) {
    throw "Chrome did not render $($frame.Name)"
  }
  & (Join-Path $PSScriptRoot 'trim-transparent.ps1') $png
}

$raw = Join-Path $env:TEMP 'snaptop-demo-raw.gif'
$pattern = Join-Path $FramesDir 'frame-%04d.png'
$filter = "scale=${Width}:-2:flags=lanczos,split[a][b];[a]palettegen=reserve_transparent=1:stats_mode=diff:max_colors=256[p];[b][p]paletteuse=dither=none:diff_mode=rectangle"
& ffmpeg -y -loglevel error -framerate 10 -i $pattern -vf $filter -loop 0 $raw
if ($LASTEXITCODE -ne 0) {
  throw 'ffmpeg failed to encode the GIF'
}

& $Gifsicle -O3 $raw -o $Out
if ($LASTEXITCODE -ne 0) {
  throw 'gifsicle failed to optimize the GIF'
}
Remove-Item $raw -Force

$size = [math]::Round((Get-Item $Out).Length / 1MB, 2)
Write-Host "wrote $Out ($size MB)"
