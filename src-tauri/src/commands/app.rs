use crate::ExitConfirmed;
use crate::webdav::AppState;
use std::sync::atomic::Ordering;
use tauri::Manager;

#[tauri::command]
pub async fn confirm_exit(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), ()> {
    // 退出前卸载所有挂载
    if let Err(e) = crate::commands::mount::unmount_all_inner(&state) {
        log::warn!("Failed to unmount all during exit: {}", e);
    }

    // 标志位必须在 exit() 之前设置。app.exit(0) 会同步触发 ExitRequested，
    // 如果此时标志位仍为 false，ExitRequested 会再次 prevent_exit() → 无限循环。
    app.state::<ExitConfirmed>().0.store(true, Ordering::Relaxed);
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn get_system_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| String::from("en"))
}
