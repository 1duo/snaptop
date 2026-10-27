# Regenerating the README screenshot

The capture is a real frame rendered from the live buffer, not a mock-up.

```powershell
cargo build --release
.\target\release\snaptop.exe --capture docs\frame.html --theme dark

$chrome = "C:\Program Files\Google\Chrome\Application\chrome.exe"
& $chrome --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=2 `
  --window-size=1120,760 --screenshot=docs\screenshot.png `
  "file:///$((Get-Location).Path -replace '\\','/')/docs/frame.html"

Remove-Item docs\frame.html
```

Edge works with the same flags. The HTML uses Cascadia Mono if it is installed,
then Consolas; the frame is sampled over ~10 s at 100 ms so the history has a
real trace.
