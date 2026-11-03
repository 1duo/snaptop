//! Linux backend: `/proc` and `/sys` only — no driver, no service.
//!
//! Everything here is a file read. Readers take a root path so the whole
//! backend runs against a fixture tree in tests; production uses `/`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::fmt;
use crate::model::{self, Core, CoreKind, Cpu, DeviceInfo, Engine, Gpu, Mem, Npu, Power, Snapshot};

pub struct Sampler {
    root: PathBuf,
    device: DeviceInfo,
    kinds: Vec<CoreKind>,
    clocks: Vec<Option<Clock>>,
    cpus: usize,
    prev: Ticks,
    gpu: Option<GpuSource>,
    npu: Option<String>,
    thermal: Vec<PathBuf>,
}

/// A cpufreq directory, either per-CPU or the policy that covers it.
#[derive(Clone, Debug)]
struct Clock {
    cur: PathBuf,
    max: PathBuf,
}

/// Busy and total jiffies, aggregate and per logical processor.
#[derive(Clone, Debug, Default)]
struct Ticks {
    busy: u64,
    total: u64,
    per: Vec<(u64, u64)>,
}

#[derive(Clone, Debug)]
struct GpuSource {
    kind: GpuKind,
    dir: PathBuf,
}

#[derive(Clone, Debug)]
enum GpuKind {
    /// Downstream Qualcomm `msm` exposes `kgsl-3d0`.
    Kgsl,
    /// `gpu_busy_percent` on a DRM card. amdgpu is the driver that exposes it;
    /// upstream msm does not (it reports usage through fdinfo).
    Card { name: String },
}

impl GpuSource {
    /// Prefer kgsl where it exists; otherwise take the first card that reports
    /// busy time. A GPU with no counter is left out rather than shown as idle.
    fn discover(root: &Path) -> Option<Self> {
        let kgsl = root.join("sys/class/kgsl/kgsl-3d0");
        if kgsl.join("gpu_busy_percentage").exists() {
            return Some(Self {
                kind: GpuKind::Kgsl,
                dir: kgsl,
            });
        }
        for card in entries(&root.join("sys/class/drm")) {
            if !is_card(&card) {
                continue;
            }
            let device = card.join("device");
            if device.join("gpu_busy_percent").exists() {
                return Some(Self {
                    kind: GpuKind::Card {
                        name: card_name(&device),
                    },
                    dir: device,
                });
            }
        }
        None
    }

    fn name(&self) -> String {
        match &self.kind {
            GpuKind::Kgsl => text(&self.dir.join("gpu_model"))
                .map(|raw| fmt::pretty_gpu_name(&raw))
                .unwrap_or_else(|| "Adreno GPU".to_string()),
            GpuKind::Card { name } => name.clone(),
        }
    }

    fn sample(&self) -> Option<Gpu> {
        let util = match &self.kind {
            GpuKind::Kgsl => percent(&self.dir.join("gpu_busy_percentage")),
            GpuKind::Card { .. } => percent(&self.dir.join("gpu_busy_percent")),
        }
        .unwrap_or(0.0);

        let (cur_mhz, max_mhz, mem_used, mem_total) = match &self.kind {
            GpuKind::Kgsl => (
                mhz_field(&self.dir.join("clock_mhz"))
                    .or_else(|| hz(&self.dir.join("gpuclk")))
                    .or_else(|| hz(&self.dir.join("gpu_clock"))),
                mhz_field(&self.dir.join("max_clock_mhz"))
                    .or_else(|| hz(&self.dir.join("max_gpuclk"))),
                None,
                None,
            ),
            GpuKind::Card { .. } => (
                None,
                None,
                number(&self.dir.join("mem_info_vram_used")),
                number(&self.dir.join("mem_info_vram_total")),
            ),
        };

        Some(Gpu {
            name: self.name(),
            util,
            engines: vec![Engine {
                name: "3D".to_string(),
                util,
            }],
            cur_mhz: cur_mhz.filter(|mhz| *mhz > 0),
            max_mhz: max_mhz.filter(|mhz| *mhz > 0),
            mem_used: mem_used.filter(|bytes| *bytes > 0),
            mem_total: mem_total.filter(|bytes| *bytes > 0),
            watts: None,
        })
    }
}

impl Sampler {
    pub fn new() -> Result<Self, String> {
        Self::with_root(Path::new("/"))
    }

    fn with_root(root: &Path) -> Result<Self, String> {
        let prev = cpu_ticks(root);
        let cpus = prev.per.len().max(1);
        let cpu_name = board_name(root);
        let mut kinds = core_kinds(root, cpus);
        model::normalize_kinds(&identity(root, &cpu_name), &mut kinds);
        let gpu = GpuSource::discover(root);
        let npu = npu_name(root);

        let p_cores = kinds
            .iter()
            .filter(|kind| **kind == CoreKind::Performance)
            .count();
        let e_cores = kinds
            .iter()
            .filter(|kind| **kind == CoreKind::Efficiency)
            .count();

        let device = DeviceInfo {
            cpu_name,
            os_name: os_release(root),
            os_build: kernel_release(root),
            cpu_count: cpus,
            p_cores,
            e_cores,
            gpu_name: gpu
                .as_ref()
                .map(GpuSource::name)
                .unwrap_or_else(|| "GPU".to_string()),
            npu_name: npu.clone().unwrap_or_else(|| "NPU".to_string()),
        };

        Ok(Self {
            root: root.to_path_buf(),
            device,
            kinds,
            clocks: clocks(root, cpus),
            cpus,
            prev,
            gpu,
            npu,
            thermal: thermal_zones(root),
        })
    }

    pub fn device(&self) -> DeviceInfo {
        self.device.clone()
    }

    pub fn sample(&mut self) -> Result<Snapshot, String> {
        let ticks = cpu_ticks(&self.root);
        let mut snapshot = Snapshot {
            cpu: self.cpu(&ticks),
            ..Default::default()
        };
        self.prev = ticks;

        if let Some(gpu) = self.gpu.as_ref().and_then(GpuSource::sample) {
            snapshot.gpus.push(gpu);
        }
        if let Some(name) = &self.npu {
            snapshot.npus.push(Npu {
                name: name.clone(),
                util: None,
                engines: Vec::new(),
                watts: None,
            });
        }
        snapshot.mem = memory(&self.root);
        snapshot.power = power(&self.root);
        snapshot.thermal_c = hottest(&self.thermal);
        Ok(snapshot)
    }

    fn cpu(&self, ticks: &Ticks) -> Cpu {
        let mut cpu = Cpu {
            util: utilization((self.prev.busy, self.prev.total), (ticks.busy, ticks.total)),
            ..Default::default()
        };
        for id in 0..self.cpus {
            let previous = self.prev.per.get(id).copied().unwrap_or((0, 0));
            let current = ticks.per.get(id).copied().unwrap_or((0, 0));
            let (cur_mhz, max_mhz) = match self.clocks.get(id).and_then(Option::as_ref) {
                Some(clock) => (khz_to_mhz(&clock.cur), khz_to_mhz(&clock.max)),
                None => (0, 0),
            };
            cpu.cores.push(Core {
                id,
                kind: self.kinds.get(id).copied().unwrap_or(CoreKind::Unknown),
                util: utilization(previous, current),
                cur_mhz,
                max_mhz,
            });
        }
        cpu.cur_mhz = cpu.cores.iter().map(|core| core.cur_mhz).max().unwrap_or(0);
        cpu.max_mhz = cpu.cores.iter().map(|core| core.max_mhz).max().unwrap_or(0);
        cpu
    }
}

fn read(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// Trimmed text, treating NUL-terminated device-tree strings as text.
fn text(path: &Path) -> Option<String> {
    let raw = read(path)?;
    let trimmed = raw.trim_matches(|c: char| c.is_control() || c.is_whitespace());
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn number(path: &Path) -> Option<u64> {
    text(path)?.parse().ok()
}

/// kgsl reports `gpuclk` and `max_gpuclk` in hertz.
fn hz(path: &Path) -> Option<u32> {
    number(path)
        .map(|hz| (hz / 1_000_000) as u32)
        .filter(|mhz| *mhz > 0)
}

/// A `*_mhz` attribute is already in megahertz.
fn mhz_field(path: &Path) -> Option<u32> {
    number(path).map(|mhz| mhz as u32).filter(|mhz| *mhz > 0)
}

/// cpufreq attributes are in kilohertz; a missing clock reads as zero.
fn khz_to_mhz(path: &Path) -> u32 {
    number(path).map(|khz| (khz / 1000) as u32).unwrap_or(0)
}

/// `"37 %"`, `"37"` and `"37.5"` all parse; anything else is not a counter.
fn percent(path: &Path) -> Option<f32> {
    let raw = text(path)?;
    let value: f32 = raw.trim_end_matches('%').trim().parse().ok()?;
    (0.0..=1000.0).contains(&value).then_some(value)
}

fn entries(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(read) => read
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect(),
        Err(_) => return Vec::new(),
    };
    paths.sort();
    paths
}

fn is_card(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("card"))
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
}

/// `/proc/stat`: one aggregate `cpu` line, then one `cpuN` line per processor.
fn cpu_ticks(root: &Path) -> Ticks {
    let Some(stat) = read(&root.join("proc/stat")) else {
        return Ticks::default();
    };
    let mut ticks = Ticks::default();
    for line in stat.lines() {
        let Some(rest) = line.strip_prefix("cpu") else {
            continue;
        };
        let mut tokens = rest.split_whitespace();
        if rest.starts_with(|c: char| c.is_whitespace()) {
            if let Some((busy, total)) = sum_tokens(tokens) {
                ticks.busy = busy;
                ticks.total = total;
            }
            continue;
        }
        let Some(id) = tokens.next().and_then(|token| token.parse::<usize>().ok()) else {
            continue;
        };
        let Some((busy, total)) = sum_tokens(tokens) else {
            continue;
        };
        if ticks.per.len() <= id {
            ticks.per.resize(id + 1, (0, 0));
        }
        ticks.per[id] = (busy, total);
    }
    ticks
}

/// user, nice, system, idle, iowait, irq, softirq, steal. `guest` is already
/// counted inside `user`, so summing more would double count.
fn sum_tokens<'a>(tokens: impl Iterator<Item = &'a str>) -> Option<(u64, u64)> {
    let mut values = [0u64; 8];
    let mut seen = 0;
    for (slot, token) in values.iter_mut().zip(tokens) {
        *slot = token.parse().ok()?;
        seen += 1;
    }
    if seen < 4 {
        return None;
    }
    let idle = values[3].saturating_add(values[4]);
    let total: u64 = values.iter().sum();
    Some((total.saturating_sub(idle), total))
}

fn utilization(previous: (u64, u64), current: (u64, u64)) -> f32 {
    let busy = current.0.saturating_sub(previous.0);
    let total = current.1.saturating_sub(previous.1);
    if total == 0 {
        0.0
    } else {
        (busy as f32 * 100.0 / total as f32).clamp(0.0, 100.0)
    }
}

fn clock_in(dir: &Path) -> Option<Clock> {
    let cur = ["scaling_cur_freq", "cpuinfo_cur_freq"]
        .into_iter()
        .map(|name| dir.join(name))
        .find(|path| number(path).is_some())?;
    let max = ["cpuinfo_max_freq", "scaling_max_freq"]
        .into_iter()
        .map(|name| dir.join(name))
        .find(|path| number(path).is_some())?;
    Some(Clock { cur, max })
}

/// Per-CPU cpufreq directories when the kernel exposes them, otherwise the
/// policy that lists the CPU in `affected_cpus`.
fn clocks(root: &Path, cpus: usize) -> Vec<Option<Clock>> {
    let cpu = root.join("sys/devices/system/cpu");
    let mut out: Vec<Option<Clock>> = (0..cpus)
        .map(|id| clock_in(&cpu.join(format!("cpu{id}/cpufreq"))))
        .collect();
    if out.iter().all(Option::is_some) {
        return out;
    }
    for policy in entries(&cpu.join("cpufreq")) {
        let Some(clock) = clock_in(&policy) else {
            continue;
        };
        let affected = text(&policy.join("affected_cpus")).unwrap_or_default();
        for id in affected
            .split_whitespace()
            .filter_map(|id| id.parse::<usize>().ok())
        {
            if id < cpus && out[id].is_none() {
                out[id] = Some(clock.clone());
            }
        }
    }
    out
}

/// `"0-3,8"` -> `[0, 1, 2, 3, 8]`.
fn cpu_list(path: &Path) -> Vec<usize> {
    let Some(raw) = text(path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for part in raw.split(',') {
        let part = part.trim();
        match part.split_once('-') {
            Some((low, high)) => {
                let (Ok(low), Ok(high)) =
                    (low.trim().parse::<usize>(), high.trim().parse::<usize>())
                else {
                    continue;
                };
                for id in low..=high.min(low.saturating_add(1024)) {
                    out.push(id);
                }
            }
            None => {
                if let Ok(id) = part.parse::<usize>() {
                    out.push(id);
                }
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Topology, most authoritative source first: Intel hybrid CPU masks, then the
/// arm64 DSU `cpu_capacity`. A single tier means "no split", not "all P".
fn core_kinds(root: &Path, cpus: usize) -> Vec<CoreKind> {
    let performance = cpu_list(&root.join("sys/devices/cpu_core/cpus"));
    let efficiency = cpu_list(&root.join("sys/devices/cpu_atom/cpus"));
    if !performance.is_empty() || !efficiency.is_empty() {
        return (0..cpus)
            .map(|id| {
                if performance.contains(&id) {
                    CoreKind::Performance
                } else if efficiency.contains(&id) {
                    CoreKind::Efficiency
                } else {
                    CoreKind::Unknown
                }
            })
            .collect();
    }

    let cpu = root.join("sys/devices/system/cpu");
    let capacities: Vec<Option<u64>> = (0..cpus)
        .map(|id| number(&cpu.join(format!("cpu{id}/cpu_capacity"))))
        .collect();
    let mut tiers: Vec<u64> = capacities.iter().flatten().copied().collect();
    tiers.sort_unstable();
    tiers.dedup();
    if tiers.len() < 2 {
        return vec![CoreKind::Unknown; cpus];
    }
    let top = tiers[tiers.len() - 1];
    capacities
        .iter()
        .map(|capacity| match capacity {
            Some(value) if *value == top => CoreKind::Performance,
            Some(_) => CoreKind::Efficiency,
            None => CoreKind::Unknown,
        })
        .collect()
}

fn meminfo(text: &str, name: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let (key, rest) = line.split_once(':')?;
        if key != name {
            return None;
        }
        rest.split_whitespace().next()?.parse().ok()
    })
}

fn memory(root: &Path) -> Mem {
    let Some(info) = read(&root.join("proc/meminfo")) else {
        return Mem::default();
    };
    let kib = |name: &str| meminfo(&info, name).unwrap_or(0).saturating_mul(1024);
    let total = kib("MemTotal");
    let available = kib("MemAvailable");
    Mem {
        used: total.saturating_sub(available),
        total,
        available,
        commit_used: kib("Committed_AS"),
        commit_total: kib("CommitLimit"),
    }
}

fn thermal_zones(root: &Path) -> Vec<PathBuf> {
    entries(&root.join("sys/class/thermal"))
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("thermal_zone"))
        })
        .collect()
}

fn hottest(zones: &[PathBuf]) -> Option<f32> {
    zones
        .iter()
        .filter_map(|zone| number(&zone.join("temp")))
        .map(|millidegrees| millidegrees as f32 / 1000.0)
        .filter(|celsius| (1.0..=150.0).contains(celsius))
        .reduce(f32::max)
}

/// Battery and charger nodes. Qualcomm laptops expose `qcom-battmgr-*` here;
/// the rails are `power_now` in microwatts.
fn power(root: &Path) -> Power {
    let mut power = Power::default();
    let mut battery_watts = 0.0f32;
    let mut battery_seen = false;
    let mut battery_status = String::new();
    let mut supply_watts = 0.0f32;
    let mut supply_seen = false;
    let mut online_seen = false;
    let mut online = false;

    for dir in entries(&root.join("sys/class/power_supply")) {
        let kind = text(&dir.join("type")).unwrap_or_default();
        match kind.as_str() {
            "Battery" => {
                if let Some(capacity) = number(&dir.join("capacity")) {
                    power.battery_pct = Some(capacity.min(100) as u8);
                }
                if let Some(watts) = watts_from(&dir) {
                    battery_watts += watts;
                    battery_seen = true;
                }
                battery_status = text(&dir.join("status")).unwrap_or_default();
            }
            // AC line status comes from the mains adapter only: a USB port
            // reports `online` for gadget roles too, and a board charger IC can
            // read 0 while the board is powered.
            "Mains" => {
                if let Some(value) = number(&dir.join("online")) {
                    online_seen = true;
                    online |= value == 1;
                }
                if let Some(watts) = watts_from(&dir) {
                    supply_watts += watts;
                    supply_seen = true;
                }
            }
            "USB" | "USB_C" | "USB_PD" => {
                if let Some(watts) = watts_from(&dir) {
                    supply_watts += watts;
                    supply_seen = true;
                }
            }
            _ => {}
        }
    }

    // While charging, the battery rail measures charge current, not system
    // draw; only a discharging battery is whole-system power.
    let discharging = battery_status.eq_ignore_ascii_case("Discharging");
    if battery_seen && (discharging || battery_status.is_empty()) {
        power.sys_w = Some(battery_watts);
    }
    if supply_seen {
        power.psu_w = Some(supply_watts);
    }
    if online_seen {
        power.ac = Some(online);
    }
    power
}

fn watts_from(dir: &Path) -> Option<f32> {
    if let Some(microwatts) = number(&dir.join("power_now")) {
        return Some(microwatts as f32 / 1_000_000.0);
    }
    let microvolts = number(&dir.join("voltage_now"))?;
    let microamps = number(&dir.join("current_now"))?;
    // uV * uA = pW
    Some((microvolts as f64 * microamps as f64 / 1e12) as f32)
}

fn card_name(device: &Path) -> String {
    match text(&device.join("vendor")).as_deref() {
        Some("0x1002") => "AMD GPU".to_string(),
        Some("0x10de") => "NVIDIA GPU".to_string(),
        Some("0x8086") => "Intel GPU".to_string(),
        _ => "GPU".to_string(),
    }
}

/// A CDSP fastrpc node or an `accel` device means a Hexagon-class NPU is up.
fn npu_name(root: &Path) -> Option<String> {
    let fastrpc = entries(&root.join("dev")).iter().any(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("fastrpc-cdsp"))
    });
    let accel = entries(&root.join("sys/class/accel")).iter().any(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("accel"))
    });
    if !fastrpc && !accel {
        return None;
    }
    let qualcomm = text(&root.join("proc/device-tree/compatible"))
        .is_some_and(|compatible| compatible.starts_with("qcom,"));
    Some(if qualcomm {
        "Hexagon NPU".to_string()
    } else {
        "NPU".to_string()
    })
}

/// Device-tree model first (arm64 boards and laptops that boot with a DTB),
/// then DMI, then `/proc/cpuinfo`.
fn board_name(root: &Path) -> String {
    for path in ["proc/device-tree/model", "sys/class/dmi/id/product_name"] {
        if let Some(raw) = text(&root.join(path)) {
            let name = tidy_board(&raw);
            if !name.is_empty() {
                return name;
            }
        }
    }
    cpuinfo_name(root).unwrap_or_else(|| "Linux".to_string())
}

fn tidy_board(raw: &str) -> String {
    let name = fmt::pretty_cpu_name(raw);
    let name = name
        .strip_prefix("Qualcomm Technologies, Inc.")
        .unwrap_or(&name)
        .trim();
    name.to_string()
}

/// What the machine calls itself: the board name plus the SoC compatible
/// string, so a reference board whose product name omits the part still folds
/// its Prime + Performance tiers into one class.
fn identity(root: &Path, board: &str) -> String {
    match text(&root.join("proc/device-tree/compatible")) {
        Some(soc) => format!("{board} {soc}"),
        None => board.to_string(),
    }
}

fn cpuinfo_name(root: &Path) -> Option<String> {
    let info = read(&root.join("proc/cpuinfo"))?;
    let wanted = ["Hardware", "model name", "Processor"];
    let mut pick = None;
    for line in info.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        if wanted.contains(&key) {
            let value = value.trim();
            if !value.is_empty() {
                pick = Some(if key == "model name" {
                    fmt::pretty_cpu_name(value)
                } else {
                    value.to_string()
                });
                if key == "Hardware" {
                    break;
                }
            }
        }
    }
    pick
}

fn os_release(root: &Path) -> String {
    read(&root.join("etc/os-release"))
        .and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("PRETTY_NAME=").map(str::to_string))
        })
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Linux".to_string())
}

fn kernel_release(root: &Path) -> String {
    text(&root.join("proc/sys/kernel/osrelease")).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway filesystem root, so every reader runs without hardware.
    struct Fixture {
        dir: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("snaptop-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("fixture root");
            Self { dir }
        }

        fn write(&self, relative: &str, contents: &str) -> &Self {
            let path = self.dir.join(relative);
            fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture parent");
            fs::write(path, contents).expect("fixture file");
            self
        }

        fn root(&self) -> &Path {
            &self.dir
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    const STAT_A: &str = "\
cpu  400 0 400 3200 0 0 0 0 0 0
cpu0 100 0 100 800 0 0 0 0 0 0
cpu1 100 0 100 800 0 0 0 0 0 0
cpu2 100 0 100 800 0 0 0 0 0 0
cpu3 100 0 100 800 0 0 0 0 0 0
";

    const STAT_B: &str = "\
cpu  600 0 600 3600 0 0 0 0 0 0
cpu0 150 0 150 900 0 0 0 0 0 0
cpu1 150 0 150 900 0 0 0 0 0 0
cpu2 150 0 150 900 0 0 0 0 0 0
cpu3 150 0 150 900 0 0 0 0 0 0
";

    fn four_cores(name: &str) -> Fixture {
        let fixture = Fixture::new(name);
        fixture.write("proc/stat", STAT_A);
        for id in 0..4 {
            fixture.write(
                &format!("sys/devices/system/cpu/cpu{id}/cpufreq/cpuinfo_max_freq"),
                "2361600\n",
            );
            fixture.write(
                &format!("sys/devices/system/cpu/cpu{id}/cpufreq/scaling_cur_freq"),
                "1180800\n",
            );
        }
        fixture
    }

    #[test]
    fn cpu_utilization_is_a_delta_between_samples() {
        let fixture = four_cores("cpu-delta");
        let mut sampler = Sampler::with_root(fixture.root()).expect("sampler");
        fixture.write("proc/stat", STAT_B);

        let snapshot = sampler.sample().expect("sample");
        assert_eq!(snapshot.cpu.cores.len(), 4);
        assert!(
            (snapshot.cpu.util - 50.0).abs() < 0.5,
            "{}",
            snapshot.cpu.util
        );
        for core in &snapshot.cpu.cores {
            assert!(
                (core.util - 50.0).abs() < 0.5,
                "core {} {}",
                core.id,
                core.util
            );
            assert_eq!(core.cur_mhz, 1180);
            assert_eq!(core.max_mhz, 2361);
        }
        assert_eq!(snapshot.cpu.cur_mhz, 1180);
        assert_eq!(snapshot.cpu.max_mhz, 2361);
    }

    #[test]
    fn a_stale_sample_does_not_invent_load() {
        let fixture = four_cores("cpu-stale");
        let mut sampler = Sampler::with_root(fixture.root()).expect("sampler");
        assert_eq!(sampler.sample().expect("sample").cpu.util, 0.0);
    }

    #[test]
    fn two_capacity_tiers_become_p_and_e() {
        let fixture = four_cores("capacity-two");
        for (id, capacity) in [(0, 1024), (1, 1024), (2, 768), (3, 768)] {
            fixture.write(
                &format!("sys/devices/system/cpu/cpu{id}/cpu_capacity"),
                &format!("{capacity}\n"),
            );
        }

        let sampler = Sampler::with_root(fixture.root()).expect("sampler");
        let device = sampler.device();
        assert_eq!((device.p_cores, device.e_cores), (2, 2));
        assert_eq!(device.core_summary(), "4 cores (2P + 2E)");
    }

    #[test]
    fn one_capacity_tier_is_not_a_p_e_split() {
        let fixture = four_cores("capacity-one");
        for id in 0..4 {
            fixture.write(
                &format!("sys/devices/system/cpu/cpu{id}/cpu_capacity"),
                "1024\n",
            );
        }

        let sampler = Sampler::with_root(fixture.root()).expect("sampler");
        let device = sampler.device();
        assert_eq!((device.p_cores, device.e_cores), (0, 0));
        assert_eq!(device.core_summary(), "4 cores");
    }

    #[test]
    fn x2_capacity_tiers_fold_into_performance() {
        let fixture = four_cores("capacity-x2");
        fixture.write("proc/device-tree/model", "Snapdragon X2 Elite X2E-88-100\0");
        for (id, capacity) in [(0, 1024), (1, 1024), (2, 768), (3, 768)] {
            fixture.write(
                &format!("sys/devices/system/cpu/cpu{id}/cpu_capacity"),
                &format!("{capacity}\n"),
            );
        }

        let sampler = Sampler::with_root(fixture.root()).expect("sampler");
        let device = sampler.device();
        assert_eq!((device.p_cores, device.e_cores), (4, 0));
        assert_eq!(device.core_summary(), "4 cores");
    }

    #[test]
    fn the_soc_compatible_string_can_identify_an_x2() {
        let fixture = four_cores("capacity-x2-crd");
        fixture.write(
            "proc/device-tree/model",
            "Qualcomm Technologies, Inc. Snapdragon X2 Elite CRD\0",
        );
        fixture.write("proc/device-tree/compatible", "qcom,x2e94100-crd\0");
        for (id, capacity) in [(0, 1024), (1, 1024), (2, 768), (3, 768)] {
            fixture.write(
                &format!("sys/devices/system/cpu/cpu{id}/cpu_capacity"),
                &format!("{capacity}\n"),
            );
        }

        let sampler = Sampler::with_root(fixture.root()).expect("sampler");
        assert_eq!(sampler.device().core_summary(), "4 cores");
    }

    #[test]
    fn intel_hybrid_masks_split_p_and_e() {
        let fixture = four_cores("intel-hybrid");
        fixture.write("sys/devices/cpu_core/cpus", "0-1\n");
        fixture.write("sys/devices/cpu_atom/cpus", "2,3\n");

        let sampler = Sampler::with_root(fixture.root()).expect("sampler");
        assert_eq!(sampler.device().core_summary(), "4 cores (2P + 2E)");
    }

    #[test]
    fn the_policy_directory_backs_up_missing_per_cpu_clocks() {
        let fixture = Fixture::new("policy-clocks");
        fixture.write("proc/stat", STAT_A);
        fixture.write(
            "sys/devices/system/cpu/cpufreq/policy0/cpuinfo_max_freq",
            "2400000\n",
        );
        fixture.write(
            "sys/devices/system/cpu/cpufreq/policy0/scaling_cur_freq",
            "1200000\n",
        );
        fixture.write(
            "sys/devices/system/cpu/cpufreq/policy0/affected_cpus",
            "0 1 2 3\n",
        );

        let sampler = Sampler::with_root(fixture.root()).expect("sampler");
        assert_eq!(sampler.device().cpu_count, 4);
        assert_eq!(
            sampler.clocks[2]
                .as_ref()
                .map(|clock| khz_to_mhz(&clock.cur)),
            Some(1200)
        );
    }

    #[test]
    fn memory_comes_from_meminfo() {
        let fixture = Fixture::new("meminfo");
        fixture.write(
            "proc/meminfo",
            "MemTotal:       16000000 kB\n\
             MemFree:         1000000 kB\n\
             MemAvailable:    4000000 kB\n\
             CommitLimit:     8000000 kB\n\
             Committed_AS:    3000000 kB\n",
        );

        let mem = memory(fixture.root());
        assert_eq!(mem.total, 16_000_000 * 1024);
        assert_eq!(mem.available, 4_000_000 * 1024);
        assert_eq!(mem.used, 12_000_000 * 1024);
        assert_eq!(mem.commit_used, 3_000_000 * 1024);
        assert_eq!(mem.commit_total, 8_000_000 * 1024);
    }

    #[test]
    fn the_hottest_plausible_zone_wins() {
        let fixture = Fixture::new("thermal");
        for (id, temp) in [
            (0, "45000"),
            (1, "0"),
            (2, "-5000"),
            (3, "not-a-temperature"),
            (4, "61250"),
        ] {
            fixture.write(&format!("sys/class/thermal/thermal_zone{id}/temp"), temp);
        }
        fixture.write("sys/class/thermal/thermal_zone0/type", "soc-therm\n");

        let zones = thermal_zones(fixture.root());
        assert_eq!(zones.len(), 5);
        assert_eq!(hottest(&zones), Some(61.25));
    }

    #[test]
    fn a_discharging_battery_is_system_power() {
        let fixture = Fixture::new("power-battery");
        fixture.write("sys/class/power_supply/BAT0/type", "Battery\n");
        fixture.write("sys/class/power_supply/BAT0/capacity", "87\n");
        fixture.write("sys/class/power_supply/BAT0/status", "Discharging\n");
        fixture.write("sys/class/power_supply/BAT0/power_now", "12300000\n");
        fixture.write("sys/class/power_supply/ADP1/type", "Mains\n");
        fixture.write("sys/class/power_supply/ADP1/online", "1\n");
        fixture.write("sys/class/power_supply/ADP1/power_now", "33000000\n");

        let power = power(fixture.root());
        assert_eq!(power.sys_w, Some(12.3));
        assert_eq!(power.psu_w, Some(33.0));
        assert_eq!(power.battery_pct, Some(87));
        assert_eq!(power.ac, Some(true));
    }

    #[test]
    fn a_charging_battery_is_not_system_power() {
        let fixture = Fixture::new("power-charging");
        fixture.write("sys/class/power_supply/BAT0/type", "Battery\n");
        fixture.write("sys/class/power_supply/BAT0/capacity", "60\n");
        fixture.write("sys/class/power_supply/BAT0/status", "Charging\n");
        fixture.write("sys/class/power_supply/BAT0/power_now", "5000000\n");

        let power = power(fixture.root());
        assert_eq!(power.sys_w, None);
        assert_eq!(power.psu_w, None);
        assert_eq!(power.battery_pct, Some(60));
        assert_eq!(power.ac, None);
    }

    #[test]
    fn voltage_times_current_falls_back_to_watts() {
        let fixture = Fixture::new("power-units");
        fixture.write("sys/class/power_supply/BAT0/type", "Battery\n");
        fixture.write("sys/class/power_supply/BAT0/status", "Discharging\n");
        fixture.write("sys/class/power_supply/BAT0/voltage_now", "12000000\n");
        fixture.write("sys/class/power_supply/BAT0/current_now", "1000000\n");

        assert_eq!(power(fixture.root()).sys_w, Some(12.0));
    }

    #[test]
    fn a_usb_port_is_not_an_ac_line() {
        let fixture = Fixture::new("power-usb");
        for id in 0..2 {
            fixture.write(
                &format!("sys/class/power_supply/usb-charger-{id}/type"),
                "USB\n",
            );
            fixture.write(
                &format!("sys/class/power_supply/usb-charger-{id}/online"),
                "0\n",
            );
        }

        let power = power(fixture.root());
        assert_eq!(power.ac, None);
        assert_eq!(power.sys_w, None);
    }

    #[test]
    fn a_gpu_without_a_busy_counter_is_left_out() {
        let fixture = four_cores("gpu-no-counter");
        fixture.write("sys/class/kgsl/kgsl-3d0/gpu_model", "Adreno663v1\n");

        let mut sampler = Sampler::with_root(fixture.root()).expect("sampler");
        assert!(sampler.sample().expect("sample").gpu().is_none());
    }

    #[test]
    fn kgsl_gpu_is_read() {
        let fixture = four_cores("kgsl");
        let kgsl = "sys/class/kgsl/kgsl-3d0";
        fixture.write(&format!("{kgsl}/gpu_model"), "Adreno663v1\n");
        fixture.write(&format!("{kgsl}/gpu_busy_percentage"), "37 %\n");
        fixture.write(&format!("{kgsl}/clock_mhz"), "405\n");
        fixture.write(&format!("{kgsl}/max_clock_mhz"), "800\n");

        let mut sampler = Sampler::with_root(fixture.root()).expect("sampler");
        assert_eq!(sampler.device().gpu_name, "Adreno 663");

        let snapshot = sampler.sample().expect("sample");
        let gpu = snapshot.gpu().expect("gpu");
        assert_eq!(gpu.util, 37.0);
        assert_eq!(gpu.cur_mhz, Some(405));
        assert_eq!(gpu.max_mhz, Some(800));
        assert_eq!(gpu.engines[0].name, "3D");
        assert_eq!(gpu.mem_total, None);
    }

    #[test]
    fn older_kgsl_reports_hertz() {
        let fixture = four_cores("kgsl-hz");
        let kgsl = "sys/class/kgsl/kgsl-3d0";
        fixture.write(&format!("{kgsl}/gpu_busy_percentage"), "0 %\n");
        fixture.write(&format!("{kgsl}/gpuclk"), "405000000\n");
        fixture.write(&format!("{kgsl}/max_gpuclk"), "800000000\n");

        let mut sampler = Sampler::with_root(fixture.root()).expect("sampler");
        let snapshot = sampler.sample().expect("sample");
        let gpu = snapshot.gpu().expect("gpu");
        assert_eq!(gpu.cur_mhz, Some(405));
        assert_eq!(gpu.max_mhz, Some(800));
    }

    #[test]
    fn a_drm_card_is_the_fallback() {
        let fixture = four_cores("drm");
        let card = "sys/class/drm/card0/device";
        fixture.write(&format!("{card}/gpu_busy_percent"), "12\n");
        fixture.write(&format!("{card}/vendor"), "0x1002\n");
        fixture.write(&format!("{card}/mem_info_vram_used"), "1073741824\n");
        fixture.write(&format!("{card}/mem_info_vram_total"), "8589934592\n");
        fixture.write("sys/class/drm/card0-DP-1/status", "connected\n");

        let mut sampler = Sampler::with_root(fixture.root()).expect("sampler");
        assert_eq!(sampler.device().gpu_name, "AMD GPU");

        let snapshot = sampler.sample().expect("sample");
        let gpu = snapshot.gpu().expect("gpu");
        assert_eq!(gpu.util, 12.0);
        assert_eq!(gpu.mem_used, Some(1073741824));
        assert_eq!(gpu.mem_total, Some(8589934592));
    }

    #[test]
    fn board_name_prefers_the_device_tree() {
        let fixture = Fixture::new("board-name");
        fixture.write(
            "proc/device-tree/model",
            "Qualcomm Technologies, Inc. IQ-9075 EVK\0",
        );
        fixture.write("proc/device-tree/compatible", "qcom,qcs9075-iq-9075-evk\0");
        fixture.write("sys/class/dmi/id/product_name", "21N2S01F00\n");
        fixture.write("etc/os-release", "PRETTY_NAME=\"Ubuntu 24.04.3 LTS\"\n");
        fixture.write("proc/sys/kernel/osrelease", "6.8.0-1084-qcom\n");

        let sampler = Sampler::with_root(fixture.root()).expect("sampler");
        let device = sampler.device();
        assert_eq!(device.cpu_name, "IQ-9075 EVK");
        assert_eq!(device.os_name, "Ubuntu 24.04.3 LTS");
        assert_eq!(device.os_build, "6.8.0-1084-qcom");
    }

    #[test]
    fn npu_is_present_but_has_no_counter() {
        let fixture = four_cores("npu");
        fixture.write("dev/fastrpc-cdsp", "");
        fixture.write("proc/device-tree/compatible", "qcom,qcs9075-iq-9075-evk\0");

        let mut sampler = Sampler::with_root(fixture.root()).expect("sampler");
        assert_eq!(sampler.device().npu_name, "Hexagon NPU");

        let snapshot = sampler.sample().expect("sample");
        let npu = snapshot.npu().expect("npu");
        assert_eq!(npu.name, "Hexagon NPU");
        assert_eq!(npu.util, None);
        assert!(npu.engines.is_empty());
    }

    #[test]
    fn no_npu_nodes_means_no_npu_panel() {
        let fixture = four_cores("no-npu");
        let mut sampler = Sampler::with_root(fixture.root()).expect("sampler");
        assert!(sampler.sample().expect("sample").npu().is_none());
    }

    #[test]
    fn garbage_files_degrade_to_zeroes() {
        let fixture = Fixture::new("garbage");
        fixture.write("proc/stat", "cpu nonsense\ncpu0 1\n");
        fixture.write("proc/meminfo", "not a meminfo file\n");
        fixture.write("sys/class/thermal/thermal_zone0/temp", "warm\n");
        fixture.write("sys/class/kgsl/kgsl-3d0/gpu_model", "Adreno\n");
        fixture.write("sys/class/kgsl/kgsl-3d0/gpu_busy_percentage", "busy\n");

        let mut sampler = Sampler::with_root(fixture.root()).expect("sampler");
        let snapshot = sampler.sample().expect("sample");
        assert_eq!(snapshot.cpu.util, 0.0);
        assert_eq!(snapshot.mem.total, 0);
        assert_eq!(snapshot.thermal_c, None);
        assert_eq!(snapshot.gpu().map(|gpu| gpu.util), Some(0.0));
    }

    #[test]
    fn percent_accepts_both_kernel_spellings() {
        let fixture = Fixture::new("percent");
        fixture.write("a", "37 %\n");
        fixture.write("b", "37\n");
        fixture.write("c", " 12.5 % ");
        fixture.write("d", "busy\n");

        assert_eq!(percent(&fixture.root().join("a")), Some(37.0));
        assert_eq!(percent(&fixture.root().join("b")), Some(37.0));
        assert_eq!(percent(&fixture.root().join("c")), Some(12.5));
        assert_eq!(percent(&fixture.root().join("d")), None);
    }

    #[test]
    fn cpu_masks_are_expanded() {
        let fixture = Fixture::new("cpu-masks");
        fixture.write("mask", "0-2,5,7-8\n");
        assert_eq!(
            cpu_list(&fixture.root().join("mask")),
            vec![0, 1, 2, 5, 7, 8]
        );
        assert!(cpu_list(&fixture.root().join("missing")).is_empty());
    }
}
