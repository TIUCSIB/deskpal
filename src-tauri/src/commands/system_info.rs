use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::Serialize;
use sysinfo::{Disks, Networks, System};
use tauri::State;

/// 系统信息快照，传递给前端
#[derive(Serialize, Clone)]
pub struct SystemInfo {
    /// CPU 总使用率 (0-100)
    pub cpu_usage: f32,
    /// 内存使用率 (0-100)
    pub memory_usage: f64,
    /// 已用内存 (MB)
    pub memory_used_mb: u64,
    /// 总内存 (MB)
    pub memory_total_mb: u64,
    /// 存储容量使用率 (0-100)
    pub disk_usage: f64,
    /// 网络下行速度 (KiB/s)
    pub network_down_kbps: f64,
    /// 网络上行速度 (KiB/s)
    pub network_up_kbps: f64,
    /// NLM 定义的网络可用性：已断开为 false，任何已连接状态为 true，未知为 null。
    pub network_connected: Option<bool>,
    /// 电池电量百分比；无电池或平台不支持时为空
    pub battery_percent: Option<u8>,
    /// 是否接通交流电源；无电池、状态未知或平台不支持时为空。
    ///
    /// 字段名为兼容前端保留；它表示 on-AC，而不是电池是否正在充电。
    pub battery_charging: Option<bool>,
    /// 用户距离上次输入的秒数；平台不支持时为空
    pub idle_seconds: Option<u64>,
    /// 系统运行时间（秒）
    pub uptime_secs: u64,
}

struct MonitorData {
    system: System,
    disks: Disks,
    networks: Networks,
    last_network_refresh: Instant,
}

/// 持久系统监控状态，保留 CPU 与网络连续采样基线
pub struct SystemMonitor {
    inner: Mutex<MonitorData>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_all();
        system.refresh_memory();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        Self {
            inner: Mutex::new(MonitorData {
                system,
                disks,
                networks,
                last_network_refresh: Instant::now(),
            }),
        }
    }

    /// Returns the current user idle duration independently of a full snapshot.
    ///
    /// This makes the platform accessor available to reminder scheduling without
    /// contending for the monitor's metric sampling state.
    pub fn idle_seconds(&self) -> Option<u64> {
        platform_idle_seconds()
    }

    fn snapshot(&self) -> Result<SystemInfo, String> {
        let mut data = self
            .inner
            .lock()
            .map_err(|_| "系统监控状态暂时不可用".to_string())?;

        data.system.refresh_cpu_all();
        data.system.refresh_memory();
        data.disks.refresh(true);

        let network_elapsed = data.last_network_refresh.elapsed();
        data.networks.refresh(true);
        data.last_network_refresh = Instant::now();

        let cpu_usage = if data.system.cpus().is_empty() {
            0.0
        } else {
            data.system
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .sum::<f32>()
                / data.system.cpus().len() as f32
        };

        let memory_total = data.system.total_memory();
        let memory_used = data.system.used_memory();
        let memory_usage = if memory_total > 0 {
            memory_used as f64 / memory_total as f64 * 100.0
        } else {
            0.0
        };

        let (storage_total, storage_available) =
            data.disks
                .iter()
                .fold((0_u64, 0_u64), |(total, available), disk| {
                    (
                        total.saturating_add(disk.total_space()),
                        available.saturating_add(disk.available_space()),
                    )
                });
        let disk_usage = if storage_total > 0 {
            storage_total.saturating_sub(storage_available) as f64 / storage_total as f64 * 100.0
        } else {
            0.0
        };

        // sysinfo exposes bytes received/transmitted since the previous refresh,
        // not a duration-normalized rate. Normalize by the real sampling interval.
        let (network_down_bytes, network_up_bytes) =
            data.networks
                .iter()
                .fold((0_u64, 0_u64), |(down, up), (_, network)| {
                    (
                        down.saturating_add(network.received()),
                        up.saturating_add(network.transmitted()),
                    )
                });
        let network_connected =
            platform_network_connectivity().and_then(NetworkConnectivity::is_connected);
        let (battery_percent, battery_charging) = platform_battery_status();

        Ok(SystemInfo {
            cpu_usage: round_f32(cpu_usage),
            memory_usage: round_f64(memory_usage),
            memory_used_mb: memory_used / 1024 / 1024,
            memory_total_mb: memory_total / 1024 / 1024,
            disk_usage: round_f64(disk_usage),
            network_down_kbps: round_f64(bytes_to_kib_per_second(
                network_down_bytes,
                network_elapsed,
            )),
            network_up_kbps: round_f64(bytes_to_kib_per_second(network_up_bytes, network_elapsed)),
            network_connected,
            battery_percent,
            battery_charging,
            idle_seconds: self.idle_seconds(),
            uptime_secs: System::uptime(),
        })
    }
}

/// NLM connectivity categories. `Unknown` means NLM returned flags outside the
/// documented connectivity mask, so callers do not mistake an API ambiguity for
/// a disconnected device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NetworkConnectivity {
    Disconnected,
    ConnectedNoTraffic,
    LocalOnly,
    Internet,
    Unknown,
}

impl NetworkConnectivity {
    fn is_connected(self) -> Option<bool> {
        match self {
            Self::Disconnected => Some(false),
            Self::ConnectedNoTraffic | Self::LocalOnly | Self::Internet => Some(true),
            Self::Unknown => None,
        }
    }
}

const NLM_IPV4_NOTRAFFIC: u32 = 0x0001;
const NLM_IPV6_NOTRAFFIC: u32 = 0x0002;
const NLM_IPV4_SUBNET: u32 = 0x0010;
const NLM_IPV4_LOCALNETWORK: u32 = 0x0020;
const NLM_IPV4_INTERNET: u32 = 0x0040;
const NLM_IPV6_SUBNET: u32 = 0x0100;
const NLM_IPV6_LOCALNETWORK: u32 = 0x0200;
const NLM_IPV6_INTERNET: u32 = 0x0400;
const NLM_KNOWN_FLAGS: u32 = NLM_IPV4_NOTRAFFIC
    | NLM_IPV6_NOTRAFFIC
    | NLM_IPV4_SUBNET
    | NLM_IPV4_LOCALNETWORK
    | NLM_IPV4_INTERNET
    | NLM_IPV6_SUBNET
    | NLM_IPV6_LOCALNETWORK
    | NLM_IPV6_INTERNET;

/// Classifies NLM's bit flags without probing the network. Internet takes
/// precedence over local reachability; a subnet-only connection is local-only.
fn classify_nlm_connectivity(flags: u32) -> NetworkConnectivity {
    if flags == 0 {
        return NetworkConnectivity::Disconnected;
    }
    if flags & !NLM_KNOWN_FLAGS != 0 {
        return NetworkConnectivity::Unknown;
    }
    if flags & (NLM_IPV4_INTERNET | NLM_IPV6_INTERNET) != 0 {
        return NetworkConnectivity::Internet;
    }
    if flags & (NLM_IPV4_LOCALNETWORK | NLM_IPV6_LOCALNETWORK | NLM_IPV4_SUBNET | NLM_IPV6_SUBNET)
        != 0
    {
        return NetworkConnectivity::LocalOnly;
    }
    if flags & (NLM_IPV4_NOTRAFFIC | NLM_IPV6_NOTRAFFIC) != 0 {
        return NetworkConnectivity::ConnectedNoTraffic;
    }

    NetworkConnectivity::Unknown
}

#[cfg(target_os = "windows")]
fn platform_network_connectivity() -> Option<NetworkConnectivity> {
    use windows::{
        core::GUID,
        Win32::{
            Networking::NetworkListManager::INetworkListManager,
            System::Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
            },
        },
    };

    const CLSID_NETWORK_LIST_MANAGER: GUID =
        GUID::from_u128(0xdcb00c01_570f_4a9b_8d69_199fdba5723b);

    // Tauri may call this from a thread that is already initialized with a
    // different apartment model. In that case COM remains usable; only undo an
    // initialization this call successfully performed.
    let should_uninitialize = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).is_ok() };
    let connectivity = (|| unsafe {
        let manager: INetworkListManager =
            CoCreateInstance(&CLSID_NETWORK_LIST_MANAGER, None, CLSCTX_ALL).ok()?;
        let flags = manager.GetConnectivity().ok()?.0 as u32;
        Some(classify_nlm_connectivity(flags))
    })();

    if should_uninitialize {
        unsafe { CoUninitialize() };
    }

    connectivity
}

#[cfg(not(target_os = "windows"))]
fn platform_network_connectivity() -> Option<NetworkConnectivity> {
    None
}

#[cfg(target_os = "windows")]
fn platform_battery_status() -> (Option<u8>, Option<bool>) {
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

    let mut status = unsafe { std::mem::zeroed::<SYSTEM_POWER_STATUS>() };
    if unsafe { GetSystemPowerStatus(&mut status) } == 0 {
        return (None, None);
    }

    // BatteryFlag bit 7 means no system battery. BatteryLifePercent and
    // ACLineStatus use 255 for an unavailable value.
    battery_status_from_raw(
        status.BatteryFlag & 0x80 == 0,
        status.BatteryLifePercent,
        status.ACLineStatus,
    )
}

#[cfg(not(target_os = "windows"))]
fn platform_battery_status() -> (Option<u8>, Option<bool>) {
    (None, None)
}

fn battery_status_from_raw(
    has_battery: bool,
    battery_life_percent: u8,
    ac_line_status: u8,
) -> (Option<u8>, Option<bool>) {
    if !has_battery {
        return (None, None);
    }

    let percent = (battery_life_percent <= 100).then_some(battery_life_percent);
    let on_ac = match ac_line_status {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    };
    (percent, on_ac)
}

#[cfg(target_os = "windows")]
fn platform_idle_seconds() -> Option<u64> {
    use windows_sys::Win32::{
        System::SystemInformation::GetTickCount,
        UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
    };

    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    if unsafe { GetLastInputInfo(&mut info) } == 0 {
        return None;
    }

    // Both values are 32-bit tick counts; wrapping subtraction correctly
    // handles the approximately 49.7-day GetTickCount rollover.
    let elapsed_ms = unsafe { GetTickCount() }.wrapping_sub(info.dwTime) as u64;
    Some(elapsed_ms / 1000)
}

#[cfg(not(target_os = "windows"))]
fn platform_idle_seconds() -> Option<u64> {
    None
}

#[tauri::command]
pub fn get_system_info(monitor: State<'_, SystemMonitor>) -> Result<SystemInfo, String> {
    monitor.snapshot()
}

fn bytes_to_kib_per_second(bytes: u64, elapsed: Duration) -> f64 {
    let elapsed_secs = elapsed.as_secs_f64();
    if elapsed_secs > 0.0 {
        bytes as f64 / elapsed_secs / 1024.0
    } else {
        0.0
    }
}

fn round_f32(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

fn round_f64(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn monitor_returns_bounded_consecutive_snapshots() {
        let monitor = SystemMonitor::new();
        let first = monitor.snapshot().expect("first system snapshot");
        thread::sleep(Duration::from_millis(250));
        let second = monitor.snapshot().expect("second system snapshot");

        assert!((0.0..=100.0).contains(&second.cpu_usage));
        assert!((0.0..=100.0).contains(&second.memory_usage));
        assert!((0.0..=100.0).contains(&second.disk_usage));
        assert!(second.network_down_kbps >= 0.0);
        assert!(second.network_up_kbps >= 0.0);
        assert!(second.memory_total_mb > 0);
        assert!(second.uptime_secs >= first.uptime_secs);
        assert!(second.battery_percent.is_none_or(|percent| percent <= 100));
    }

    #[test]
    fn nlm_connectivity_categories_follow_product_precedence() {
        assert_eq!(
            classify_nlm_connectivity(0),
            NetworkConnectivity::Disconnected
        );
        assert_eq!(
            classify_nlm_connectivity(NLM_IPV4_NOTRAFFIC | NLM_IPV6_NOTRAFFIC),
            NetworkConnectivity::ConnectedNoTraffic
        );
        assert_eq!(
            classify_nlm_connectivity(NLM_IPV4_SUBNET | NLM_IPV6_LOCALNETWORK),
            NetworkConnectivity::LocalOnly
        );
        assert_eq!(
            classify_nlm_connectivity(NLM_IPV4_INTERNET | NLM_IPV6_LOCALNETWORK),
            NetworkConnectivity::Internet
        );
        assert_eq!(
            classify_nlm_connectivity(0x8000),
            NetworkConnectivity::Unknown
        );
    }

    #[test]
    fn connectivity_projection_preserves_unknown_state() {
        assert_eq!(
            NetworkConnectivity::Disconnected.is_connected(),
            Some(false)
        );
        assert_eq!(
            NetworkConnectivity::ConnectedNoTraffic.is_connected(),
            Some(true)
        );
        assert_eq!(NetworkConnectivity::LocalOnly.is_connected(), Some(true));
        assert_eq!(NetworkConnectivity::Internet.is_connected(), Some(true));
        assert_eq!(NetworkConnectivity::Unknown.is_connected(), None);
    }

    #[test]
    fn battery_status_keeps_percent_nullable_and_reports_ac_separately() {
        assert_eq!(battery_status_from_raw(false, 75, 1), (None, None));
        assert_eq!(battery_status_from_raw(true, 75, 1), (Some(75), Some(true)));
        assert_eq!(battery_status_from_raw(true, 0, 0), (Some(0), Some(false)));
        assert_eq!(battery_status_from_raw(true, 255, 255), (None, None));
        assert_eq!(battery_status_from_raw(true, 101, 1), (None, Some(true)));
    }

    #[test]
    fn network_rate_is_normalized_by_sampling_duration() {
        assert_eq!(bytes_to_kib_per_second(2048, Duration::from_secs(2)), 1.0);
        assert_eq!(bytes_to_kib_per_second(1024, Duration::ZERO), 0.0);
    }
}
