# snaptop

**CPU, GPU, NPU and power in one terminal.** A fast, single-binary `top` for
Windows on Snapdragon — no SDK, no driver, no admin prompt.

```
CPU   7% █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  3.42 GHz · 5.0 W
      P 8× 4%   E 4× 14%   ▂▁▁▁ ▂···▁···
GPU   8% █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0.72/1.44 GHz · 0.0 W
      Adreno X1-85 · 0.7/15.9 GiB · 3D 8%
NPU   0% ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  idle · Hexagon NPU
MEM  52% ██████████████████████████████░░░░░░░░░░░░░  16.6 / 31.6 GiB
SYS 15.8 W · SoC 59 °C · BAT 100% AC
```

<sub>Condensed from a real `snaptop --preview` frame on a Snapdragon X Elite
X1E78100.</sub>

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
snaptop --theme nebula        aurora (default), nebula, mono
```

Keys: `q` quit · `space` pause · `+`/`-` speed · `r` reset history · `t` theme · `?` help.

The NPU is the MCDM accelerator Windows exposes as a LUID no DXGI adapter
claims; power comes from the Snapdragon energy-meter rails. Sources and design
notes: [docs/how-it-works.md](docs/how-it-works.md).

## Limits

- Windows only today; the Linux backend is the next milestone.
- Aggregate utilisation — no per-process attribution yet (`nptop` covers that).
- NPU power is not exposed by the hardware, and GPU clocks are optional.

## License

MIT.
