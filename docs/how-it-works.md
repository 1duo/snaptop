# How snaptop reads the chip

Every value comes from an interface the OS already ships. No SDK, no driver,
and no background service. There is one backend per platform; the UI only ever
sees a `Snapshot`.

## Windows

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

### The NPU is a LUID nobody claims

Windows models the Hexagon NPU as an **MCDM compute accelerator**, not a display
adapter, so it never appears in DXGI. It does appear in `\GPU Engine` under a
LUID that no DXGI adapter owns — that difference *is* the NPU:

```
luid_0x00000000_0x00011643   → DXGI adapter  → Adreno X1-85
luid_0x00000000_0x0001221C   → no adapter    → Hexagon NPU
```

The rule is vendor-neutral, so it also finds Intel and AMD NPUs. Software
rasterisers (WARP) are excluded explicitly; they emit engine counters too.

### Power on Windows

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

## Linux

| Reading | Source |
|---|---|
| CPU utilisation | `/proc/stat` deltas (`busy = total - idle - iowait`) |
| CPU clocks | `cpu*/cpufreq/scaling_cur_freq` and `cpuinfo_max_freq` |
| CPU classes | `cpu_capacity` tiers, or `cpu_core` / `cpu_atom` on x86 hybrid |
| GPU | `/sys/class/kgsl/kgsl-3d0` (`gpu_busy_percentage`, `gpu_model`, clocks) |
| GPU fallback | a DRM card's `gpu_busy_percent` (amdgpu) |
| NPU | the CDSP `fastrpc` node, or an `accel` class device |
| Power | `/sys/class/power_supply` — battery `power_now`, mains `online` |
| Memory | `/proc/meminfo` — `MemTotal`, `MemAvailable`, `Committed_AS`/`CommitLimit` |
| Thermal | `/sys/class/thermal/thermal_zone*/temp`, millidegrees |
| Identity | `/proc/device-tree/model`, `/etc/os-release` |

Everything is a file read, so the backend runs unprivileged and needs no
device-node access — the Dragonwing IQ-9075 EVK, for example, restricts
`/dev/dri/renderD128` to the `render` group, and snaptop does not touch it.

Core classes come from the topmost `cpu_capacity` tier when the firmware
reports more than one; a single tier means "no split", not "all P". On x86
hybrid parts the kernel's `cpu_core` / `cpu_atom` masks are used instead.

### NPU on Linux

Linux has no system-wide NPU busy counter. The Hexagon is driven through
FastRPC, its `qcom_stats` residency counters live in debugfs and need root.
The NPU row therefore shows the part it found and `—` for utilisation rather
than a fabricated `0%`. The model keeps utilisation optional, so the day a
kernel exposes a counter, the panel shows it without a protocol change; JSON
reports `"util_pct": null` until then.

### Power on Linux

There is no energy meter. A battery's `power_now` is whole-system draw while it
discharges, and a mains adapter may report input power. A board powered from
USB-C without a battery, like the Dragonwing IQ-9075 EVK, has neither node, so
the POWER panel shows `no rails` and the row degrades to `—`.

## Hardware

Tested on Snapdragon X Elite (X1E-78-100) and X2 Elite (X2E-88-100) on Windows 11
Arm, and the Qualcomm Dragonwing IQ-9075 EVK on Ubuntu 24.04 arm64. Nothing is
hard-coded to a part: core count and classes come from the firmware, GPU/NPU
identity from the driver, and power rails from whatever the platform exposes.

X2 Elite's 12 Prime + 6 Performance cores have no efficiency tier. Both
backends still rank the two tiers, so one shared rule in `model.rs` folds the
lower tier back into Performance: the UI reports `18 cores` instead of
inventing a P/E split, and `normalize_kinds` is the single place that rule
lives. On X2E-88-100 that reads `P 18×`, with `e_util_pct` null in JSON.

## Design

- One sampler thread; the render loop never waits on the chip.
- One backend per OS behind `Sampler::{new, device, sample}`.
- Utilisation sparklines are fixed 0–100; power auto-scales to its peak.
- Missing counters degrade to `—`, never to an error.
- `aurora` (dark), `daylight` (light) and `mono` palettes. `--theme auto`
  follows the OS app theme, and every palette has a contrast test.
- `#![deny(clippy::undocumented_unsafe_blocks)]`; FFI layouts are asserted in tests.
- The Linux backend takes a root path, so its `/proc` and `/sys` readers are
  covered by fixture tests that run on every host and in CI.

## Limitations

- Aggregate utilisation, no per-process attribution yet.
- NPU utilisation is Windows-only; Linux shows presence and no counter.
- NPU power is shown only when the platform exposes a rail; GPU clocks depend
  on an optional driver DDI.
- GPU Engine counters are usually readable unelevated, but some systems gate
  them behind an elevated shell.
- Power rails are Snapdragon-specific on Windows; on Linux only battery and
  mains nodes are read.
- NPU detection assumes the MCDM-LUID shape on Windows and a CDSP/accel node on
  Linux.

## Roadmap

- per-process attribution
- a one-line status-bar mode
- NPU utilisation on Linux once a kernel counter lands
