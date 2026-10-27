//! snaptop — a fast, elegant top for Snapdragon laptops.
//!
//! The binary is deliberately small in scope: one sampler thread, one screen,
//! a stable JSON mode. Anything that is not "what is the chip doing right now"
//! belongs in a different tool.

mod app;
mod fmt;
mod history;
mod model;
mod platform;
mod theme;
mod ui;

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use app::App;
use model::{CoreKind, DeviceInfo, Snapshot};
use platform::Sampler;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Args {
    interval_ms: u64,
    once: bool,
    json: bool,
    preview: bool,
    count: Option<u64>,
    theme: usize,
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("snaptop: {message}");
            eprintln!("Try 'snaptop --help'.");
            std::process::exit(2);
        }
    };

    let result = if args.preview {
        run_preview(&args)
    } else if args.once || args.json || args.count.is_some() {
        run_headless(&args)
    } else {
        run_tui(&args)
    };

    if let Err(message) = result {
        eprintln!("snaptop: {message}");
        std::process::exit(1);
    }
}

// ------------------------------------------------------------- args --------

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        interval_ms: 1000,
        once: false,
        json: false,
        preview: false,
        count: None,
        theme: 0,
    };

    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("snaptop {VERSION}");
                std::process::exit(0);
            }
            "-1" | "--once" => args.once = true,
            "-j" | "--json" => args.json = true,
            "--preview" => args.preview = true,
            "-i" | "--interval" => {
                let value = it.next().ok_or("--interval needs a value")?;
                args.interval_ms = parse_interval(&value)?;
            }
            "-n" | "--count" => {
                let value = it.next().ok_or("--count needs a value")?;
                let n: u64 = value
                    .parse()
                    .map_err(|_| format!("not a sample count: {value}"))?;
                args.count = Some(n.max(1));
            }
            "--theme" => {
                let value = it.next().ok_or("--theme needs a value")?;
                args.theme = theme_index(&value)?;
            }
            other => {
                if let Some(value) = other.strip_prefix("--interval=") {
                    args.interval_ms = parse_interval(value)?;
                } else if let Some(value) = other.strip_prefix("--count=") {
                    args.count = Some(
                        value
                            .parse::<u64>()
                            .map_err(|_| format!("not a sample count: {value}"))?
                            .max(1),
                    );
                } else if let Some(value) = other.strip_prefix("--theme=") {
                    args.theme = theme_index(value)?;
                } else {
                    return Err(format!("unknown option: {other}"));
                }
            }
        }
    }
    Ok(args)
}

fn parse_interval(value: &str) -> Result<u64, String> {
    let seconds: f64 = value
        .parse()
        .map_err(|_| format!("not a number of seconds: {value}"))?;
    if !(0.1..=60.0).contains(&seconds) {
        return Err("interval must be between 0.1 and 60 seconds".to_string());
    }
    Ok((seconds * 1000.0).round() as u64)
}

fn theme_index(value: &str) -> Result<usize, String> {
    match value.to_ascii_lowercase().as_str() {
        "aurora" | "default" => Ok(0),
        "nebula" => Ok(1),
        "mono" => Ok(2),
        other => Err(format!("unknown theme: {other} (aurora, nebula, mono)")),
    }
}

fn print_help() {
    println!(
        "\
snaptop {VERSION} — CPU, GPU, NPU and power for Snapdragon laptops

USAGE
    snaptop [OPTIONS]

OPTIONS
    -i, --interval <SECS>   refresh interval, 0.1–60 (default 1)
    -n, --count <N>         stop after N samples
    -1, --once              print one sample and exit
    -j, --json              JSON output (NDJSON stream unless --once/--count)
        --preview           render one frame as plain text, then exit
        --theme <NAME>      aurora (default), nebula, mono
    -h, --help              show this help
    -V, --version           show the version

KEYS (interactive)
    q / esc                 quit
    space                   pause / resume
    + / -                   slower / faster
    r                       reset history
    t                       cycle theme
    ?                       help

WHAT IT READS
    CPU utilisation   \\Processor Information(*)\\% Processor Utility
    CPU topology      GetLogicalProcessorInformationEx (EfficiencyClass)
    GPU / NPU         \\GPU Engine(*)\\Utilization Percentage, grouped by LUID
    GPU clock         D3DKMTQueryAdapterInfo(KMTQAITYPE_NODEPERFDATA)
    Power             \\Energy Meter(*)\\Power (Snapdragon EMI rails)
    Thermal           \\Thermal Zone Information(*)\\Temperature

    The NPU is the MCDM compute accelerator: a LUID that appears in the GPU
    engine counters but belongs to no DXGI adapter. No driver or admin rights
    are required.

EXAMPLES
    snaptop                       the dashboard
    snaptop --once                one plain-text snapshot
    snaptop --json --once         one JSON object
    snaptop --json -i 0.5         metric stream for a script
"
    );
}

// --------------------------------------------------------- headless --------

/// Render a single real frame with ratatui's test backend and print it as
/// plain text. Terminal-agnostic, which makes it the right artifact for a bug
/// report and the honest source for the README screenshot.
fn run_preview(args: &Args) -> Result<(), String> {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let mut sampler = Sampler::new()?;
    let device = sampler.device();
    thread::sleep(Duration::from_millis(400));

    let mut app = App::new(device, args.interval_ms, args.theme);
    // A single sample would draw a flat line. Collect a short real trace so a
    // preview shows the dashboard doing its job.
    for _ in 0..200 {
        let snapshot = sampler.sample()?;
        app.update(snapshot);
        thread::sleep(Duration::from_millis(20));
    }
    // The energy meter is a rate counter; a 20 ms delta reads as zero. Let a
    // full second elapse before the sample that the numbers are taken from.
    thread::sleep(Duration::from_millis(1000));
    let snapshot = sampler.sample()?;
    app.update(snapshot);

    let (width, height) = (100u16, 28u16);
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).map_err(|e| e.to_string())?;
    terminal
        .draw(|frame| ui::draw(frame, &app))
        .map_err(|e| e.to_string())?;

    let buffer = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..height {
        for x in 0..width {
            out.push_str(buffer[(x, y)].symbol());
        }
        out.push('\n');
    }
    print!("{out}");
    Ok(())
}

fn run_headless(args: &Args) -> Result<(), String> {
    let mut sampler = Sampler::new()?;
    let device = sampler.device();

    // PDH rate counters need two collections separated by a real interval;
    // the energy meter itself updates at ~1 Hz.
    thread::sleep(Duration::from_millis(1050));

    let limit = args.count.unwrap_or(if args.once { 1 } else { u64::MAX });
    let mut seq = 0u64;
    loop {
        let snapshot = sampler.sample()?;
        if args.json {
            let line = snapshot_json(&device, &snapshot, seq);
            println!("{line}");
            let _ = std::io::stdout().flush();
        } else {
            print_plain(&device, &snapshot, seq == 0);
        }
        seq += 1;
        if seq >= limit {
            break;
        }
        thread::sleep(Duration::from_millis(args.interval_ms));
    }
    Ok(())
}

fn print_plain(device: &DeviceInfo, s: &Snapshot, header: bool) {
    if header {
        println!(
            "snaptop {VERSION} · {} · {} · {}",
            device.cpu_name,
            device.os_name,
            device.core_summary()
        );
        println!();
    }

    println!(
        "CPU  {} {:>4}   {} · {}",
        bar(s.cpu.util),
        fmt::pct(s.cpu.util),
        fmt::mhz(Some(s.cpu.cur_mhz)),
        fmt::watts(s.cpu.watts)
    );
    let mut split = Vec::new();
    if let Some(v) = s.cpu.util_of(CoreKind::Performance) {
        split.push(format!(
            "P {}× {}",
            s.cpu.count(CoreKind::Performance),
            fmt::pct(v)
        ));
    }
    if let Some(v) = s.cpu.util_of(CoreKind::Efficiency) {
        split.push(format!(
            "E {}× {}",
            s.cpu.count(CoreKind::Efficiency),
            fmt::pct(v)
        ));
    }
    if !split.is_empty() {
        println!("     {}", split.join("   "));
    }

    match s.gpu() {
        Some(gpu) => {
            println!(
                "GPU  {} {:>4}   {} · {:.1}/{:.1} GiB · {}",
                bar(gpu.util),
                fmt::pct(gpu.util),
                fmt::mhz_pair(gpu.cur_mhz, gpu.max_mhz),
                gpu.mem_used.unwrap_or(0) as f64 / 1073741824.0,
                gpu.mem_total.unwrap_or(0) as f64 / 1073741824.0,
                fmt::watts(gpu.watts)
            );
            let engines = gpu
                .engines
                .iter()
                .filter(|e| e.util >= 0.5)
                .map(|e| format!("{} {}", e.name, fmt::pct(e.util)))
                .collect::<Vec<_>>();
            if !engines.is_empty() {
                println!("     {} · {}", gpu.name, engines.join(" · "));
            }
        }
        None => println!("GPU  no adapter found"),
    }

    match s.npu() {
        Some(npu) => {
            println!(
                "NPU  {} {:>4}   {}",
                bar(npu.util),
                fmt::pct(npu.util),
                npu.name
            );
        }
        None => println!("NPU  not found"),
    }

    println!(
        "MEM  {} {:>4}   {}",
        bar(s.mem.pct()),
        fmt::pct(s.mem.pct()),
        fmt::gib_pair(s.mem.used, s.mem.total)
    );

    let mut tail = Vec::new();
    if let Some(w) = s.power.sys_w {
        tail.push(format!("SYS {}", fmt::watts(Some(w))));
    }
    if let Some(w) = s.power.cpu_w {
        tail.push(format!("CPU {}", fmt::watts(Some(w))));
    }
    if let Some(w) = s.power.gpu_w {
        tail.push(format!("GPU {}", fmt::watts(Some(w))));
    }
    if let Some(t) = s.thermal_c {
        tail.push(format!("SoC {}", fmt::temp(Some(t))));
    }
    if let Some(b) = s.power.battery_pct {
        let ac = match s.power.ac {
            Some(true) => " ⚡",
            _ => "",
        };
        tail.push(format!("BAT {b}%{ac}"));
    }
    if !tail.is_empty() {
        println!("     {}", tail.join(" · "));
    }
    println!();
}

fn bar(value: f32) -> String {
    const WIDTH: usize = 24;
    let filled = ((value.clamp(0.0, 100.0) / 100.0) * WIDTH as f32).round() as usize;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(WIDTH - filled))
}

fn snapshot_json(device: &DeviceInfo, s: &Snapshot, seq: u64) -> serde_json::Value {
    use serde_json::json;

    let gpus = s
        .gpus
        .iter()
        .map(|g| {
            json!({
                "name": g.name,
                "util_pct": round1(g.util),
                "clock_mhz": g.cur_mhz,
                "clock_max_mhz": g.max_mhz,
                "mem_used_bytes": g.mem_used,
                "mem_total_bytes": g.mem_total,
                "power_w": g.watts.map(round2),
                "engines": g.engines.iter()
                    .map(|e| json!({"name": e.name, "util_pct": round1(e.util)}))
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();

    let npus = s
        .npus
        .iter()
        .map(|n| {
            json!({
                "name": n.name,
                "util_pct": round1(n.util),
                "power_w": n.watts.map(round2),
                "engines": n.engines.iter()
                    .map(|e| json!({"name": e.name, "util_pct": round1(e.util)}))
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();

    json!({
        "seq": seq,
        "ts_ms": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        "version": VERSION,
        "device": {
            "cpu": device.cpu_name,
            "os": device.os_name,
            "build": device.os_build,
            "gpu": device.gpu_name,
            "npu": device.npu_name,
        },
        "cpu": {
            "util_pct": round1(s.cpu.util),
            "p_util_pct": s.cpu.util_of(CoreKind::Performance).map(round1),
            "e_util_pct": s.cpu.util_of(CoreKind::Efficiency).map(round1),
            "cur_mhz": s.cpu.cur_mhz,
            "max_mhz": s.cpu.max_mhz,
            "power_w": s.cpu.watts.map(round2),
            "cores": s.cpu.cores.iter().map(|c| json!({
                "id": c.id,
                "kind": c.kind.label(),
                "util_pct": round1(c.util),
                "cur_mhz": c.cur_mhz,
                "max_mhz": c.max_mhz,
            })).collect::<Vec<_>>(),
        },
        "gpus": gpus,
        "npus": npus,
        "memory": {
            "used_bytes": s.mem.used,
            "total_bytes": s.mem.total,
            "util_pct": round1(s.mem.pct()),
        },
        "power": {
            "sys_w": s.power.sys_w.map(round2),
            "psu_w": s.power.psu_w.map(round2),
            "cpu_w": s.power.cpu_w.map(round2),
            "gpu_w": s.power.gpu_w.map(round2),
            "battery_pct": s.power.battery_pct,
            "ac": s.power.ac,
        },
        "thermal_c": s.thermal_c.map(round1),
    })
}

fn round1(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

// -------------------------------------------------------------- TUI --------

fn run_tui(args: &Args) -> Result<(), String> {
    let (device, snapshots, interval, stop, sampler) = spawn_sampler(args.interval_ms)?;
    let mut app = App::new(device, args.interval_ms, args.theme);

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        hook(info);
    }));

    let mut terminal = ratatui::init();
    let result = tui_loop(&mut terminal, &mut app, &snapshots, &interval);
    ratatui::restore();

    stop.store(true, Ordering::Relaxed);
    let _ = sampler.join();
    result
}

fn tui_loop(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    snapshots: &Receiver<Snapshot>,
    interval: &Arc<AtomicU64>,
) -> Result<(), String> {
    let mut dirty = true;
    loop {
        if dirty {
            terminal
                .draw(|frame| ui::draw(frame, app))
                .map_err(|e| e.to_string())?;
            dirty = false;
        }

        if event::poll(Duration::from_millis(40)).map_err(|e| e.to_string())? {
            match event::read().map_err(|e| e.to_string())? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && matches!(key.code, KeyCode::Char('c'))
                    {
                        break;
                    }
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char(' ') => app.paused = !app.paused,
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            app.slower();
                            interval.store(app.interval_ms, Ordering::Relaxed);
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            app.faster();
                            interval.store(app.interval_ms, Ordering::Relaxed);
                        }
                        KeyCode::Char('r') => app.reset(),
                        KeyCode::Char('t') => app.theme = (app.theme + 1) % theme::PALETTES.len(),
                        KeyCode::Char('?') | KeyCode::Char('h') => app.help = !app.help,
                        _ => {}
                    }
                    dirty = true;
                }
                Event::Resize(_, _) => dirty = true,
                _ => {}
            }
        }

        // Newest sample wins; draining keeps the UI current after a stall.
        let mut latest = None;
        while let Ok(snapshot) = snapshots.try_recv() {
            latest = Some(snapshot);
        }
        if let Some(snapshot) = latest
            && !app.paused
        {
            app.update(snapshot);
            dirty = true;
        }
    }
    Ok(())
}

type SamplerThread = (
    DeviceInfo,
    Receiver<Snapshot>,
    Arc<AtomicU64>,
    Arc<AtomicBool>,
    thread::JoinHandle<()>,
);

fn spawn_sampler(interval_ms: u64) -> Result<SamplerThread, String> {
    let (device_tx, device_rx) = mpsc::channel::<Result<DeviceInfo, String>>();
    let (snapshot_tx, snapshot_rx) = mpsc::channel::<Snapshot>();
    let interval = Arc::new(AtomicU64::new(interval_ms));
    let stop = Arc::new(AtomicBool::new(false));

    let thread_interval = Arc::clone(&interval);
    let thread_stop = Arc::clone(&stop);

    let handle = thread::spawn(move || {
        let mut sampler = match Sampler::new() {
            Ok(sampler) => sampler,
            Err(message) => {
                let _ = device_tx.send(Err(message));
                return;
            }
        };
        if device_tx.send(Ok(sampler.device())).is_err() {
            return;
        }
        // The counter query was primed in `new()`, but PDH needs a real delta
        // before the first rate reads are meaningful.
        thread::sleep(Duration::from_millis(150));

        let mut last: Option<Instant> = None;
        loop {
            if thread_stop.load(Ordering::Relaxed) {
                break;
            }
            let period = Duration::from_millis(thread_interval.load(Ordering::Relaxed));
            let due = last.is_none_or(|t| t.elapsed() >= period);
            if due {
                if let Ok(snapshot) = sampler.sample()
                    && snapshot_tx.send(snapshot).is_err()
                {
                    break;
                }
                last = Some(Instant::now());
            } else {
                thread::sleep(Duration::from_millis(20));
            }
        }
    });

    let device = device_rx
        .recv()
        .map_err(|_| "sampler thread stopped unexpectedly".to_string())??;
    Ok((device, snapshot_rx, interval, stop, handle))
}
