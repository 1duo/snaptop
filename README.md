# snaptop

**CPU, GPU, NPU and power — in one terminal.**

A fast, single-binary `top` for Snapdragon laptops. No SDK, no driver, no
administrator rights. Built for Windows on Snapdragon first; the model is
already shaped for a Linux backend.

```
snaptop v0.1.0                                                           Snapdragon X Elite X1E78100
Windows 11 Enterprise 24H2 · build 26100 · 12 cores (8P + 4E)                                   1.0s

CPU  15% ██████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░         3.42 GHz · 13.1 W
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⣌⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢈⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢈⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⢌⠀⠀⠀⠀⢀⣀⢌⢀⠀⢀⣮⠌⠀⢀⠀
⢌⣾⣿⣮⣾⣮⣎⢈⢈⣨⣎⣌⣎⢈⣿⣿⣿⣌⢈⢈⢈⢈⢈⢌⣮⣌⣾⢏⢌⢈⢈⢈⢌⣈⣈⢌⢈⣌⣾⢏⢈⢈⢈⣈⢌⢈⣬⢌⣌⣨⣿⢈⢈⢈⢈⣈⢌⢈⣈⣌⣈⣿⢈⣬⣎⢈⢈⢈⣈⣌⣈⣾⣿⣯⣮⣎⣮⣮⣬⣌⢈⣈⢌⣼⣿⢈⢈⣈⢈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣎
P 8× 12%   E 4× 20%   ▂▂▁▁ ▃▁··▄▁··

GPU  10% ██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░     0.72/1.44 GHz · 0.1 W
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⢌⢌⢌⢌⣈⢌⣌⢈⣌⣈⣌⣌⣈⢌⢌⢌⣌⢌⣌⢈⢌⣌⣌⣌⣌⣌⣌⣌⣌⣈⣌⢈⣌⣌⣈⣌⣈⣈⣌⣈⢌⢌⣌⢌⣌⣌⣈⢌⢈⣌⣈⢈⢈⢈⣈⢌⢈⣈⢈⢈⢌⣌⢈⣈⣌⢈⣈⣈⢈⢌⣈⣌⢈⢈⢈⢌⣌⢌⣈⣌⣌⣌⢈⣌⢌⢌⣌⢌⣌⣈⣈⣌⣌⢌⣌⣌⢌⣌⢌⣌
Adreno X1-85 · 0.6/15.9 GiB · 3D 10%

NPU   0% ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░                      idle
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
Hexagon NPU · engine Compute 0%

MEM  55% ████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░           17.3 / 31.6 GiB
⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌⣌

SYS 29.3 W  ·  SoC 69 °C  ·  BAT 100% ⚡                                q quit · space pause · ? help
```

<sub>Real capture from a Snapdragon X Elite (X1E78100); `snaptop --preview`. The
CPU trace is sampled every 20 ms here, so it is spikier than the default 1 s cadence.</sub>

## Why another monitor?

On Apple Silicon, `mactop` and `macmon` answer one question well: *what are the
CPU, GPU and neural engine doing right now?* On Windows on Snapdragon there was
no terminal answer at all.

| | CPU | Adreno GPU | Hexagon NPU | Power |
|---|:---:|:---:|:---:|:---:|
| Task Manager | ● | ● | ● (GUI only, no history, no pipe) | — |
| btop / btop4win | ● | — | — | — |
| bottom | ● | — (NVML only) | — | — |
| nvtop / nvitop | — | — | — | — |
| nptop | — | — | ● (per process) | — |
| xpu-top | ● | ● (vendor-neutral WDDM) | ● | — |
| **snaptop** | ● | ● | ● | ● |

`snaptop` is deliberately small: four panels, one screen, a stable JSON mode.
It does not try to be a process explorer.

## Install

### winget (recommended)

```powershell
winget install 1duo.snaptop
```

One command — no clone, no toolchain. The package is a portable ZIP: winget
keeps `snaptop.exe` in its package store and puts a `snaptop` shim on `PATH`, so
`winget upgrade 1duo.snaptop` and `winget uninstall 1duo.snaptop` manage the
lifecycle too.

> The 0.1.0 manifest is submitted to `microsoft/winget-pkgs` with the first
> release. Until it is merged, use the build-from-source path below.

### From source

```powershell
git clone https://github.com/1duo/snaptop
cd snaptop
cargo build --release
copy target\release\snaptop.exe "$env:LOCALAPPDATA\Microsoft\WindowsApps\"
```

Rust 1.85+. No runtime dependencies; one ~0.8 MB executable.

## Usage

```
snaptop                     the dashboard
snaptop --once              one plain-text snapshot, then exit
snaptop --json --once       one JSON object
snaptop --json -i 0.5       NDJSON metric stream for a script
snaptop --preview           render one frame as plain text (bug reports)
snaptop --theme nebula      aurora (default), nebula, mono
```

Keys: `q` quit · `space` pause · `+`/`-` slower/faster · `r` reset history ·
`t` theme · `?` help.

### JSON

```json
{
  "seq": 0,
  "ts_ms": 1770000000000,
  "version": "0.1.0",
  "device": { "cpu": "Snapdragon X Elite X1E78100", "os": "Windows 11 Enterprise", "build": "26100", "gpu": "Adreno X1-85", "npu": "Hexagon NPU" },
  "cpu": { "util_pct": 18.3, "p_util_pct": 4.3, "e_util_pct": 46.1, "cur_mhz": 2707, "max_mhz": 3417, "power_w": 9.9, "cores": [ … ] },
  "gpus": [ { "name": "Adreno X1-85", "util_pct": 10.0, "clock_mhz": 720, "clock_max_mhz": 1443, "mem_used_bytes": 629145600, "mem_total_bytes": 17054097408, "power_w": 0.1, "engines": [ … ] } ],
  "npus": [ { "name": "Hexagon NPU", "util_pct": 0.0, "power_w": null, "engines": [ { "name": "Compute", "util_pct": 0.0 } ] } ],
  "memory": { "used_bytes": 19327352832, "total_bytes": 33929805824, "util_pct": 57.0 },
  "power": { "sys_w": 29.3, "psu_w": 29.3, "cpu_w": 13.1, "gpu_w": 0.1, "battery_pct": 100, "ac": true },
  "thermal_c": 69.0
}
```

## How it reads the chip

Every value comes from an interface Windows already ships. Nothing here needs a
vendor SDK, a kernel driver or elevation.

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

### The NPU is a LUID nobody claims

Windows models the Hexagon NPU as an **MCDM compute accelerator**, not as a
display adapter. It never appears in DXGI, so a naïve enumerator misses it.

It does appear in `\GPU Engine`, under a LUID that belongs to no DXGI adapter.
That difference *is* the NPU:

```
luid_0x00000000_0x00011643   → DXGI adapter  → Adreno X1-85
luid_0x00000000_0x0001221C   → no adapter    → Hexagon NPU
```

The same rule finds Intel and AMD NPUs, so the detection is vendor-neutral even
though the power rails are Snapdragon-specific. Software rasterisers (WARP /
Basic Render Driver) are excluded explicitly, because they also emit engine
counters.

### Why power is free

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
inventing a number.

## Design

- **One sampler thread.** PDH rate counters need two collections; the render
  loop never waits on the chip. A paused UI keeps sampling and drops frames.
- **Fixed 0–100 sparklines.** The shape is comparable between runs and between
  machines. The meter carries the precise number.
- **Degrade, never crash.** A missing counter, an idle NPU, an old driver that
  does not implement the clock DDI — each becomes `—`, not an error.
- **Small surface.** Three dependencies (`ratatui`, `crossterm`, `serde_json`)
  plus `windows` and `winreg` on Windows. No async runtime, no config file.

## Limitations

- **Windows only** today. The Linux backend is the next milestone; the data
  model and UI already have everything it needs.
- **No per-process attribution yet.** Aggregate utilisation only. `nptop`
  remains the tool for "which process is on the NPU".
- **NPU power is unavailable** on both Windows and Linux; the hardware exposes
  no rail.
- **Power rails are Snapdragon-specific.** On other Windows machines the power
  row is simply absent; CPU, GPU, NPU and memory still work.
- **GPU clock is optional.** It depends on the driver implementing
  `KMTQAITYPE_NODEPERFDATA`; when it does not, the field is `—`.
- NPU detection assumes the MCDM-LUID shape. That is how Windows exposes
  compute accelerators, but a future driver could change it.

## Roadmap

- Linux backend (`/proc/stat`, `amdgpu`/`i915`, `accel/*`, `hwmon`)
- per-process NPU and GPU attribution
- a compact one-line mode for status bars
- signed release binaries

## License

MIT. See [LICENSE](LICENSE).
