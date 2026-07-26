use std::sync::Mutex;

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
    /// 网络下行速度 (KB/s)
    pub network_down_kbps: f64,
    /// 网络上行速度 (KB/s)
    pub network_up_kbps: f64,
    /// 系统运行时间（秒）
    pub uptime_secs: u64,
}

struct MonitorData {
    system: System,
    disks: Disks,
    networks: Networks,
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
            }),
        }
    }

    fn snapshot(&self) -> Result<SystemInfo, String> {
        let mut data = self
            .inner
            .lock()
            .map_err(|_| "系统监控状态暂时不可用".to_string())?;

        data.system.refresh_cpu_all();
        data.system.refresh_memory();
        data.disks.refresh(true);
        data.networks.refresh(true);

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

        let (storage_total, storage_available) = data
            .disks
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

        let (network_down_bytes, network_up_bytes) = data
            .networks
            .iter()
            .fold((0_u64, 0_u64), |(down, up), (_, network)| {
                (
                    down.saturating_add(network.received()),
                    up.saturating_add(network.transmitted()),
                )
            });

        Ok(SystemInfo {
            cpu_usage: round_f32(cpu_usage),
            memory_usage: round_f64(memory_usage),
            memory_used_mb: memory_used / 1024 / 1024,
            memory_total_mb: memory_total / 1024 / 1024,
            disk_usage: round_f64(disk_usage),
            network_down_kbps: round_f64(network_down_bytes as f64 / 1024.0),
            network_up_kbps: round_f64(network_up_bytes as f64 / 1024.0),
            uptime_secs: System::uptime(),
        })
    }
}

#[tauri::command]
pub fn get_system_info(monitor: State<'_, SystemMonitor>) -> Result<SystemInfo, String> {
    monitor.snapshot()
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
    }
}
