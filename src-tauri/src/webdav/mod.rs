//! WebDAV 模块 - 核心业务逻辑

pub mod client;
pub mod types;

use client::WebDavClient;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 共享流状态类型（HTTP 服务器和 Tauri commands 共享）
pub type SharedStreams = Arc<Mutex<HashMap<String, StreamState>>>;

/// 流式播放状态
pub struct StreamState {
    pub webdav_path: String,
    pub base_url: String,
    pub auth_header: String,
}

/// 应用全局状态
pub struct AppState {
    pub connections: Mutex<HashMap<String, WebDavClient>>,
    pub active_connection_id: Mutex<Option<String>>,
    pub stream_paths: SharedStreams,
    pub streaming_port: Mutex<u16>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
            active_connection_id: Mutex::new(None),
            stream_paths: Arc::new(Mutex::new(HashMap::new())),
            streaming_port: Mutex::new(0),
        }
    }
}

impl AppState {
    pub fn get_client(&self) -> Result<WebDavClient, crate::error::AppError> {
        let active_id = self
            .active_connection_id
            .lock()
            .map_err(|e| crate::error::AppError::Io(std::io::Error::other(e.to_string())))?
            .clone();

        let id = active_id.ok_or(crate::error::AppError::NotConnected)?;

        let conns = self
            .connections
            .lock()
            .map_err(|e| crate::error::AppError::Io(std::io::Error::other(e.to_string())))?;

        let client = conns.get(&id).ok_or(crate::error::AppError::NotConnected)?;
        Ok(client.clone())
    }
}
