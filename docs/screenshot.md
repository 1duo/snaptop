# Regenerating the README screenshot

The capture is a real frame rendered from the live buffer, not a mock-up.

```powershell
cargo build --release
.\target\release\snaptop.exe --capture docs\frame.html --theme dark

$chrome = "C:\Program Files\Google\Chrome\Application\chrome.exe"
& $chrome --headless=new --disable-gpu --hide-scrollbars `
  --default-background-color=00000000 --force-device-scale-factor=3 `
  --window-size=1000,640 --screenshot=docs\screenshot.png `
  "file:///$((Get-Location).Path -replace '\\','/')/docs/frame.html"

Remove-Item docs\frame.html
.\docs\trim-transparent.ps1 docs\screenshot.png
```

The page has no background, so Chrome writes the rounded window with
transparent corners; the crop removes the viewport margin and leaves the window
alone. The committed frame is 2762 × 1764 at 3×. Edge works with the same flags.
The HTML uses Cascadia Mono if it is installed, then Consolas; the frame is
sampled over ~10 s at 100 ms so the history has a real trace.
