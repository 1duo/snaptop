//! Platform backends. A new OS is a new `Sampler`; the frontend only sees
//! `Snapshot` and `DeviceInfo`.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::Sampler;

// The Linux backend is plain `/proc` and `/sys` reads, so it builds — and its
// fixture tests run — on every host; only the export is platform-specific.
#[cfg_attr(windows, allow(dead_code))]
mod linux;
#[cfg(not(windows))]
pub use linux::Sampler;
