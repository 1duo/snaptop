//! Windows backend: PDH counters for CPU/GPU/NPU utilisation, DXGI for adapter
//! identity and memory, D3DKMT for GPU clocks, the registry for names, and the
//! Energy Meter rails for power.
//!
//! The NPU is the MCDM compute accelerator: a LUID in `\GPU Engine` that belongs
//! to no DXGI adapter. That is also how Task Manager finds it.

use std::collections::BTreeMap;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::time::{Duration, Instant};

use winreg::RegKey;
use winreg::enums::HKEY_LOCAL_MACHINE;

use crate::fmt;
use crate::model::{Core, CoreKind, Cpu, DeviceInfo, Engine, Gpu, Mem, Npu, Power, Snapshot};

pub type Result<T> = std::result::Result<T, String>;

const PDH_FMT_DOUBLE: u32 = 0x0000_0200;
const PDH_MORE_DATA: u32 = 0x8000_07D2;
const PDH_CSTATUS_VALID_DATA: u32 = 0x0000_0000;
const PDH_CSTATUS_NEW_DATA: u32 = 0x0000_0001;

#[repr(C)]
#[derive(Clone, Copy)]
struct FmtValue {
    status: u32,
    value: f64,
}

#[repr(C)]
struct FmtItem {
    name: *mut u16,
    value: FmtValue,
}

#[link(name = "pdh")]
unsafe extern "system" {
    fn PdhOpenQueryW(src: *const u16, user: usize, query: *mut isize) -> u32;
    fn PdhAddEnglishCounterW(
        query: isize,
        path: *const u16,
        user: usize,
        counter: *mut isize,
    ) -> u32;
    fn PdhCollectQueryData(query: isize) -> u32;
    fn PdhGetFormattedCounterValue(
        counter: isize,
        format: u32,
        kind: *mut u32,
        value: *mut FmtValue,
    ) -> u32;
    fn PdhGetFormattedCounterArrayW(
        counter: isize,
        format: u32,
        size: *mut u32,
        count: *mut u32,
        buffer: *mut FmtItem,
    ) -> u32;
    fn PdhCloseQuery(query: isize) -> u32;
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Read a NUL-terminated UTF-16 string returned by a Win32 API.
///
/// # Safety
///
/// `p` must be null or point to a NUL-terminated UTF-16 string that stays
/// valid for the duration of the call. The 4096-unit cap is a defensive stop,
/// not a guarantee.
unsafe fn pcwstr_to_string(p: *const u16) -> String {
    if p.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    // SAFETY: the caller guarantees a NUL-terminated string and `len` is capped.
    while unsafe { *p.add(len) } != 0 {
        len += 1;
        if len > 4096 {
            break;
        }
    }
    // SAFETY: `len` units were just proven readable and in bounds above.
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(p, len) })
}

struct Pdh {
    query: isize,
}

impl Pdh {
    fn new() -> Option<Self> {
        let mut query = 0isize;
        // SAFETY: `query` is a local out-param; on success the handle is owned
        // by the returned `Pdh` and closed in `Drop`.
        let rc = unsafe { PdhOpenQueryW(null(), 0, &mut query) };
        (rc == 0).then_some(Self { query })
    }

    fn add(&mut self, path: &str) -> Option<Counter> {
        let path = wide(path);
        let mut counter = 0isize;
        // SAFETY: `self.query` is the live handle from `PdhOpenQueryW`; `path`
        // is NUL-terminated and outlives the call.
        let rc = unsafe { PdhAddEnglishCounterW(self.query, path.as_ptr(), 0, &mut counter) };
        (rc == 0).then_some(Counter { handle: counter })
    }

    fn collect(&self) -> bool {
        // SAFETY: the query handle is live and single-threaded.
        unsafe { PdhCollectQueryData(self.query) == 0 }
    }
}

impl Drop for Pdh {
    fn drop(&mut self) {
        // SAFETY: `self.query` was returned by `PdhOpenQueryW` and is closed
        // exactly once here; `Pdh` is neither `Clone` nor `Copy`.
        unsafe {
            PdhCloseQuery(self.query);
        }
    }
}

#[derive(Clone, Copy)]
struct Counter {
    handle: isize,
}

impl Counter {
    fn value(&self) -> Option<f64> {
        let mut value = FmtValue {
            status: 0,
            value: 0.0,
        };
        // SAFETY: the counter belongs to a live query and `value` is a valid,
        // correctly sized out-param for PDH_FMT_COUNTERVALUE.
        let rc = unsafe {
            PdhGetFormattedCounterValue(self.handle, PDH_FMT_DOUBLE, null_mut(), &mut value)
        };
        (rc == 0).then_some(value.value)
    }

    /// Every instance of a wildcard counter.
    fn array(&self) -> Vec<(String, f64)> {
        let mut size = 0u32;
        let mut count = 0u32;
        // SAFETY: a null buffer is the documented way to ask PDH for the
        // required byte size; no memory is written.
        let mut rc = unsafe {
            PdhGetFormattedCounterArrayW(
                self.handle,
                PDH_FMT_DOUBLE,
                &mut size,
                &mut count,
                null_mut(),
            )
        };
        if rc == 0 && count == 0 {
            return Vec::new();
        }
        if rc != PDH_MORE_DATA || size == 0 {
            return Vec::new();
        }

        // Vec<u64> keeps the buffer 8-byte aligned for FmtItem.
        let mut buffer = vec![0u64; (size as usize).div_ceil(8)];
        // SAFETY: `buffer` is at least `size` bytes (rounded up to u64) and
        // 8-byte aligned; PDH writes at most `size` bytes into it.
        rc = unsafe {
            PdhGetFormattedCounterArrayW(
                self.handle,
                PDH_FMT_DOUBLE,
                &mut size,
                &mut count,
                buffer.as_mut_ptr().cast::<FmtItem>(),
            )
        };
        if rc != 0 {
            return Vec::new();
        }

        let base = buffer.as_ptr().cast::<FmtItem>();
        let max_items = (size as usize) / std::mem::size_of::<FmtItem>();
        let mut out = Vec::with_capacity(count as usize);
        for i in 0..(count as usize).min(max_items) {
            // SAFETY: `i` is bounded by the item count and by the number of
            // items that fit in the buffer PDH just filled.
            let item = unsafe { &*base.add(i) };
            if item.value.status != PDH_CSTATUS_VALID_DATA
                && item.value.status != PDH_CSTATUS_NEW_DATA
            {
                continue;
            }
            out.push((
                // SAFETY: `item.name` is a NUL-terminated string inside the
                // buffer PDH just filled.
                unsafe { pcwstr_to_string(item.name) },
                item.value.value.max(0.0),
            ));
        }
        out
    }
}

use windows::Win32::Foundation::LUID;
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};

#[derive(Clone, Debug)]
struct Adapter {
    name: String,
    key: String,
    dedicated: u64,
    shared: u64,
}

fn luid_key(luid: LUID) -> String {
    format!("luid_0x{:08x}_0x{:08x}", luid.HighPart as u32, luid.LowPart)
}

fn dxgi_adapters() -> (Vec<Adapter>, Vec<String>) {
    let mut out = Vec::new();
    let mut software = Vec::new();
    // SAFETY: CreateDXGIFactory1 returns an owned COM interface; every
    // reference is released by Drop.
    unsafe {
        let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
            Ok(f) => f,
            Err(_) => return (out, software),
        };
        let mut index = 0;
        while let Ok(adapter) = factory.EnumAdapters1(index) {
            if let Ok(desc) = adapter.GetDesc1() {
                let len = desc
                    .Description
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(desc.Description.len());
                // 0x2 = DXGI_ADAPTER_FLAG_SOFTWARE. WARP also emits engine
                // counters, so remember it: it must never look like an NPU.
                if desc.Flags & 0x2 != 0 {
                    software.push(luid_key(desc.AdapterLuid));
                } else {
                    out.push(Adapter {
                        name: String::from_utf16_lossy(&desc.Description[..len]),
                        key: luid_key(desc.AdapterLuid),
                        dedicated: desc.DedicatedVideoMemory as u64,
                        shared: desc.SharedSystemMemory as u64,
                    });
                }
            }
            index += 1;
        }
    }
    (out, software)
}

#[repr(C)]
struct KmtOpen {
    luid: LUID,
    handle: u32,
}

#[repr(C)]
struct KmtClose {
    handle: u32,
}

#[repr(C)]
struct KmtQuery {
    handle: u32,
    kind: i32,
    data: *mut c_void,
    size: u32,
}

#[repr(C)]
#[derive(Default)]
struct NodePerfData {
    node_ordinal: u32,
    physical_adapter_index: u32,
    frequency: u64,
    max_frequency: u64,
    max_frequency_oc: u64,
    voltage: u32,
    voltage_max: u32,
    voltage_max_oc: u32,
    max_transition_latency: u64,
}

const KMTQAITYPE_NODEPERFDATA: i32 = 61;

type KmtOpenFn = unsafe extern "system" fn(*mut KmtOpen) -> i32;
type KmtQueryFn = unsafe extern "system" fn(*const KmtQuery) -> i32;
type KmtCloseFn = unsafe extern "system" fn(*const KmtClose) -> i32;

struct Kmt {
    open: KmtOpenFn,
    query: KmtQueryFn,
    close: KmtCloseFn,
}

unsafe extern "system" {
    fn LoadLibraryW(name: *const u16) -> *mut c_void;
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
}

impl Kmt {
    fn load() -> Option<Self> {
        // SAFETY: gdi32 is a system module; each address is transmuted to the
        // exact D3DKMT signature declared above.
        unsafe {
            let mut module = GetModuleHandleW(wide("gdi32.dll").as_ptr());
            if module.is_null() {
                module = LoadLibraryW(wide("gdi32.dll").as_ptr());
            }
            if module.is_null() {
                return None;
            }
            let open = GetProcAddress(module, c"D3DKMTOpenAdapterFromLuid".as_ptr().cast());
            let query = GetProcAddress(module, c"D3DKMTQueryAdapterInfo".as_ptr().cast());
            let close = GetProcAddress(module, c"D3DKMTCloseAdapter".as_ptr().cast());
            if open.is_null() || query.is_null() || close.is_null() {
                return None;
            }
            Some(Self {
                open: std::mem::transmute::<*mut c_void, KmtOpenFn>(open),
                query: std::mem::transmute::<*mut c_void, KmtQueryFn>(query),
                close: std::mem::transmute::<*mut c_void, KmtCloseFn>(close),
            })
        }
    }

    /// Engine clock in MHz, when the driver implements the optional DDI.
    fn clock(&self, luid: LUID) -> Option<(u32, u32)> {
        // SAFETY: the LUID comes from DXGI; each buffer matches NodePerfData's
        // layout, and the adapter is closed on every path after opening.
        unsafe {
            let mut open = KmtOpen { luid, handle: 0 };
            if (self.open)(&mut open) != 0 {
                return None;
            }
            let mut current = 0u64;
            let mut max = 0u64;
            for node in 0..16u32 {
                let mut data = NodePerfData {
                    node_ordinal: node,
                    ..Default::default()
                };
                let query = KmtQuery {
                    handle: open.handle,
                    kind: KMTQAITYPE_NODEPERFDATA,
                    data: (&raw mut data).cast(),
                    size: std::mem::size_of::<NodePerfData>() as u32,
                };
                if (self.query)(&query) == 0 {
                    current = current.max(data.frequency);
                    max = max.max(data.max_frequency);
                }
            }
            let close = KmtClose {
                handle: open.handle,
            };
            (self.close)(&close);
            (current > 0).then_some(((current / 1_000_000) as u32, (max / 1_000_000) as u32))
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GroupAffinity {
    mask: usize,
    group: u16,
    reserved: [u16; 3],
}

/// Fixed head of `SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX`; the affinity tail
/// is variable-length, so it is read with raw pointers, not `[T; 1]`.
#[repr(C)]
#[derive(Clone, Copy)]
struct LpiExHead {
    relationship: u32,
    size: u32,
}

/// Fixed head of `PROCESSOR_RELATIONSHIP`, up to the affinity array.
#[repr(C)]
#[derive(Clone, Copy)]
struct ProcessorHead {
    flags: u8,
    efficiency_class: u8,
    reserved: [u8; 20],
    group_count: u16,
}

#[repr(C)]
struct ProcessorPowerInformation {
    number: u32,
    max_mhz: u32,
    current_mhz: u32,
    mhz_limit: u32,
    max_idle_state: u32,
    current_idle_state: u32,
}

#[repr(C)]
#[derive(Default)]
struct MemoryStatusEx {
    length: u32,
    memory_load: u32,
    total_phys: u64,
    avail_phys: u64,
    total_page_file: u64,
    avail_page_file: u64,
    total_virtual: u64,
    avail_virtual: u64,
    avail_extended_virtual: u64,
}

#[repr(C)]
#[derive(Default)]
struct SystemPowerStatus {
    ac_line_status: u8,
    battery_flag: u8,
    battery_life_pct: u8,
    system_status_flag: u8,
    battery_life_time: u32,
    battery_full_life_time: u32,
}

const ALL_PROCESSOR_GROUPS: u16 = 0xffff;
const RELATION_PROCESSOR_CORE: u32 = 0;
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const PROCESSOR_INFORMATION: i32 = 11;

unsafe extern "system" {
    fn GetLogicalProcessorInformationEx(
        relationship: u32,
        buffer: *mut c_void,
        len: *mut u32,
    ) -> i32;
    fn GetLastError() -> u32;
    fn GetActiveProcessorCount(group: u16) -> u32;
    fn GlobalMemoryStatusEx(status: *mut MemoryStatusEx) -> i32;
    fn GetSystemPowerStatus(status: *mut SystemPowerStatus) -> i32;
}

#[link(name = "powrprof")]
unsafe extern "system" {
    fn CallNtPowerInformation(
        class: i32,
        input: *mut c_void,
        input_len: u32,
        output: *mut c_void,
        output_len: u32,
    ) -> i32;
}

/// Logical processor -> P/E class, from the firmware's EfficiencyClass.
fn core_kinds() -> Vec<CoreKind> {
    // SAFETY: GetLogicalProcessorInformationEx is called twice as documented
    // (size probe, then fill); every read is bounded by `len` and entry `size`.
    unsafe {
        let mut len = 0u32;
        if GetLogicalProcessorInformationEx(RELATION_PROCESSOR_CORE, null_mut(), &mut len) != 0 {
            return Vec::new();
        }
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER || len == 0 {
            return Vec::new();
        }
        let mut buffer = vec![0u64; (len as usize).div_ceil(8)];
        if GetLogicalProcessorInformationEx(
            RELATION_PROCESSOR_CORE,
            buffer.as_mut_ptr().cast(),
            &mut len,
        ) == 0
        {
            return Vec::new();
        }

        let mut classes: BTreeMap<usize, u8> = BTreeMap::new();
        let mut order: Vec<(usize, u8)> = Vec::new();
        let base = buffer.as_ptr().cast::<u8>();
        let mut offset = 0usize;
        let head_len = std::mem::size_of::<LpiExHead>();
        let processor_len = std::mem::size_of::<ProcessorHead>();
        let mask_len = std::mem::size_of::<GroupAffinity>();

        while offset + head_len <= len as usize {
            // SAFETY: at least `head_len` bytes remain; LpiExHead is Copy and
            // contains no pointers.
            let head = *base.add(offset).cast::<LpiExHead>();
            let size = head.size as usize;
            if size < head_len || offset + size > len as usize {
                break;
            }
            if head.relationship == RELATION_PROCESSOR_CORE && size >= head_len + processor_len {
                let processor_at = offset + head_len;
                // SAFETY: the entry is at least `head_len + processor_len`
                // bytes, so this 24-byte read is inside it. ProcessorHead is
                // Copy.
                let processor = *base.add(processor_at).cast::<ProcessorHead>();
                let class = processor.efficiency_class;
                let group_count = (processor.group_count as usize).max(1);
                let mask_at = processor_at + processor_len;
                let max_groups = (offset + size - mask_at) / mask_len;
                let masks = base.add(mask_at).cast::<GroupAffinity>();
                for g in 0..group_count.min(max_groups) {
                    // SAFETY: `g` fits in this entry; `read_unaligned` does not
                    // require the record to be 8-byte padded.
                    let affinity = masks.add(g).read_unaligned();
                    for bit in 0..64usize {
                        if affinity.mask & (1usize << bit) != 0 {
                            let index = affinity.group as usize * 64 + bit;
                            order.push((index, class));
                            classes.insert(index, class);
                        }
                    }
                }
            }
            offset += size;
        }

        let mut distinct: Vec<u8> = classes.values().copied().collect();
        distinct.sort_unstable();
        distinct.dedup();
        let Some(performance_class) = distinct.last().copied().filter(|_| distinct.len() >= 2)
        else {
            return Vec::new();
        };

        let max_index = order.iter().map(|(i, _)| *i).max().unwrap_or(0);
        let mut kinds = vec![CoreKind::Unknown; max_index + 1];
        for (index, class) in order {
            kinds[index] = if class == performance_class {
                CoreKind::Performance
            } else {
                CoreKind::Efficiency
            };
        }
        kinds
    }
}

/// X2 Elite has Prime + Performance cores and no efficiency tier. Windows
/// still ranks the two by EfficiencyClass, so fold the lower tier back into
/// Performance instead of mislabelling it "E".
fn normalize_kinds(cpu_name: &str, kinds: &mut [CoreKind]) {
    if cpu_name.to_ascii_uppercase().contains("X2") {
        for kind in kinds {
            if *kind == CoreKind::Efficiency {
                *kind = CoreKind::Performance;
            }
        }
    }
}

/// Map `\Energy Meter` instances onto rails. Names and counts differ across
/// X Elite / X2 Elite models, so match by prefix and sum every CPU cluster.
fn parse_power(instances: impl IntoIterator<Item = (String, f64)>) -> Power {
    let mut power = Power::default();
    let mut cpu_w = 0.0f32;
    let mut cpu_seen = false;
    let mut sys = None;
    let mut total = None;
    let mut npu = None;

    for (instance, milliwatts) in instances {
        let watts = milliwatts as f32 / 1000.0;
        match instance.to_ascii_lowercase().as_str() {
            "_total" => total = Some(watts),
            name if name.starts_with("sys") => sys = Some(watts),
            name if name.starts_with("psu") => power.psu_w = Some(watts),
            name if name.starts_with("gpu") => power.gpu_w = Some(watts),
            name if name.starts_with("npu") || name.starts_with("nsp") => npu = Some(watts),
            name if name.starts_with("cpu_cluster") => {
                cpu_w += watts;
                cpu_seen = true;
            }
            _ => {}
        }
    }

    // SYS is the platform rail; `_Total` then charger input are fallbacks.
    power.sys_w = sys.or(total).or(power.psu_w);
    if cpu_seen {
        power.cpu_w = Some(cpu_w);
    }
    power.npu_w = npu;
    power
}

fn cpu_clocks(count: usize) -> Vec<(u32, u32)> {
    let mut out = vec![(0, 0); count];
    if count == 0 {
        return out;
    }
    let mut info: Vec<ProcessorPowerInformation> = (0..count)
        .map(|_| ProcessorPowerInformation {
            number: 0,
            max_mhz: 0,
            current_mhz: 0,
            mhz_limit: 0,
            max_idle_state: 0,
            current_idle_state: 0,
        })
        .collect();
    let bytes = (info.len() * std::mem::size_of::<ProcessorPowerInformation>()) as u32;
    // SAFETY: `info` holds `count` initialized PROCESSOR_POWER_INFORMATION
    // records and `bytes` describes exactly that buffer.
    let rc = unsafe {
        CallNtPowerInformation(
            PROCESSOR_INFORMATION,
            null_mut(),
            0,
            info.as_mut_ptr().cast(),
            bytes,
        )
    };
    if rc != 0 {
        return out;
    }
    for item in &info {
        let index = item.number as usize;
        if index < out.len() {
            out[index] = (item.current_mhz, item.max_mhz);
        }
    }
    out
}

fn reg_string(path: &str, name: &str) -> Option<String> {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(path)
        .ok()?
        .get_value::<String, _>(name)
        .ok()
}

fn cpu_name() -> String {
    let raw = reg_string(
        r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
        "ProcessorNameString",
    )
    .unwrap_or_default();
    let pretty = fmt::pretty_cpu_name(&raw);
    if pretty.is_empty() {
        "CPU".to_string()
    } else {
        pretty
    }
}

fn os_identity() -> (String, String) {
    const PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";
    let mut name = reg_string(PATH, "ProductName").unwrap_or_else(|| "Windows".into());
    let build = reg_string(PATH, "CurrentBuildNumber").unwrap_or_default();
    if build.parse::<u32>().unwrap_or(0) >= 22000 {
        name = name.replace("Windows 10", "Windows 11");
    }
    if let Some(edition) = reg_string(PATH, "DisplayVersion")
        && !edition.is_empty()
    {
        name = format!("{name} {edition}");
    }
    (name, build)
}

fn npu_name() -> String {
    const BASE: &str =
        r"SYSTEM\CurrentControlSet\Control\Class\{f01a9d53-3ff6-48d2-9f97-c8a7004be10c}";
    for i in 0..10 {
        if let Some(desc) = reg_string(&format!(r"{BASE}\{i:04}"), "DriverDesc") {
            return fmt::pretty_npu_name(&desc);
        }
    }
    "NPU".to_string()
}

/// `pid_1234_luid_0x00000000_0x00011643_phys_0_eng_0_engtype_3D`
fn parse_engine_instance(instance: &str) -> Option<(String, String)> {
    let lower = instance.to_ascii_lowercase();
    let luid_at = lower.find("luid_")?;
    let phys_at = lower[luid_at..].find("_phys_")? + luid_at;
    let luid = lower[luid_at..phys_at].to_string();
    let engine = match lower.find("engtype_") {
        Some(at) => instance[at + 8..].to_string(),
        None => "Unknown".to_string(),
    };
    Some((luid, engine))
}

/// `luid_0x00000000_0x00011643_phys_0`
fn parse_adapter_instance(instance: &str) -> Option<String> {
    let lower = instance.to_ascii_lowercase();
    let luid_at = lower.find("luid_")?;
    let phys_at = lower[luid_at..].find("_phys_").map(|i| i + luid_at);
    Some(match phys_at {
        Some(at) => lower[luid_at..at].to_string(),
        None => lower[luid_at..].to_string(),
    })
}

/// `0,7` -> (group 0, processor 7)
fn parse_processor_instance(instance: &str) -> Option<(usize, usize)> {
    let mut parts = instance.split(',');
    let group = parts.next()?.trim().parse().ok()?;
    let index = parts.next()?.trim().parse().ok()?;
    Some((group, index))
}

fn engine_rank(name: &str) -> u8 {
    match name {
        "3D" => 0,
        "Compute" => 1,
        "VideoDecode" => 2,
        "VideoEncode" => 3,
        "VideoProcessing" => 4,
        "HW Rotator" => 5,
        "Decryption" => 6,
        _ => 9,
    }
}

pub struct Sampler {
    device: DeviceInfo,
    pdh: Pdh,
    cpu_total: Option<Counter>,
    cpu_cores: Option<Counter>,
    gpu_engine: Option<Counter>,
    gpu_dedicated: Option<Counter>,
    gpu_shared: Option<Counter>,
    energy: Option<Counter>,
    thermal: Option<Counter>,
    kinds: Vec<CoreKind>,
    adapters: Vec<Adapter>,
    software: Vec<String>,
    npu: String,
    kmt: Option<Kmt>,
    cpus: usize,
    energy_query: Option<Pdh>,
    power: Power,
    power_at: Option<Instant>,
}

impl Sampler {
    pub fn new() -> Result<Self> {
        let mut pdh = Pdh::new().ok_or_else(|| "PDH is unavailable on this system".to_string())?;

        let cpu_total = pdh
            .add(r"\Processor Information(_Total)\% Processor Utility")
            .or_else(|| pdh.add(r"\Processor(_Total)\% Processor Time"));
        let cpu_cores = pdh
            .add(r"\Processor Information(*)\% Processor Utility")
            .or_else(|| pdh.add(r"\Processor(*)\% Processor Time"));
        let gpu_engine = pdh.add(r"\GPU Engine(*)\Utilization Percentage");
        let gpu_dedicated = pdh.add(r"\GPU Adapter Memory(*)\Dedicated Usage");
        let gpu_shared = pdh.add(r"\GPU Adapter Memory(*)\Shared Usage");
        let mut energy_query = Pdh::new();
        let energy = energy_query
            .as_mut()
            .and_then(|query| query.add(r"\Energy Meter(*)\Power"));
        if let Some(query) = &energy_query {
            query.collect();
        }
        let thermal = pdh.add(r"\Thermal Zone Information(*)\Temperature");

        // SAFETY: the call takes no pointers and has no preconditions.
        let cpus = unsafe { GetActiveProcessorCount(ALL_PROCESSOR_GROUPS) as usize }.max(1);
        let cpu = cpu_name();
        let mut kinds = {
            let k = core_kinds();
            (0..cpus)
                .map(|i| k.get(i).copied().unwrap_or(CoreKind::Unknown))
                .collect::<Vec<_>>()
        };
        normalize_kinds(&cpu, &mut kinds);

        let (adapters, software) = dxgi_adapters();
        let npu = npu_name();
        let (os_name, os_build) = os_identity();

        let p_cores = kinds
            .iter()
            .filter(|k| **k == CoreKind::Performance)
            .count();
        let e_cores = kinds.iter().filter(|k| **k == CoreKind::Efficiency).count();

        let device = DeviceInfo {
            cpu_name: cpu,
            os_name,
            os_build,
            cpu_count: cpus,
            p_cores,
            e_cores,
            gpu_name: adapters
                .first()
                .map(|a| fmt::pretty_gpu_name(&a.name))
                .unwrap_or_else(|| "GPU".to_string()),
            npu_name: npu.clone(),
        };

        // Prime the rate counters; PDH needs two collections.
        pdh.collect();

        Ok(Self {
            device,
            pdh,
            cpu_total,
            cpu_cores,
            gpu_engine,
            gpu_dedicated,
            gpu_shared,
            energy_query,
            energy,
            thermal,
            kinds,
            adapters,
            software,
            npu,
            kmt: Kmt::load(),
            cpus,
            power: Power::default(),
            power_at: Some(Instant::now()),
        })
    }

    pub fn device(&self) -> DeviceInfo {
        self.device.clone()
    }

    pub fn sample(&mut self) -> Result<Snapshot> {
        self.pdh.collect();
        let mut snapshot = Snapshot::default();

        // ---- CPU ----------------------------------------------------------
        let mut cpu = Cpu {
            // `% Processor Utility` can exceed 100% when a core runs above its
            // nominal frequency; every consumer here treats 100 as full scale.
            util: self
                .cpu_total
                .and_then(|c| c.value())
                .unwrap_or(0.0)
                .clamp(0.0, 100.0) as f32,
            ..Default::default()
        };

        let mut util = vec![0.0f32; self.cpus];
        if let Some(counter) = self.cpu_cores {
            for (instance, value) in counter.array() {
                let lower = instance.to_ascii_lowercase();
                if lower.contains("_total") {
                    continue;
                }
                if let Some((group, index)) = parse_processor_instance(&instance) {
                    let i = group * 64 + index;
                    if i < util.len() {
                        util[i] = value as f32;
                    }
                }
            }
        }

        let clocks = cpu_clocks(self.cpus);
        for (i, value) in util.iter().enumerate().take(self.cpus) {
            let (cur, max) = clocks.get(i).copied().unwrap_or((0, 0));
            cpu.cores.push(Core {
                id: i,
                kind: self.kinds.get(i).copied().unwrap_or(CoreKind::Unknown),
                util: value.clamp(0.0, 100.0),
                cur_mhz: cur,
                max_mhz: max,
            });
        }
        cpu.cur_mhz = cpu.cores.iter().map(|c| c.cur_mhz).max().unwrap_or(0);
        cpu.max_mhz = cpu.cores.iter().map(|c| c.max_mhz).max().unwrap_or(0);
        snapshot.cpu = cpu;

        // ---- GPU + NPU engines -------------------------------------------
        // luid -> engtype -> summed utilisation across every process
        let mut by_luid: BTreeMap<String, BTreeMap<String, f32>> = BTreeMap::new();
        if let Some(counter) = self.gpu_engine {
            for (instance, value) in counter.array() {
                if let Some((luid, engine)) = parse_engine_instance(&instance) {
                    *by_luid
                        .entry(luid)
                        .or_default()
                        .entry(engine)
                        .or_insert(0.0) += value as f32;
                }
            }
        }

        // Discrete cards count dedicated allocations against VRAM; integrated
        // GPUs share system memory, so both pools are summed.
        let mut dedicated_used: BTreeMap<String, u64> = BTreeMap::new();
        let mut shared_used: BTreeMap<String, u64> = BTreeMap::new();
        if let Some(counter) = self.gpu_dedicated {
            for (instance, value) in counter.array() {
                if let Some(luid) = parse_adapter_instance(&instance) {
                    let entry = dedicated_used.entry(luid).or_insert(0);
                    *entry = entry.saturating_add(value as u64);
                }
            }
        }
        if let Some(counter) = self.gpu_shared {
            for (instance, value) in counter.array() {
                if let Some(luid) = parse_adapter_instance(&instance) {
                    let entry = shared_used.entry(luid).or_insert(0);
                    *entry = entry.saturating_add(value as u64);
                }
            }
        }

        let engines_of = |luid: &str| -> Vec<Engine> {
            let mut engines: Vec<Engine> = by_luid
                .get(luid)
                .map(|m| {
                    m.iter()
                        .map(|(name, value)| Engine {
                            name: name.clone(),
                            util: value.clamp(0.0, 100.0),
                        })
                        .collect()
                })
                .unwrap_or_default();
            engines.sort_by(|a, b| {
                engine_rank(&a.name)
                    .cmp(&engine_rank(&b.name))
                    .then(a.name.cmp(&b.name))
            });
            engines
        };

        let known: Vec<String> = self.adapters.iter().map(|a| a.key.clone()).collect();
        for adapter in &self.adapters {
            let engines = engines_of(&adapter.key);
            let clock = self
                .kmt
                .as_ref()
                .and_then(|kmt| adapter_key_luid(&adapter.key).and_then(|l| kmt.clock(l)));
            let dedicated = dedicated_used.get(&adapter.key).copied().unwrap_or(0);
            let shared = shared_used.get(&adapter.key).copied().unwrap_or(0);
            let discrete = adapter.dedicated > 1024 * 1024 * 1024;
            let total = if discrete {
                adapter.dedicated
            } else {
                adapter.dedicated.saturating_add(adapter.shared)
            };
            let used = if discrete {
                dedicated
            } else {
                dedicated.saturating_add(shared)
            };
            snapshot.gpus.push(Gpu {
                name: fmt::pretty_gpu_name(&adapter.name),
                util: engines.iter().map(|e| e.util).fold(0.0, f32::max),
                engines,
                cur_mhz: clock.map(|c| c.0),
                max_mhz: clock.map(|c| c.1),
                mem_used: (used > 0 && total > 0).then_some(used.min(total)),
                mem_total: (total > 0).then_some(total),
                watts: None, // filled from the energy meter below
            });
        }

        // A LUID with engine or memory counters that no DXGI adapter claims is
        // an MCDM accelerator: the NPU. Memory instances exist even when idle.
        let mut luids: std::collections::BTreeSet<String> = by_luid.keys().cloned().collect();
        luids.extend(dedicated_used.keys().cloned());
        luids.extend(shared_used.keys().cloned());

        for luid in &luids {
            if known.iter().any(|k| k == luid) || self.software.iter().any(|k| k == luid) {
                continue;
            }
            let engines = engines_of(luid);
            snapshot.npus.push(Npu {
                name: self.npu.clone(),
                util: engines.iter().map(|e| e.util).fold(0.0, f32::max),
                engines,
                watts: None,
            });
        }

        // ---- memory -------------------------------------------------------
        let mut mem = MemoryStatusEx {
            length: std::mem::size_of::<MemoryStatusEx>() as u32,
            ..Default::default()
        };
        // SAFETY: `length` is set to the size of the struct, as the API
        // requires, and `mem` is a valid out-param.
        if unsafe { GlobalMemoryStatusEx(&mut mem) } != 0 {
            snapshot.mem = Mem {
                used: mem.total_phys.saturating_sub(mem.avail_phys),
                total: mem.total_phys,
                available: mem.avail_phys,
                commit_used: mem.total_page_file.saturating_sub(mem.avail_page_file),
                commit_total: mem.total_page_file,
            };
        }

        // ---- power --------------------------------------------------------
        // The energy meter updates at ~1 Hz; sampling faster reads zeros, so
        // the rails are cached and refreshed at most once a second.
        let now = Instant::now();
        let refresh = self
            .power_at
            .is_none_or(|t| now.duration_since(t) >= Duration::from_millis(1000));
        if refresh {
            self.power_at = Some(now);
            if let (Some(query), Some(counter)) = (&self.energy_query, self.energy) {
                query.collect();
                let mut power = parse_power(counter.array());
                power.battery_pct = self.power.battery_pct;
                power.ac = self.power.ac;
                self.power = power;
            }
        }

        let mut status = SystemPowerStatus::default();
        // SAFETY: `status` is a valid zeroed out-param.
        if unsafe { GetSystemPowerStatus(&mut status) } != 0 {
            if status.battery_life_pct <= 100 {
                self.power.battery_pct = Some(status.battery_life_pct);
            }
            self.power.ac = Some(status.ac_line_status == 1);
        }

        snapshot.power = self.power;
        if let Some(gpu) = snapshot.gpus.first_mut() {
            gpu.watts = self.power.gpu_w;
        }
        if let Some(npu) = snapshot.npus.first_mut() {
            npu.watts = self.power.npu_w;
        }
        snapshot.cpu.watts = self.power.cpu_w;

        // ---- thermal ------------------------------------------------------
        if let Some(counter) = self.thermal {
            let hottest = counter
                .array()
                .into_iter()
                .map(|(_, kelvin)| kelvin as f32)
                .filter(|k| (273.0..=398.0).contains(k))
                .fold(0.0f32, f32::max);
            if hottest > 273.0 {
                snapshot.thermal_c = Some(hottest - 273.15);
            }
        }

        Ok(snapshot)
    }
}

/// `luid_0x00000000_0x0001221c` -> LUID, for the D3DKMT clock query.
fn adapter_key_luid(key: &str) -> Option<LUID> {
    let rest = key.strip_prefix("luid_0x")?;
    let (high, low) = rest.split_once("_0x")?;
    Some(LUID {
        HighPart: u32::from_str_radix(high, 16).ok()? as i32,
        LowPart: u32::from_str_radix(low, 16).ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_instances_are_parsed() {
        let (luid, engine) = parse_engine_instance(
            "pid_1234_luid_0x00000000_0x00011643_phys_0_eng_4_engtype_Compute",
        )
        .unwrap();
        assert_eq!(luid, "luid_0x00000000_0x00011643");
        assert_eq!(engine, "Compute");
    }

    #[test]
    fn adapter_instances_are_parsed() {
        assert_eq!(
            parse_adapter_instance("luid_0x00000000_0x00011643_phys_0").unwrap(),
            "luid_0x00000000_0x00011643"
        );
    }

    #[test]
    fn processor_instances_are_parsed() {
        assert_eq!(parse_processor_instance("0,11"), Some((0, 11)));
        assert_eq!(parse_processor_instance("0,_Total"), None);
    }

    #[test]
    fn luid_round_trips() {
        let key = "luid_0x00000000_0x0001221c";
        let luid = adapter_key_luid(key).unwrap();
        assert_eq!(luid_key(luid), key);
    }

    #[test]
    fn x2_has_no_efficiency_tier() {
        let mut x2 = vec![CoreKind::Performance, CoreKind::Efficiency];
        normalize_kinds("Snapdragon X2 Elite X2E-88-100", &mut x2);
        assert_eq!(x2, vec![CoreKind::Performance, CoreKind::Performance]);

        let mut x1 = vec![CoreKind::Performance, CoreKind::Efficiency];
        normalize_kinds("Snapdragon X Elite X1E78100", &mut x1);
        assert_eq!(x1, vec![CoreKind::Performance, CoreKind::Efficiency]);
    }

    #[test]
    fn rails_are_data_driven() {
        let x1 = parse_power(vec![
            ("sys".to_string(), 33_000.0),
            ("psu_usb".to_string(), 32_000.0),
            ("cpu_cluster_0".to_string(), 8_000.0),
            ("cpu_cluster_1".to_string(), 4_000.0),
            ("gpu".to_string(), 100.0),
            ("_total".to_string(), 33_000.0),
        ]);
        assert_eq!(x1.sys_w, Some(33.0));
        assert_eq!(x1.cpu_w, Some(12.0));
        assert_eq!(x1.gpu_w, Some(0.1));
        assert_eq!(x1.npu_w, None);

        let x2 = parse_power(vec![
            ("SYS".to_string(), 40_000.0),
            ("cpu_cluster_0".to_string(), 10_000.0),
            ("cpu_cluster_3".to_string(), 4_000.0),
            ("gpu_0".to_string(), 500.0),
            ("npu".to_string(), 1_500.0),
        ]);
        assert_eq!(x2.sys_w, Some(40.0));
        assert_eq!(x2.cpu_w, Some(14.0));
        assert_eq!(x2.gpu_w, Some(0.5));
        assert_eq!(x2.npu_w, Some(1.5));
    }

    #[test]
    fn sys_falls_back_to_total_and_psu() {
        let total = parse_power(vec![("_total".to_string(), 20_000.0)]);
        assert_eq!(total.sys_w, Some(20.0));
        let psu = parse_power(vec![("psu_usb".to_string(), 18_000.0)]);
        assert_eq!(psu.sys_w, Some(18.0));
    }

    /// These structures are passed to Win32; a field reorder or a size change
    /// would be silently accepted by the compiler and corrupt the call.
    #[test]
    fn ffi_layouts_match_the_c_abi() {
        use std::mem::{offset_of, size_of};

        assert_eq!(size_of::<FmtValue>(), 16);
        assert_eq!(offset_of!(FmtValue, value), 8);
        assert_eq!(size_of::<FmtItem>(), 24);
        assert_eq!(size_of::<KmtOpen>(), 12);
        assert_eq!(size_of::<KmtQuery>(), 24);
        assert_eq!(size_of::<NodePerfData>(), 56);
        assert_eq!(size_of::<GroupAffinity>(), 16);
        assert_eq!(size_of::<LpiExHead>(), 8);
        assert_eq!(size_of::<ProcessorHead>(), 24);
        assert_eq!(size_of::<SystemPowerStatus>(), 12);
    }
}
