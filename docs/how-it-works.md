# How snaptop reads the chip

Every value comes from an interface Windows already ships. No SDK, no driver,
and no background service.

## Sources

| Reading | Source |
|---|---|
| CPU utilisation | `\Processor Information(*)\% Processor Utility` (PDH) |
| CPU clocks | `CallNtPowerInformation(ProcessorInformation)` |
| CPU classes | `GetLogicalProcessorInformationEx` → `EfficiencyClass` |
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

Measured shunt values exposed by the platform. They update at ~1 Hz, so they
are cached at that rate. Rail names and counts vary by model, so they are
matched by prefix: every `cpu_cluster_*` is summed, and an `npu`/`nsp` rail is
used when present.

## Hardware

Tested on Snapdragon X Elite X1E78100. Nothing is hard-coded to that part:
core count and classes come from the firmware, GPU/NPU identity from DXGI and
the MCDM LUIDs, and power rails from whatever `\Energy Meter` exposes. X2
Elite's 12 Prime + 6 Performance cores are treated as performance cores — X2
has no efficiency tier — so the UI reports `18 cores` instead of inventing a
P/E split.

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
- NPU power is shown only when the platform exposes a rail; GPU clocks depend
  on an optional driver DDI.
- GPU Engine counters are usually readable unelevated, but some systems gate
  them behind an elevated shell.
- Power rails are Snapdragon-specific; elsewhere the row is absent.
- NPU detection assumes the MCDM-LUID shape.

## Roadmap

- Linux backend (`/proc/stat`, `amdgpu`/`i915`, `accel/*`, `hwmon`)
- per-process attribution
- a one-line status-bar mode
