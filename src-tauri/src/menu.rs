use tauri::{
    menu::{MenuBuilder, MenuEvent, SubmenuBuilder},
    AppHandle, Emitter, LogicalPosition, Manager,
};

use crate::{
    reminder,
    settings::{AppSettings, SettingsState},
    windowing,
};

const OPEN_CHAT_ID: &str = "pet-context.open-chat";
const SHOW_STATUS_ID: &str = "pet-context.show-status";
const PAUSE_REMINDERS_ID: &str = "pet-context.pause-reminders";
const OPEN_SETTINGS_ID: &str = "pet-context.open-settings";
const ROLE_GUGA_ID: &str = "pet-context.role.guga";
const ROLE_MONTHLY_SALARY_CAT_ID: &str = "pet-context.role.monthly-salary-cat";
const ROLE_BROOM_WITCH_ID: &str = "pet-context.role.broom-witch";
const EXIT_ID: &str = "pet-context.exit";
const SETTINGS_UPDATED_EVENT: &str = "pet://settings-updated";

fn checked_label(label: &str, checked: bool) -> String {
    if checked {
        format!("✓ {label}")
    } else {
        label.to_string()
    }
}

fn current_settings(app: &AppHandle) -> AppSettings {
    app.try_state::<SettingsState>()
        .and_then(|settings| settings.get().ok())
        .unwrap_or_default()
}

pub fn show_context_menu(app: &AppHandle, x: f64, y: f64) -> Result<(), String> {
    let window = app
        .get_webview_window(windowing::MAIN_WINDOW)
        .ok_or_else(|| "找不到桌宠窗口".to_string())?;
    let settings = current_settings(app);
    let role_menu = SubmenuBuilder::new(app, "切换角色")
        .text(
            ROLE_GUGA_ID,
            checked_label("咕嘎", settings.pet_role == "guga"),
        )
        .text(
            ROLE_MONTHLY_SALARY_CAT_ID,
            checked_label("月薪猫", settings.pet_role == "monthly-salary-cat"),
        )
        .text(
            ROLE_BROOM_WITCH_ID,
            checked_label("琪琪", settings.pet_role == "broom-witch"),
        )
        .build()
        .map_err(|error| error.to_string())?;
    let menu = MenuBuilder::new(app)
        .text(OPEN_CHAT_ID, "打开聊天")
        .text(SHOW_STATUS_ID, "查看状态")
        .text(PAUSE_REMINDERS_ID, "提醒暂停到明天")
        .text(OPEN_SETTINGS_ID, "打开设置")
        .item(&role_menu)
        .separator()
        .text(EXIT_ID, "退出")
        .build()
        .map_err(|error| error.to_string())?;

    window
        .popup_menu_at(&menu, LogicalPosition::new(x, y))
        .map_err(|error| error.to_string())
}

pub fn handles_menu_id(id: &str) -> bool {
    matches!(
        id,
        OPEN_CHAT_ID
            | SHOW_STATUS_ID
            | PAUSE_REMINDERS_ID
            | OPEN_SETTINGS_ID
            | ROLE_GUGA_ID
            | ROLE_MONTHLY_SALARY_CAT_ID
            | ROLE_BROOM_WITCH_ID
            | EXIT_ID
    )
}

pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        OPEN_CHAT_ID => report("打开聊天窗口", windowing::show_chat_window(app)),
        SHOW_STATUS_ID => report("显示系统状态", windowing::show_info_window_now(app)),
        PAUSE_REMINDERS_ID => report("暂停提醒", reminder::pause_all_until_tomorrow(app)),
        OPEN_SETTINGS_ID => report("打开设置窗口", windowing::show_settings_window(app)),
        ROLE_GUGA_ID => update_role(app, "guga"),
        ROLE_MONTHLY_SALARY_CAT_ID => update_role(app, "monthly-salary-cat"),
        ROLE_BROOM_WITCH_ID => update_role(app, "broom-witch"),
        EXIT_ID => app.exit(0),
        _ => {}
    }
}

fn update_role(app: &AppHandle, role: &str) {
    let Some(settings_state) = app.try_state::<SettingsState>() else {
        eprintln!("无法切换角色: 找不到应用设置状态");
        return;
    };
    let settings = match settings_state.set_pet_role(role.to_string()) {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("无法保存角色设置: {error}");
            return;
        }
    };
    if let Err(error) = reminder::sync_from_settings(app) {
        eprintln!("无法同步提醒设置: {error}");
        return;
    }
    if let Err(error) = app.emit(SETTINGS_UPDATED_EVENT, &settings) {
        eprintln!("无法同步角色设置事件: {error}");
    }
}

fn report(action: &str, result: Result<(), String>) {
    if let Err(error) = result {
        eprintln!("无法{action}: {error}");
    }
}
