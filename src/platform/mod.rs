//! Platform backends. A new OS is a new `Sampler`; the frontend only sees
//! `Snapshot` and `DeviceInfo`.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::Sampler;

#[cfg(not(windows))]
mod linux;
#[cfg(not(windows))]
pub use linux::Sampler;
