//! Platform backends.
//!
//! The frontend only ever sees `Snapshot` and `DeviceInfo`, so a new OS is
//! a new `Sampler` implementation and nothing else. Linux is deliberately
//! staged as the next target: the same `/proc` + `/sys` data exists, and the
//! model above was shaped so it can be filled without touching the UI.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::Sampler;

#[cfg(not(windows))]
mod linux;
#[cfg(not(windows))]
pub use linux::Sampler;
