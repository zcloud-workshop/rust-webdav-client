use crate::ExitConfirmed;
use std::sync::atomic::Ordering;
use tauri::Manager;

#[tauri::command]
pub async fn confirm_exit(app: tauri::AppHandle) {
    // 标志位必须在 exit() 之前设置。app.exit(0) 会同步触发 ExitRequested，
    // 如果此时标志位仍为 false，ExitRequested 会再次 prevent_exit() → 无限循环。
    app.state::<ExitConfirmed>().0.store(true, Ordering::Relaxed);
    app.exit(0);
}

#[tauri::command]
pub fn get_system_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| String::from("en"))
}
