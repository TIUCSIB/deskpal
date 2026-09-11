/** zorder.rs - Windows 窗口 Z 序强制置顶
 *
 * 背景：Tauri 的 `set_always_on_top(true)` 只在窗口创建阶段写入一次
 * `WS_EX_TOPMOST` 扩展样式；一旦窗口被隐藏、被同层 topmost 窗口（浏览器、
 * 播放器等）激活覆盖，仅靠 `show()` 无法保证重新获得正确的相对顺序。
 *
 * 本模块提供显式的 `SetWindowPos(HWND_TOPMOST)` 调用作为确定性兜底。
 */
use tauri::WebviewWindow;

#[cfg(target_os = "windows")]
mod windows_impl {
    use tauri::WebviewWindow;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_TOPMOST,
    };

    /** 取得窗口句柄；窗口尚未创建句柄时返回 None。 */
    fn window_handle(window: &WebviewWindow) -> Option<HWND> {
        window.hwnd().ok().map(|handle| handle.0 as HWND)
    }

    /** 查询窗口当前是否带有 topmost 扩展样式。 */
    pub(super) fn is_topmost(window: &WebviewWindow) -> Option<bool> {
        let hwnd = window_handle(window)?;
        let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
        Some(style as u32 & WS_EX_TOPMOST != 0)
    }

    /** 强制窗口进入 topmost 层，但保持相对顺序由最后一次调用决定。 */
    pub(super) fn apply_topmost(window: &WebviewWindow, topmost: bool) -> Result<(), String> {
        let Some(hwnd) = window_handle(window) else {
            return Err("窗口句柄尚未就绪".to_string());
        };
        let insert_after = if topmost {
            HWND_TOPMOST
        } else {
            // HWND_NOTOPMOST = -2，windows-sys 未导出该常量，直接使用其值
            -2isize as HWND
        };
        let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW;
        let succeeded = unsafe { SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, flags) };
        if succeeded == 0 {
            return Err("SetWindowPos 置顶调用失败".to_string());
        }
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod windows_impl {
    use tauri::WebviewWindow;

    pub(super) fn is_topmost(_window: &WebviewWindow) -> Option<bool> {
        None
    }

    pub(super) fn apply_topmost(_window: &WebviewWindow, _topmost: bool) -> Result<(), String> {
        Ok(())
    }
}

use windows_impl::apply_topmost;

/**
 * 确保窗口稳定处于桌面最顶层（含抖动）。
 *
 * 执行顺序经过刻意设计：
 * 1. 先以 `SetWindowPos(HWND_TOPMOST)` 显式写入 topmost 标记 —— 即使扩展样式
 *    在窗口隐藏/重建后丢失，此调用也能恢复；
 * 2. 再追加一次轻量位置抖动（x + 1 → x），强制 Windows 在跨进程的同层
 *    topmost 竞争中把本窗口重新排到该层顶部。
 *
 * **仅在低频路径使用**（窗口创建、主窗口重呈现、用户主动唤起）。抖动包含
 * 两次异步位置写入，若在渲染线程恰好于两次写入之间合成一帧，会看到窗口
 * 横向偏移 1 物理像素的闪烁；高频路径（悬停驱动的浮窗显示）请改用
 * [`assert_on_top`]。
 */
pub(crate) fn ensure_on_top(window: &WebviewWindow) -> Result<(), String> {
    assert_on_top(window)?;
    nudge_z_order(window)
}

/**
 * 仅显式置顶，不做位置抖动。
 *
 * `SetWindowPos(HWND_TOPMOST)` 本身即保证窗口位于所有非 topmost 窗口之上，
 * 并在 topmost 层中置于顶部，因此对"标记正常、只需重申层级"的场景足够。
 * 本函数是高频路径（每次浮窗显示）与守护修复的安全选择：无位置写入，
 * 不存在 1 像素抖动风险。
 */
pub(crate) fn assert_on_top(window: &WebviewWindow) -> Result<(), String> {
    match apply_topmost(window, true) {
        Ok(()) => Ok(()),
        Err(error) => {
            // 句柄不可用时降级到 Tauri 自带能力，不向上传播错误
            eprintln!("显式置顶失败，降级为 Tauri 默认实现: {error}");
            window
                .set_always_on_top(true)
                .map_err(|error| error.to_string())
        }
    }
}

/** 通过一次位置抖动诱使 Windows 重排窗口 Z 序。 */
fn nudge_z_order(window: &WebviewWindow) -> Result<(), String> {
    let position = window.outer_position().map_err(|error| error.to_string())?;
    let nudged = tauri::PhysicalPosition::new(position.x.saturating_add(1), position.y);
    window
        .set_position(nudged)
        .map_err(|error| error.to_string())?;
    window
        .set_position(position)
        .map_err(|error| error.to_string())
}

/**
 * 批量确保一组窗口处于顶层。
 *
 * 用于系统级事件（DPI 变化、显示器切换、主窗口重新呈现）后的层级修复。
 * 返回失败窗口的标签列表，调用方可据此记录日志而不中断整体流程。
 *
 * 不执行位置抖动：批量场景下多个窗口同时抖动会叠加视觉扰动，且 `SetWindowPos`
 * 已保证 topmost 归属。
 */
pub(crate) fn ensure_all_on_top<'a, I>(windows: I) -> Vec<(&'a str, String)>
where
    I: IntoIterator<Item = (&'a str, WebviewWindow)>,
{
    let mut failures = Vec::new();
    for (label, window) in windows {
        if let Err(error) = assert_on_top(&window) {
            failures.push((label, error));
        }
    }
    failures
}

/**
 * 校验式置顶：仅在窗口确实丢失 topmost 标记时才修复。
 *
 * 与 `ensure_on_top` 的差异有两点：
 * - 先读取 `WS_EX_TOPMOST` 扩展样式，若标记完好则不做任何操作 —— 低频守护
 *   循环需要这种"无变化则无副作用"的语义，避免周期性 `SetWindowPos` 扰动
 *   用户当前的前台窗口顺序；
 * - 修复路径不执行位置抖动：标记丢失已被确证，`SetWindowPos` 足以恢复层级，
 *   抖动只会引入无谓的位置写入。
 *
 * 返回值语义：
 * - `Ok(true)`  —— 检测到标记丢失并已成功修复
 * - `Ok(false)` —— 标记完好，未做任何操作
 * - `Err(_)`    —— 标记丢失但修复失败
 */
pub(crate) fn verify_and_repair_on_top(window: &WebviewWindow) -> Result<bool, String> {
    match windows_impl::is_topmost(window) {
        Some(true) => Ok(false),
        // 句柄未就绪时无法查询（None）也归入修复路径，交由修复逻辑报错
        Some(false) | None => {
            assert_on_top(window)?;
            Ok(true)
        }
    }
}

/**
 * 批量校验式置顶，返回被修复窗口的标签列表。
 *
 * 供低频守护循环使用：正常情况下返回值恒为空，不产生任何窗口操作。
 */
pub(crate) fn verify_all_on_top<'a, I>(windows: I) -> Result<Vec<&'a str>, String>
where
    I: IntoIterator<Item = (&'a str, WebviewWindow)>,
{
    let mut repaired = Vec::new();
    for (label, window) in windows {
        match verify_and_repair_on_top(&window) {
            Ok(true) => repaired.push(label),
            Ok(false) => {}
            Err(error) => eprintln!("守护修复 {label} 窗口层级失败: {error}"),
        }
    }
    Ok(repaired)
}
