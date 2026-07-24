use serde::Serialize;
use sysinfo::{Disks, System};
use tauri::command;

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
    /// 磁盘使用率 (0-100)，取第一个磁盘
    pub disk_usage: f64,
    /// 系统运行时间（秒）
    pub uptime_secs: u64,
}

#[command]
pub fn get_system_info() -> SystemInfo {
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();

    // CPU 使用率：所有核心的平均值
    let cpu_usage: f32 =
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;

    // 内存信息
    let memory_total_mb = sys.total_memory() / 1024 / 1024;
    let memory_used_mb = sys.used_memory() / 1024 / 1024;
    let memory_usage = if sys.total_memory() > 0 {
        sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0
    } else {
        0.0
    };

    // 磁盘使用率：取第一个磁盘
    let disks = Disks::new_with_refreshed_list();
    let disk_usage = disks
        .first()
        .map(|d| {
            if d.total_space() > 0 {
                (d.total_space() - d.available_space()) as f64 / d.total_space() as f64 * 100.0
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);

    // 系统运行时间
    let uptime_secs = System::uptime();

    SystemInfo {
        cpu_usage: (cpu_usage * 10.0).round() / 10.0,
        memory_usage: (memory_usage * 10.0).round() / 10.0,
        memory_used_mb,
        memory_total_mb,
        disk_usage: (disk_usage * 10.0).round() / 10.0,
        uptime_secs,
    }
}
