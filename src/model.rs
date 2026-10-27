//! The single snapshot that every backend produces and every frontend renders.

/// Which kind of CPU core a logical processor belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreKind {
    Performance,
    Efficiency,
    Unknown,
}

impl CoreKind {
    pub fn label(self) -> &'static str {
        match self {
            CoreKind::Performance => "P",
            CoreKind::Efficiency => "E",
            CoreKind::Unknown => "C",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Core {
    pub id: usize,
    pub kind: CoreKind,
    pub util: f32,
    pub cur_mhz: u32,
    pub max_mhz: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Cpu {
    pub util: f32,
    pub cores: Vec<Core>,
    pub cur_mhz: u32,
    pub max_mhz: u32,
    pub watts: Option<f32>,
}

impl Cpu {
    pub fn count(&self, kind: CoreKind) -> usize {
        self.cores.iter().filter(|c| c.kind == kind).count()
    }

    pub fn util_of(&self, kind: CoreKind) -> Option<f32> {
        let mut sum = 0.0;
        let mut n = 0;
        for core in self.cores.iter().filter(|c| c.kind == kind) {
            sum += core.util;
            n += 1;
        }
        (n > 0).then(|| sum / n as f32)
    }
}

#[derive(Clone, Debug)]
pub struct Engine {
    pub name: String,
    pub util: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Gpu {
    pub name: String,
    pub util: f32,
    pub engines: Vec<Engine>,
    pub cur_mhz: Option<u32>,
    pub max_mhz: Option<u32>,
    pub mem_used: Option<u64>,
    pub mem_total: Option<u64>,
    pub watts: Option<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct Npu {
    pub name: String,
    pub util: f32,
    pub engines: Vec<Engine>,
    pub watts: Option<f32>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Mem {
    pub used: u64,
    pub total: u64,
    pub available: u64,
    pub commit_used: u64,
    pub commit_total: u64,
}

impl Mem {
    pub fn pct(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.used as f32 * 100.0 / self.total as f32
        }
    }

    pub fn commit_pct(&self) -> f32 {
        if self.commit_total == 0 {
            0.0
        } else {
            self.commit_used as f32 * 100.0 / self.commit_total as f32
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Power {
    /// Whole-system draw, measured at the platform rail.
    pub sys_w: Option<f32>,
    /// Charger input, measured at the USB-C rail.
    pub psu_w: Option<f32>,
    pub cpu_w: Option<f32>,
    pub gpu_w: Option<f32>,
    pub npu_w: Option<f32>,
    pub battery_pct: Option<u8>,
    pub ac: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub cpu: Cpu,
    pub gpus: Vec<Gpu>,
    pub npus: Vec<Npu>,
    pub mem: Mem,
    pub power: Power,
    /// Hottest thermal zone, in degrees Celsius.
    pub thermal_c: Option<f32>,
}

impl Snapshot {
    pub fn gpu(&self) -> Option<&Gpu> {
        self.gpus.first()
    }

    pub fn npu(&self) -> Option<&Npu> {
        self.npus.first()
    }
}

/// Static facts about the machine, gathered once at startup.
#[derive(Clone, Debug, Default)]
pub struct DeviceInfo {
    pub cpu_name: String,
    pub os_name: String,
    pub os_build: String,
    pub cpu_count: usize,
    pub p_cores: usize,
    pub e_cores: usize,
    pub gpu_name: String,
    pub npu_name: String,
}

impl DeviceInfo {
    pub fn core_summary(&self) -> String {
        if self.p_cores == 0 || self.e_cores == 0 {
            return format!("{} cores", self.cpu_count);
        }
        format!(
            "{} cores ({}P + {}E)",
            self.cpu_count, self.p_cores, self.e_cores
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_class_summary_has_no_split() {
        let x2 = DeviceInfo {
            cpu_count: 18,
            p_cores: 18,
            e_cores: 0,
            ..Default::default()
        };
        assert_eq!(x2.core_summary(), "18 cores");

        let x1 = DeviceInfo {
            cpu_count: 12,
            p_cores: 8,
            e_cores: 4,
            ..Default::default()
        };
        assert_eq!(x1.core_summary(), "12 cores (8P + 4E)");
    }
}
