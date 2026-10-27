# Brand

## Files

| File | Use |
|---|---|
| `logo.png` | Lockup, 2200 × 520, transparent outside the dark card |
| `logo-mark.svg` | Vector mark |
| `logo-mark.png` | 1024 px mark, transparent |
| `lockup.html` | Source for the lockup |

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
- Do not stretch, outline, recolor, or separate the gradient from the S.
- Wordmark: Cascadia Mono Bold, lowercase; only the `s` is teal.
- The lockup is a dark card by design; do not place the white wordmark directly
  on a light background.

## Regenerating

`logo-mark.svg` is hand-kept (the S is a real glyph outline). For the lockup,
render `lockup.html` at 1100 × 260 with a 2× device scale and a transparent
background, then round the corners (`docs/screenshot.md` shows the same Chrome
pipeline).
