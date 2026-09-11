/** guard.rs - 浮窗层级低频守护
 *
 * 背景：浮窗层级漂移的触发源具有长尾特征 —— 显示器热插拔、分辨率切换、
 * 资源管理器（explorer.exe）重启、跨进程 topmost 窗口竞争等，无法全部通过
 * 事件钩子覆盖。事件驱动的修复在任何未枚举到的场景下都会静默失效。
 *
 * 本模块以固定低频轮询作为兜底：仅校验 topmost 标记是否完好，标记正常时
 * 不做任何窗口操作，因此对用户前台窗口顺序零扰动；标记确实丢失时才触发
 * 一次确定性修复。
 *
 * 成本控制：
 * - 轮询周期 10 秒，每次仅对"当前可见"的浮窗执行一次 `GetWindowLongPtrW`；
 * - 浮窗全部隐藏时提前返回，不产生任何系统调用；
 * - 常态下 `SetWindowPos` 调用次数为 0。
 */
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use super::zorder;
use super::{topmost_preference, OVERLAY_WINDOW_LABELS, SETTINGS_WINDOW};

/** 守护轮询周期。取值需在"及时性"与"系统调用开销"之间平衡。 */
const GUARD_INTERVAL: Duration = Duration::from_secs(10);

/**
 * 受守护的窗口标签。
 *
 * 在浮窗集合之外额外纳入设置窗口：它同样以 `alwaysOnTop + skipTaskbar +
 * transparent` 呈现，具备完全一致的层级漂移风险，但不参与浮窗互斥仲裁，
 * 因此不并入 `OVERLAY_WINDOW_LABELS`，仅在此处独立守护。
 */
fn guarded_window_labels() -> Vec<&'static str> {
    let mut labels = OVERLAY_WINDOW_LABELS.to_vec();
    labels.push(SETTINGS_WINDOW);
    labels
}

/** 标记守护循环是否已启动，避免重复生成线程。 */
static GUARD_STARTED: AtomicBool = AtomicBool::new(false);

/**
 * 启动层级守护循环。重复调用是安全的，只有首次生效。
 *
 * 循环在独立线程中运行，通过 `AppHandle` 访问窗口而无需主线程参与；
 * 应用退出时线程随进程结束，无需显式停止。
 */
pub(crate) fn start_overlay_guard(app: AppHandle) {
    if GUARD_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = Arc::new(app);
    std::thread::spawn(move || loop {
        std::thread::sleep(GUARD_INTERVAL);
        run_guard_pass(&app);
    });
}

/**
 * 执行一次守护校验。返回本次被修复的窗口标签数量。
 *
 * 尊重用户偏好：只有当 `main_window_always_on_top` 为真时才修复层级。
 * 关闭置顶后，窗口层级完全交由系统决定，守护循环不得干预。
 */
pub(crate) fn run_guard_pass(app: &AppHandle) -> usize {
    match topmost_preference(app) {
        Some(true) => {}
        // 用户已关闭置顶，或设置尚未就绪：均不介入
        Some(false) | None => return 0,
    }

    let windows = guarded_window_labels()
        .into_iter()
        .filter_map(|label| {
            let window = app.get_webview_window(label)?;
            // 只守护可见窗口：隐藏窗口的层级无意义，且其 topmost 标记
            // 会在下次 show() 时由 present_overlay 重新施加。
            window.is_visible().ok().filter(|visible| *visible)?;
            Some((label, window))
        })
        .collect::<Vec<_>>();

    if windows.is_empty() {
        return 0;
    }

    match zorder::verify_all_on_top(windows) {
        Ok(repaired) => {
            for label in &repaired {
                eprintln!("守护检测到 {label} 窗口层级丢失，已修复");
            }
            repaired.len()
        }
        Err(error) => {
            eprintln!("层级守护执行失败: {error}");
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_covers_all_overlays_plus_settings_window() {
        let labels = guarded_window_labels();

        // 浮窗集合必须被完整覆盖，遗漏任何一个都会留下层级漂移的死角
        for label in OVERLAY_WINDOW_LABELS {
            assert!(labels.contains(&label), "守护集合缺少浮窗 {label}");
        }
        // 设置窗口同样以 alwaysOnTop 呈现，需独立守护
        assert!(labels.contains(&SETTINGS_WINDOW));
        assert_eq!(labels.len(), OVERLAY_WINDOW_LABELS.len() + 1);
    }
}
