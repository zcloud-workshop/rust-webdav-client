//! WebDAV 桌面客户端 Tauri 应用库

mod commands;
mod error;
mod streaming;
mod webdav;

use tauri::{Emitter, Manager, RunEvent};
use webdav::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::default();
    let port = streaming::start_http_server(state.stream_paths.clone());
    *state.streaming_port.lock().unwrap() = port;

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
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
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.emit("close-requested", ());
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("close-requested", ());
                }
            }
        });
}
