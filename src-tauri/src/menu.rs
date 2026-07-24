use tauri::{
    menu::{MenuBuilder, MenuEvent, SubmenuBuilder},
    AppHandle, Emitter, LogicalPosition, Manager, WebviewWindow,
};

use crate::windowing;

const CHAT_ID: &str = "chat";
const SIZE_SMALL_ID: &str = "size-small";
const SIZE_MEDIUM_ID: &str = "size-medium";
const SIZE_LARGE_ID: &str = "size-large";
const QUIT_ID: &str = "quit";

fn size_label(label: &str, selected: bool) -> String {
    if selected {
        format!("✓ {label}")
    } else {
        label.to_string()
    }
}

pub fn show_context_menu(
    app: &AppHandle,
    window: &WebviewWindow,
    x: f64,
    y: f64,
    scale: f64,
) -> Result<(), String> {
    if let Some(info) = app.get_webview_window(windowing::INFO_WINDOW) {
        let _ = info.hide();
    }

    let size_menu = SubmenuBuilder::new(app, "大小")
        .text(SIZE_SMALL_ID, size_label("小", (scale - 0.6).abs() < 0.05))
        .text(SIZE_MEDIUM_ID, size_label("中", (scale - 1.0).abs() < 0.05))
        .text(SIZE_LARGE_ID, size_label("大", (scale - 1.5).abs() < 0.05))
        .build()
        .map_err(|error| error.to_string())?;
    let menu = MenuBuilder::new(app)
        .text(CHAT_ID, "聊天")
        .separator()
        .item(&size_menu)
        .separator()
        .text(QUIT_ID, "退出")
        .build()
        .map_err(|error| error.to_string())?;

    window
        .popup_menu_at(&menu, LogicalPosition::new(x, y))
        .map_err(|error| error.to_string())
}

pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        CHAT_ID => {
            if let Err(error) = windowing::toggle_chat_window(app) {
                eprintln!("无法切换聊天窗口: {error}");
            }
        }
        SIZE_SMALL_ID => emit_scale(app, 0.6),
        SIZE_MEDIUM_ID => emit_scale(app, 1.0),
        SIZE_LARGE_ID => emit_scale(app, 1.5),
        QUIT_ID => app.exit(0),
        _ => {}
    }
}

fn emit_scale(app: &AppHandle, scale: f64) {
    let Some(main) = app.get_webview_window(windowing::MAIN_WINDOW) else {
        return;
    };
    if let Err(error) = main.emit("pet://set-scale", scale) {
        eprintln!("无法发送桌宠缩放事件: {error}");
    }
}
