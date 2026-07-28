mod commands;
mod feedback;
mod reminder;
mod role_packs;
mod settings;
mod tray;
mod windowing;

use std::str::FromStr;

use commands::{
    role_packs as role_pack_commands, settings as settings_commands,
    settings_transfer as settings_transfer_commands, system_info, window,
};
use tauri::{AppHandle, Manager, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as AutostartExt};
use tauri_plugin_global_shortcut::{
    Builder as GlobalShortcutBuilder, GlobalShortcutExt, Shortcut, ShortcutState,
};

pub(crate) fn parse_chat_shortcut(shortcut: &str) -> Result<Shortcut, String> {
    Shortcut::from_str(shortcut.trim()).map_err(|error| format!("快捷键格式无效: {error}"))
}

pub(crate) fn sync_chat_shortcut(
    app: &AppHandle,
    shortcut: &str,
    enabled: bool,
) -> Result<bool, String> {
    let manager = app.global_shortcut();
    manager
        .unregister_all()
        .map_err(|error| error.to_string())?;
    if !enabled {
        return Ok(false);
    }
    let shortcut = parse_chat_shortcut(shortcut)?;
    match manager.on_shortcut(shortcut, |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let _ = windowing::toggle_chat_window(app);
        }
    }) {
        Ok(()) => Ok(true),
        Err(error) => {
            eprintln!("无法注册聊天快捷键 {}: {error}", shortcut.to_string());
            Ok(false)
        }
    }
}

pub(crate) fn sync_autostart(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    let current = manager.is_enabled().map_err(|error| error.to_string())?;
    if current == enabled {
        return Ok(());
    }
    if enabled {
        manager.enable().map_err(|error| error.to_string())
    } else {
        manager.disable().map_err(|error| error.to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .register_uri_scheme_protocol("role-pack", |context, request| {
            let role_id = request.uri().path().trim_start_matches('/');
            let origin = request
                .headers()
                .get("origin")
                .and_then(|value| value.to_str().ok());
            let allowed_origin = match origin {
                Some("http://tauri.localhost") => Some("http://tauri.localhost"),
                Some("http://localhost:1420") => Some("http://localhost:1420"),
                _ => None,
            };
            let response = role_packs::resource_response(context.app_handle(), role_id);
            match response {
                Ok((content, content_type)) => {
                    let mut builder = tauri::http::Response::builder()
                        .header("content-type", content_type)
                        .header("cache-control", "no-store");
                    if let Some(origin) = allowed_origin {
                        builder = builder
                            .header("access-control-allow-origin", origin)
                            .header("vary", "origin");
                    }
                    builder.body(content).expect("角色资源响应构建失败")
                }
                Err(_) => tauri::http::Response::builder()
                    .status(tauri::http::StatusCode::NOT_FOUND)
                    .body(Vec::new())
                    .expect("角色资源错误响应构建失败"),
            }
        })
        .manage(system_info::SystemMonitor::new())
        .manage(feedback::SystemFeedbackState::default())
        .manage(windowing::OverlayState::default())
        .manage(reminder::ReminderState::default())
        .plugin(GlobalShortcutBuilder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .on_menu_event(|app, event| tray::handle_menu_event(app, event))
        .invoke_handler(tauri::generate_handler![
            system_info::get_system_info,
            window::resize_main_window,
            window::resize_info_window,
            window::toggle_chat_window,
            window::show_chat_window,
            window::hide_chat_window,
            window::hide_settings_window,
            window::hide_reminder_window,
            window::hide_system_feedback_window,
            window::active_reminder_payload,
            window::active_system_feedback_payload,
            window::show_system_feedback,
            window::dismiss_system_feedback,
            window::dismiss_reminder_window,
            window::complete_reminder_window,
            window::snooze_reminder,
            window::get_reminder_activity,
            window::clear_reminder_activity,
            window::pause_reminder_until_tomorrow,
            window::preview_reminder_window,
            window::set_info_window_visible,
            window::show_main_context_menu,
            window::hide_main_context_menu,
            window::show_main_context_status,
            window::pause_all_reminders_until_tomorrow,
            window::show_reminders_paused_confirmation,
            window::show_main_settings_window,
            window::exit_application,
            settings_commands::load_app_settings,
            role_pack_commands::list_installed_role_packs,
            role_pack_commands::install_role_pack,
            role_pack_commands::remove_role_pack,
            settings_transfer_commands::export_portable_settings,
            settings_transfer_commands::import_portable_settings,
            settings_transfer_commands::complete_settings_onboarding,
            settings_commands::save_pet_scale,
            settings_commands::set_pet_role,
            settings_commands::save_main_window_position,
            settings_commands::save_settings_window_bounds,
            settings_commands::set_info_mode,
            settings_commands::set_size_locked,
            settings_commands::set_shortcut_enabled,
            settings_commands::set_chat_shortcut,
            settings_commands::set_launch_at_startup,
            settings_commands::set_main_window_always_on_top,
            settings_commands::set_main_window_show_in_taskbar,
            settings_commands::set_reminder_quiet_hours,
            settings_commands::create_reminder,
            settings_commands::update_reminder,
            settings_commands::delete_reminder,
            settings_commands::set_reminder_enabled,
            settings_commands::reset_main_window_position,
            settings_commands::reset_settings_window_bounds,
            settings_commands::reset_all_settings,
        ])
        .setup(|app| {
            let settings_state = settings::SettingsState::load(&app.handle())?;
            let reminder_history = reminder::ReminderHistoryState::load(&app.handle())?;
            let mut initial_settings = settings_state.get()?;
            app.manage(settings_state);
            app.manage(reminder_history);

            #[cfg(target_os = "windows")]
            {
                for label in [
                    windowing::MAIN_WINDOW,
                    windowing::CONTEXT_MENU_WINDOW,
                    windowing::CHAT_WINDOW,
                    windowing::INFO_WINDOW,
                    windowing::REMINDER_WINDOW,
                    windowing::SYSTEM_FEEDBACK_WINDOW,
                    windowing::SETTINGS_WINDOW,
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

            if let Err(error) = sync_autostart(&app.handle(), initial_settings.launch_at_startup) {
                eprintln!("无法同步开机自启设置: {error}");
                if let Some(settings) = app.try_state::<settings::SettingsState>() {
                    initial_settings = settings.set_launch_at_startup(false)?;
                }
            }
            windowing::apply_main_window_settings(&app.handle(), &initial_settings)?;
            if !sync_chat_shortcut(
                &app.handle(),
                &initial_settings.chat_shortcut,
                initial_settings.shortcut_enabled,
            )? && initial_settings.shortcut_enabled
            {
                if let Some(settings) = app.try_state::<settings::SettingsState>() {
                    initial_settings = settings.set_shortcut_enabled(false)?;
                }
            }

            tray::create_tray(&app.handle())?;
            windowing::restore_main_window_position(
                &app.handle(),
                initial_settings.main_position.map(Into::into),
            )?;
            reminder::sync_from_settings(&app.handle())?;
            reminder::start_scheduler(app.handle().clone());
            windowing::sync_info_window_visibility(&app.handle())?;
            windowing::sync_reminder_window_visibility(&app.handle())?;
            windowing::sync_system_feedback_window_visibility(&app.handle())?;

            if let Some(main) = app.get_webview_window(windowing::MAIN_WINDOW) {
                let app_handle = app.handle().clone();
                main.on_window_event(move |event| {
                    if matches!(event, WindowEvent::Moved(_)) {
                        let _ = windowing::hide_context_menu(&app_handle);
                        windowing::reposition_visible_overlays(&app_handle);
                        if let Some(settings) = app_handle.try_state::<settings::SettingsState>() {
                            if let Some(window) =
                                app_handle.get_webview_window(windowing::MAIN_WINDOW)
                            {
                                if let Ok(position) = window.outer_position() {
                                    let _ = settings.save_main_position_throttled(position.into());
                                }
                            }
                        }
                    }
                    if matches!(event, WindowEvent::ScaleFactorChanged { .. }) {
                        let _ = windowing::hide_context_menu(&app_handle);
                        if let Err(error) = windowing::reclamp_main_window_position(&app_handle) {
                            eprintln!("无法在 DPI 变化后重新约束桌宠窗口: {error}");
                        }
                        if let Some(settings) = app_handle.try_state::<settings::SettingsState>() {
                            if let Some(window) =
                                app_handle.get_webview_window(windowing::MAIN_WINDOW)
                            {
                                if let Ok(position) = window.outer_position() {
                                    let _ = settings.save_main_position_throttled(position.into());
                                }
                            }
                        }
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
