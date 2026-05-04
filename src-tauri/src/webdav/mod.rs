//! WebDAV 模块 - 核心业务逻辑
//!
//! ## 锁顺序
//!
//! 当需要同时获取 `active_connection_id` 和 `connections` 锁时，必须按以下顺序：
//! `active_connection_id` → `connections`。`get_client()`、`connect()`、`disconnect()`
//! 都遵循此顺序。违反会导致死锁。

pub mod client;
pub mod types;

use client::WebDavClient;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type SharedStreams = Arc<Mutex<HashMap<String, StreamState>>>;

pub struct StreamState {
    pub webdav_path: String,
    pub base_url: String,
    pub auth_header: String,
    pub accept_insecure: bool,
}

pub struct AppState {
    pub connections: Mutex<HashMap<String, WebDavClient>>,
    pub active_connection_id: Mutex<Option<String>>,
    pub stream_paths: SharedStreams,
    // 0 = 未初始化（sentinel），由 lib.rs 在 Tauri build 前设置
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
    // 克隆 WebDavClient 而非返回引用：调用者是 Tauri async command，
    // 需要拥有值以便跨 await 使用。WebDavClient 内部是 Arc + 少量 String，克隆开销低。
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
