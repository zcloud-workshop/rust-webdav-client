//! WebDAV 桌面客户端 Tauri 应用库
//!
//! ## 退出确认协议
//!
//! 退出流程涉及两层事件和三个阶段：
//!
//! 1. **窗口关闭按钮 / Cmd+W** → `WindowEvent::CloseRequested`
//! 2. **Cmd+Q / Dock 退出** → 自定义菜单项 (避免系统默认直接退出) 或 `RunEvent::ExitRequested`
//!
//! 两种路径都发射 `close-requested` 到前端，前端弹出确认对话框。
//! 用户确认后调用 `confirm_exit`，设置 `ExitConfirmed` 标志位后调用 `app.exit(0)`。
//!
//! `app.exit(0)` 会再次触发 `ExitRequested`，此时检测到标志位为 true 则放行。
//! **标志位必须在 `exit()` 之前设置**，否则 `ExitRequested` 再次拦截 → 无限循环。
//!
//! 自定义菜单项替代了系统默认 Quit（后者直接调用 `NSApplication.terminate:` 会跳过确认）。

mod commands;
mod error;
mod streaming;
mod webdav;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    Emitter, Manager, RunEvent,
};
use webdav::AppState;

struct ExitConfirmed(AtomicBool);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::default();
    // 流媒体端口必须在 Tauri 启动前获取并存储，否则首帧视频请求会拿到端口 0（未初始化的默认值）
    let port = streaming::start_http_server(state.stream_paths.clone());
    *state.streaming_port.lock().unwrap() = port;

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .manage(ExitConfirmed(AtomicBool::new(false)))
        .invoke_handler(tauri::generate_handler![
            commands::connection::connect,
            commands::connection::disconnect,
            commands::connection::test_connection,
            commands::connection::save_profile,
            commands::connection::load_profiles,
            commands::connection::delete_profile,
            commands::files::list_directory,
            commands::download::download_file,
            commands::download::download_file_to,
            commands::upload::upload_file,
            commands::upload::upload_local_file,
            commands::operations::create_folder,
            commands::operations::delete_item,
            commands::operations::rename_item,
            commands::operations::move_item,
            commands::operations::copy_item,
            commands::preview::get_preview_data,
            commands::preview::start_video_stream,
            commands::preview::stop_video_stream,
            commands::edit::get_text_content,
            commands::edit::save_text_content,
            commands::app::confirm_exit,
            commands::app::get_system_locale,
        ])
        .setup(|app| {
            // 用自定义 Quit 菜单项替代系统默认，让 Cmd+Q 走 close-requested → 确认对话框流程
            let quit = MenuItemBuilder::with_id("custom_quit", "Quit WebDAV Client")
                .accelerator("CmdOrCtrl+Q")
                .build(app)?;

            let app_menu = SubmenuBuilder::new(app, "WebDAV Client")
                .item(&quit)
                .separator()
                .item(&PredefinedMenuItem::services(app, None)?) // macOS Services 菜单集成
                .separator()
                .item(&PredefinedMenuItem::hide(app, None)?)
                .item(&PredefinedMenuItem::hide_others(app, None)?)
                .item(&PredefinedMenuItem::show_all(app, None)?)
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .item(&PredefinedMenuItem::undo(app, None)?)
                .item(&PredefinedMenuItem::redo(app, None)?)
                .separator()
                .item(&PredefinedMenuItem::cut(app, None)?)
                .item(&PredefinedMenuItem::copy(app, None)?)
                .item(&PredefinedMenuItem::paste(app, None)?)
                .item(&PredefinedMenuItem::select_all(app, None)?)
                .build()?;

            let window_menu = SubmenuBuilder::new(app, "Window")
                .item(&PredefinedMenuItem::minimize(app, None)?)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&app_menu)
                .item(&edit_menu)
                .item(&window_menu)
                .build()?;

            app.set_menu(menu)?;

            let handle = app.handle().clone();
            app.on_menu_event(move |_app, event| {
                if event.id() == "custom_quit" {
                    if let Some(window) = handle.get_webview_window("main") {
                        let _ = window.emit("close-requested", ());
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // CloseRequested：窗口关闭按钮、Cmd+W。必须拦截 -> 弹确认框
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.emit("close-requested", ());
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // ExitRequested：Cmd+Q（经菜单）、Dock 右键退出、app.exit()。
            // 如果 ExitConfirmed 已设置（来自 confirm_exit），放行；否则拦截 -> 弹确认框
            if let RunEvent::ExitRequested { api, .. } = event {
                let confirmed = app.state::<ExitConfirmed>();
                if confirmed.0.load(Ordering::Relaxed) {
                    return; // 用户已确认，允许退出
                }
                api.prevent_exit();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("close-requested", ());
                }
            }
        });
}
