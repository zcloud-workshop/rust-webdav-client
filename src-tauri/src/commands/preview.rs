//! 文件预览命令

use crate::error::AppError;
use crate::webdav::{AppState, StreamState};
use tauri::State;

#[tauri::command]
pub async fn get_preview_data(
    state: State<'_, AppState>,
    path: String,
    _size: Option<u64>,
) -> Result<Vec<u8>, AppError> {
    let client = state.get_client()?;
    client.download(&path).await
}

#[tauri::command]
pub async fn start_video_stream(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, AppError> {
    let client = state.get_client()?;
    let id = uuid::Uuid::new_v4().to_string();
    let port = *state.streaming_port.lock().unwrap();

    state
        .stream_paths
        .lock()
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
        .insert(
            id.clone(),
            StreamState {
                webdav_path: path,
                base_url: client.base_url().to_string(),
                auth_header: client.auth_header(),
            },
        );

    let url = format!("http://localhost:{}/stream/{}", port, id);
    eprintln!("[preview] video stream URL: {}", url);
    Ok(url)
}

#[tauri::command]
pub async fn stop_video_stream(
    state: State<'_, AppState>,
    stream_id: String,
) -> Result<(), AppError> {
    state
        .stream_paths
        .lock()
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
        .remove(&stream_id);
    Ok(())
}
