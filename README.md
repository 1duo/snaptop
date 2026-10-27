# snaptop

**CPU, GPU, NPU and power in one terminal.** A fast, single-binary `top` for
Windows on Snapdragon — mactop-style panels, no SDK, no driver, no admin prompt.

```
┌ snaptop v0.1.0 ──────────────────────────────────────────────────────────────────────────────────┐
│Snapdragon X Elite X1E78100                        Windows 11 Enterprise 24H2 · 12 cores (8P + 4E)│
│┌-CPU 12 cores (8P + 4E) · 37%  ──────────────────────────────┐┌-GPU Adreno X1-85 · 16%  ────────┐│
││P 8× 27%   E 4× 57%                         3.42 GHz · 13.5 W││░░░░░░  16%         0.72/1.44 GHz││
││ 0 [████░░]  61.5%  4 [███░░░]  53.8%  8 [███░░░]  46.3%     ││░░░░░░  0.7/15.9 GiB · 3D 16%    ││
││ 1 [███░░░]  55.9%  5 [██░░░░]  27.1%  9 [██░░░░]  25.4%     ││░░░░░░                           ││
││ 2 [███░░░]  56.8%  6 [█░░░░░]  17.9% 10 [█░░░░░]  14.8%     ││░░░░░░                           ││
││ 3 [███░░░]  55.2%  7 [█░░░░░]  11.9% 11 [█░░░░░]  15.8%     ││░░░░░░                           ││
││⠀⠀⠀⠀⠀⠀⠀⠀⠀⢈⠀⢈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣮⣿⣮⣮⢈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣌⢈⢈⠀⠀⠀⠀⠀⠀⠀⠀││░░░░░░  ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀││
││⢈⠀⠀⢈⢈⢈⢈⢈⣮⣿⣿⣿⣮⣮⣌⢈⢈⢈⣌⢈⢈⢈⢈⢈⢈⢈⢈⢈⣌⣌⣮⣮⣿⣿⣿⣿⣿⣿⣌⣮⣮⣿⣮⣿⣿⣮⣌⢈⣌⣮⣿⣿⣿⣮⣮⣌⢈⢈⣌⣌⣮││██████  ⣌⢈⢈⢈⢈⢈⢈⢈⣌⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈⢈││
││                                                             │└─────────────────────────────────┘│
││                                                             │┌-NPU Hexagon · 0%  ──────────────┐│
││                                                             ││░░░░░░  0%                   idle││
││                                                             ││░░░░░░  engine Compute 0%        ││
││                                                             ││░░░░░░                           ││
││                                                             ││░░░░░░  ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀││
││                                                             ││░░░░░░  ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀││
│└─────────────────────────────────────────────────────────────┘└─────────────────────────────────┘│
│┌-MEMORY 17.2 / 31.6 GiB  ────────────────────────────┐┌-POWER SYS 27.2 W  ──────────────────────┐│
││54%                                    avail 14.4 GiB││CPU 13.5 W · GPU 0.1 W · PSU 34.7 W      ││
││█████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░││BAT 91% AC · SoC 73 °C                   ││
││commit 24.8 / 33.6 GiB · 74%                         ││                                         ││
││                                                     ││     ████████        ███████████████████ ││
││⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀││█████████████████████████████████████████││
││⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿││█████████████████████████████████████████││
│└─────────────────────────────────────────────────────┘└─────────────────────────────────────────┘│
│q quit · space pause · t theme · ? help                                              aurora · 1.0s│
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
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
