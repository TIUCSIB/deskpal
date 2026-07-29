use std::{thread, time::Duration};

use tauri::{AppHandle, Manager};

use super::MAIN_WINDOW;

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT,
};

const CLICK_FORWARD_DELAY: Duration = Duration::from_millis(12);

/** passthrough.rs - 主窗口左键透传辅助逻辑 */
pub fn forward_main_left_click(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    #[cfg(target_os = "windows")]
    {
        window
            .set_ignore_cursor_events(true)
            .map_err(|error| error.to_string())?;
        thread::sleep(CLICK_FORWARD_DELAY);
        let send_result = unsafe { send_left_click() };
        thread::sleep(CLICK_FORWARD_DELAY);
        let restore_result = window.set_ignore_cursor_events(false);
        send_result?;
        restore_result.map_err(|error| error.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
unsafe fn send_left_click() -> Result<(), String> {
    let inputs = [
        mouse_input(MOUSEEVENTF_LEFTDOWN),
        mouse_input(MOUSEEVENTF_LEFTUP),
    ];
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };
    if sent == inputs.len() as u32 {
        Ok(())
    } else {
        Err("无法将左键点击透传到底层窗口。".to_string())
    }
}

#[cfg(target_os = "windows")]
fn mouse_input(flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
