# snaptop

**CPU, GPU, NPU and power in one terminal.** A fast, single-binary `top` for
Windows on Snapdragon — mactop-style panels, no SDK, no driver, no admin prompt.

```
snaptop v0.1.0                              Snapdragon X Elite X1E78100 · Windows 11 Enterprise 24H2
┌ CPU 12 cores (8P + 4E) · 12%  ────────────────────────────┐┌ GPU Adreno X1-85 · 5%  ─────────────┐
│P 8× 1%   E 4× 34%                         3.42 GHz · 2.7 W││5%                      0.72/1.44 GHz│
│ 0[███░░░░░]  39%  4[░░░░░░░░]   5%  8[░░░░░░░░]   0%      ││██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│ 1[███░░░░░]  37%  5[░░░░░░░░]   0%  9[░░░░░░░░]   0%      ││0.8/15.9 GiB · 3D 5%                 │
│ 2[███░░░░░]  36%  6[░░░░░░░░]   0% 10[░░░░░░░░]   0%      ││                                     │
│ 3[██░░░░░░]  23%  7[░░░░░░░░]   0% 11[░░░░░░░░]   0%      ││                                     │
│⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀││                                     │
│⢈⢈⢈⢈⣌⢈⢈⠀⢈⢈⢈⠀⠀⢈⢈⠀⠀⣌⣌⣮⣮⣌⣌⢈⢈⢈⢈⢈⢈⢈⣌⣌⣮⣌⣮⣿⣮⢈⣌⣌⣌⣮⣌⣌⣌⣌⢈⢈⢈⢈⢈⢈⠀⢈⢈⢈⠀⢈⢈││⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀│
│                                                           ││⠀⠀⠀⢈⠀⢈⠀⢈⠀⠀⠀⠀⢈⠀⢈⠀⠀⠀⢈⢈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀│
│                                                           │└─────────────────────────────────────┘
│                                                           │┌ NPU Hexagon · 0%  ──────────────────┐
│                                                           ││0%                               idle│
│                                                           ││░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│                                                           ││engine Compute 0%                    │
│                                                           ││                                     │
│                                                           ││⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀│
│                                                           ││⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀│
└───────────────────────────────────────────────────────────┘└─────────────────────────────────────┘
┌ MEMORY 17.7 / 31.6 GiB  ─────────────────────────────┐┌ POWER SYS 10.9 W  ───────────────────────┐
│56%                                     avail 13.9 GiB││CPU 2.7 W · GPU 0.0 W · PSU 0.0 W         │
│██████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░││BAT 99% · SoC 73 °C                       │
│commit 25.0 / 33.6 GiB · 75%                          ││                                          │
│                                                      ││⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣌⣌⣌⣌⣌⣌⣌⣌⠀⠀⠀⠀⠀⠀⠀⠀⠀⣌⣌⣌⣌⣌⣌⠀│
│⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀││⣌⣌⣌⣌⣌⣌⣌⣌⣌⢈⢈⢈⢈⢈⢈⢈⢈⢈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢈│
│⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿││⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿│
└──────────────────────────────────────────────────────┘└──────────────────────────────────────────┘
q quit · space pause · t theme · ? help                                                aurora · 1.0s
```

<sub>Real `snaptop --preview --theme dark` frame on a Snapdragon X Elite X1E78100.</sub>

## Install

```powershell
winget install 1duo.snaptop
```

Portable ZIP; winget puts `snaptop` on `PATH` and handles upgrades and removal.
The 0.1.0 manifest is submitted to `microsoft/winget-pkgs` with the first
release — until it is merged, build from source:

```powershell
git clone https://github.com/1duo/snaptop
cd snaptop
cargo build --release          # Rust 1.88+
copy target\release\snaptop.exe "$env:LOCALAPPDATA\Microsoft\WindowsApps\"
```

## Use

```
snaptop                       the dashboard
snaptop --once                one plain-text snapshot
snaptop --json --once         one JSON object
snaptop --json -i 0.5         NDJSON metric stream
snaptop --preview             one frame as text (bug reports)
snaptop --theme dark          auto (default), dark, light, mono
```

Keys: `q` quit · `space` pause · `+`/`-` speed · `r` reset history · `t` theme · `?` help.

`--theme auto` follows the OS app theme (aurora for dark, daylight for light).
The NPU is the MCDM accelerator Windows exposes as a LUID no DXGI adapter
claims; power comes from the Snapdragon energy-meter rails. Sources and design
notes: [docs/how-it-works.md](docs/how-it-works.md).

## Limits

- Windows only today; the Linux backend is the next milestone.
- Aggregate utilisation — no per-process attribution yet (`nptop` covers that).
- NPU power is not exposed by the hardware, and GPU clocks are optional.

## License

MIT.
