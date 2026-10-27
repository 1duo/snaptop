//! Linux backend — not implemented yet.
//!
//! `/proc/stat`, `amdgpu`/`i915`, `accel/*` and `hwmon` map onto the same
//! snapshot fields.

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
