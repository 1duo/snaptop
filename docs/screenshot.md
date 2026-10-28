# Captures

The README hero is a real 10 fps sequence, not a mock-up; the linked still is
a 3× frame from the same dashboard.

## Animated demo

The GIF is 60 frames / 6 s. Start a sustained NPU workload if you want the NPU
active, then:

```powershell
Remove-Item -Recurse -Force docs\frames -ErrorAction SilentlyContinue
.\target\release\snaptop.exe --capture-frames docs\frames --theme dark
.\docs\make-gif.ps1
Remove-Item -Recurse -Force docs\frames
```

`--capture-frames` first samples 100 frames (10 s) to fill the history, then
writes 60 HTML frames at 10 fps. `make-gif.ps1` renders them at 2×, trims the
transparent margin, and encodes `docs/demo.gif` with ffmpeg and gifsicle:
1600 px wide, 256-colour palette, transparency, 10 fps, about 1 MB. Chrome,
ffmpeg and gifsicle are required; the Windows ARM64 gifsicle build is linked
from [lcdf.org/gifsicle](https://www.lcdf.org/gifsicle/). Put it on `PATH`, or
pass `-Gifsicle <path>` to the script.

## Still

The still is 3× and is the click-through target for the GIF.

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
