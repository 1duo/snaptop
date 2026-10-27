# How snaptop reads the chip

Every value comes from an interface Windows already ships. Nothing here needs a
vendor SDK, a kernel driver, or administrator rights.

## A full frame

```
snaptop v0.1.0                                                           Snapdragon X Elite X1E78100
Windows 11 Enterprise 24H2 · build 26100 · 12 cores (8P + 4E)                                   1.0s

CPU   7% █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░          3.42 GHz · 5.0 W
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⣈⣎⣼⢏⢌⢈⢎⢈⢈⢈⢈⢈⢌⣈⣨⣏⢈⢈⢈⢈⢈⢈⢈⢈⣈⢈⢈⣾⢌⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⣼⢎⣎⣌⢈⢈⢈⣼⣿⣮⣮⣯⣸⣏⢈⢈⢈⢈⢈⢈⢈⢌⢈⢈⢎⣿⣈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⣨⣏⢈⢈⢈⢈⣈⢈⢈⢈⢈⢈⣈⣿⢈⢈⢈⢈⢌⢈⢈⢈⢈⢈
P 8× 4%   E 4× 14%   ▂▁▁▁ ▂···▁···

GPU   8% █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░     0.72/1.44 GHz · 0.0 W
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⢈⣈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢌⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢌⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⣈⢈⣈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢌⢈⢈⢌⢈⣈⢈⢈⢈⢈⢈⢈⢌⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢌⢈⢈⢈⢈⢈
Adreno X1-85 · 0.7/15.9 GiB · 3D 8%

NPU   0% ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░                      idle
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
Hexagon NPU · engine Compute 0%

MEM  52% ██████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░           16.6 / 31.6 GiB
⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌

SYS 15.8 W  ·  SoC 59 °C  ·  BAT 100% AC                               q quit · space pause · ? help
```

## Data sources

| Reading | Source |
|---|---|
| CPU utilisation | `\Processor Information(*)\% Processor Utility` (PDH) |
| CPU clocks | `CallNtPowerInformation(ProcessorInformation)` |
| CPU P/E split | `GetLogicalProcessorInformationEx` → `EfficiencyClass` |
| GPU / NPU engines | `\GPU Engine(*)\Utilization Percentage`, grouped by LUID |
| GPU memory | `\GPU Adapter Memory(*)` + DXGI adapter totals |
| GPU clock | `D3DKMTQueryAdapterInfo(KMTQAITYPE_NODEPERFDATA)` |
| Power | `\Energy Meter(*)\Power` — Snapdragon EMI rails, milliwatts |
| Thermal | `\Thermal Zone Information(*)\Temperature`, Kelvin |
| Battery | `GetSystemPowerStatus` |

## The NPU is a LUID nobody claims

Windows models the Hexagon NPU as an **MCDM compute accelerator**, not a display
adapter. It never appears in DXGI, so a naïve enumerator misses it.

It does appear in `\GPU Engine`, under a LUID that belongs to no DXGI adapter.
That difference *is* the NPU:

```
luid_0x00000000_0x00011643   → DXGI adapter  → Adreno X1-85
luid_0x00000000_0x0001221C   → no adapter    → Hexagon NPU
```

The same rule finds Intel and AMD NPUs, so detection is vendor-neutral even
though the power rails are Snapdragon-specific. Software rasterisers (WARP /
Basic Render Driver) are excluded explicitly, because they also emit engine
counters.

## Why power is free

Snapdragon platforms expose their Energy Meter Interface as ordinary PDH
counters:

```
\Energy Meter(sys)\Power            29.3 W    whole system
\Energy Meter(psu_usb)\Power        29.3 W    charger in
\Energy Meter(cpu_cluster_0)\Power   8.2 W
\Energy Meter(cpu_cluster_1)\Power   4.3 W
\Energy Meter(gpu)\Power             0.1 W
```

These are measured shunt values, not estimates, and they are readable without
administrator rights. The NPU is the one block with no rail — Windows and the
driver expose no NPU energy counter, so snaptop shows a dash rather than
inventing a number. The rails update at roughly 1 Hz, so they are cached at that
rate independently of the UI interval.

## Design

- **One sampler thread.** PDH rate counters need two collections; the render
  loop never waits on the chip. A paused UI keeps sampling and drops frames.
- **Fixed 0–100 sparklines.** The shape is comparable between runs and between
  machines. The meter carries the precise number.
- **Degrade, never crash.** A missing counter, an idle NPU, or a driver that
  does not implement the clock DDI becomes `—`, not an error.
- **Small surface.** Three dependencies (`ratatui`, `crossterm`, `serde_json`)
  plus `windows` and `winreg` on Windows. No async runtime, no config file.
- **Documented unsafe.** `#![deny(clippy::undocumented_unsafe_blocks)]` is on;
  every Win32 call states the invariant. FFI struct sizes and offsets are
  asserted in tests.

## Limitations

- **Windows only** today. The Linux backend is the next milestone; the data
  model and UI already have everything it needs.
- **No per-process attribution yet.** Aggregate utilisation only. `nptop`
  remains the tool for "which process is on the NPU".
- **NPU power is unavailable** on both Windows and Linux; the hardware exposes
  no rail.
- **Power rails are Snapdragon-specific.** On other Windows machines the power
  row is absent; CPU, GPU, NPU and memory still work.
- **GPU clock is optional.** It depends on the driver implementing
  `KMTQAITYPE_NODEPERFDATA`; when it does not, the field is `—`.
- NPU detection assumes the MCDM-LUID shape. That is how Windows exposes
  compute accelerators, but a future driver could change it.

## Roadmap

- Linux backend (`/proc/stat`, `amdgpu`/`i915`, `accel/*`, `hwmon`)
- per-process NPU and GPU attribution
- a compact one-line mode for status bars
- signed release binaries
