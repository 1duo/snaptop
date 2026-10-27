# How snaptop reads the chip

Every value comes from an interface Windows already ships. No SDK, no driver,
and no admin rights.

## Sources

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
| Memory | `GlobalMemoryStatusEx` — used, available, commit |
| Battery | `GetSystemPowerStatus` |

## The NPU is a LUID nobody claims

Windows models the Hexagon NPU as an **MCDM compute accelerator**, not a display
adapter, so it never appears in DXGI. It does appear in `\GPU Engine` under a
LUID that no DXGI adapter owns — that difference *is* the NPU:

```
luid_0x00000000_0x00011643   → DXGI adapter  → Adreno X1-85
luid_0x00000000_0x0001221C   → no adapter    → Hexagon NPU
```

The rule is vendor-neutral, so it also finds Intel and AMD NPUs. Software
rasterisers (WARP) are excluded explicitly; they emit engine counters too.

## Power

Snapdragon exposes its Energy Meter as ordinary PDH counters:

```
\Energy Meter(sys)\Power            29.3 W    whole system
\Energy Meter(psu_usb)\Power        29.3 W    charger in
\Energy Meter(cpu_cluster_0)\Power   8.2 W
\Energy Meter(gpu)\Power             0.1 W
```

Measured shunt values, no admin. They update at ~1 Hz, so they are cached at
that rate. The NPU has no rail on any platform, so it shows a dash rather than a
guess.

## Design

- One sampler thread; the render loop never waits on the chip.
- Utilisation sparklines are fixed 0–100; power auto-scales to its peak.
- Missing counters degrade to `—`, never to an error.
- `aurora` (dark), `daylight` (light) and `mono` palettes. `--theme auto`
  follows the OS app theme, and every palette has a contrast test.
- `#![deny(clippy::undocumented_unsafe_blocks)]`; FFI layouts are asserted in tests.

## Limitations

- Windows only; Linux is the next milestone.
- Aggregate utilisation, no per-process attribution yet.
- NPU power unavailable; GPU clocks depend on an optional driver DDI.
- Power rails are Snapdragon-specific; elsewhere the row is absent.
- NPU detection assumes the MCDM-LUID shape.

## Roadmap

- Linux backend (`/proc/stat`, `amdgpu`/`i915`, `accel/*`, `hwmon`)
- per-process attribution
- a one-line status-bar mode
