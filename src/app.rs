//! UI state: the last snapshot plus presentation history.

use std::time::{Duration, Instant};

use crate::history::History;
use crate::model::{DeviceInfo, Snapshot};
use crate::theme;

pub const HISTORY: usize = 240;

pub struct App {
    pub device: DeviceInfo,
    pub snapshot: Snapshot,
    pub cpu: History,
    pub gpu: History,
    pub npu: History,
    pub mem: History,
    pub power: History,
    pub last_power: Option<f32>,
    pub paused: bool,
    pub interval_ms: u64,
    pub theme: usize,
    pub help: bool,
    pub samples: u64,
    pub last: Option<Instant>,
}

impl App {
    pub fn new(device: DeviceInfo, interval_ms: u64, theme: Option<usize>) -> Self {
        Self {
            device,
            snapshot: Snapshot::default(),
            cpu: History::new(HISTORY),
            gpu: History::new(HISTORY),
            npu: History::new(HISTORY),
            mem: History::new(HISTORY),
            power: History::new(HISTORY),
            last_power: None,
            paused: false,
            interval_ms,
            theme: theme.unwrap_or_else(theme::auto_index),
            help: false,
            samples: 0,
            last: None,
        }
    }

    pub fn update(&mut self, snapshot: Snapshot) {
        self.cpu.push(snapshot.cpu.util);
        self.gpu.push(snapshot.gpu().map(|g| g.util).unwrap_or(0.0));
        self.npu.push(snapshot.npu().map(|n| n.util).unwrap_or(0.0));
        self.mem.push(snapshot.mem.pct());
        if let Some(watts) = snapshot.power.sys_w.or(self.last_power) {
            self.power.push(watts);
            self.last_power = Some(watts);
        }
        self.snapshot = snapshot;
        self.samples += 1;
        self.last = Some(Instant::now());
    }

    pub fn reset(&mut self) {
        self.cpu.clear();
        self.gpu.clear();
        self.npu.clear();
        self.mem.clear();
        self.power.clear();
    }

    pub fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }

    pub fn faster(&mut self) {
        self.interval_ms = self.interval_ms.saturating_sub(250).max(250);
    }

    pub fn slower(&mut self) {
        self.interval_ms = (self.interval_ms + 250).min(10_000);
    }

    pub fn stale(&self) -> bool {
        self.last
            .map(|t| {
                t.elapsed()
                    > self
                        .interval()
                        .saturating_mul(6)
                        .max(Duration::from_secs(2))
            })
            .unwrap_or(false)
    }
}
