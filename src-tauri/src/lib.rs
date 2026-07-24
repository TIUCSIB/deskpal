mod commands;
mod menu;
mod windowing;

use commands::{system_info, window};
use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            system_info::get_system_info,
            window::resize_main_window,
            window::toggle_chat_window,
            window::hide_chat_window,
            window::set_info_window_visible,
            window::show_context_menu,
        ])
        .on_menu_event(menu::handle_menu_event)
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                for label in [
                    windowing::MAIN_WINDOW,
                    windowing::CHAT_WINDOW,
                    windowing::INFO_WINDOW,
                ] {
                    if let Some(window) = app.get_webview_window(label) {
                        if let Err(error) =
                            window.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)))
                        {
                            eprintln!("无法设置 {label} 窗口透明背景: {error}");
                        }
                    }
                }

                if let Some(info) = app.get_webview_window(windowing::INFO_WINDOW) {
                    if let Err(error) = info.set_ignore_cursor_events(true) {
                        eprintln!("无法设置系统信息窗口点击穿透: {error}");
                    }
                }
            }

            if let Some(main) = app.get_webview_window(windowing::MAIN_WINDOW) {
                let app_handle = app.handle().clone();
                main.on_window_event(move |event| {
                    if matches!(
                        event,
                        WindowEvent::Moved(_) | WindowEvent::ScaleFactorChanged { .. }
                    ) {
                        windowing::reposition_visible_overlays(&app_handle);
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
