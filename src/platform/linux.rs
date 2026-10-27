//! Linux backend — the next milestone, not a shipped path yet.
//!
//! The data exists and is simpler than on Windows: `/proc/stat` for CPU,
//! `/sys/class/drm/card*/device/gpu_busy_percent` for the GPU and
//! `/sys/class/accel/accel*/device/npu_busy_time_us` for the NPU, plus
//! `/sys/class/hwmon` for power and thermal. The snapshot model already has
//! slots for every one of those, so this file only needs a `Sampler`.

use crate::model::{DeviceInfo, Snapshot};

pub struct Sampler;

impl Sampler {
    pub fn new() -> Result<Self, String> {
        Err(
            "the Linux backend is not implemented yet; Windows on Snapdragon is the current target"
                .to_string(),
        )
    }

    pub fn device(&self) -> DeviceInfo {
        DeviceInfo::default()
    }

    pub fn sample(&mut self) -> Result<Snapshot, String> {
        Err("the Linux backend is not implemented yet".to_string())
    }
}
