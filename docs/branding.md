# Brand

## Mark

A single geometric S on a dark rounded tile. The S is hand-drawn vector
geometry, not a font glyph, so the mark has no font dependency.

## Files

| File | Use |
|---|---|
| `logo-mark.svg` | Vector source, 512 × 512 |
| `logo-mark.png` | 1024 px raster, transparent outside the tile |
| `screenshot.png` | 3× still used as the GIF's click-through |
| `demo.gif` | Animated README hero, 1600 px, 60 frames at 10 fps |

## Palette

| Role | Hex |
|---|---|
| Background | `#0A0C11` |
| Foreground | `#D8E0EE` |
| Muted | `#6F7C91` |
| Border | `#2A3140` |
| Teal (CPU) | `#48DCC6` |
| Violet (GPU) | `#B094FF` |
| Amber (NPU) | `#FAC046` |

The mark's gradient runs 135°: `#7AF2DF` → `#B094FF` at 50% → `#FAC046`.

## Rules

- Keep the dark tile. It is what makes the mark legible on light and dark pages.
- Clear space: at least 25% of the tile width on every side.
- Use the mark alone; there is no wordmark lockup. Set the name in the
  surrounding text when needed.
- Do not stretch, outline, recolor, or separate the gradient from the S.

## Regenerating

`logo-mark.svg` is hand-kept. Render it at 512 × 512 with a 2× device scale and
a transparent page background:

```powershell
$chrome = "C:\Program Files\Google\Chrome\Application\chrome.exe"
& $chrome --headless=new --disable-gpu --hide-scrollbars `
  --default-background-color=00000000 --force-device-scale-factor=2 `
  --window-size=512,512 --screenshot=docs\logo-mark.png `
  "file:///$((Get-Location).Path -replace '\\','/')/docs/logo-mark.svg"
```
