//! 连接管理命令
//!
//! 处理 WebDAV 连接的建立、断开、测试和配置持久化

use crate::error::AppError;
use crate::webdav::client::WebDavClient;
use crate::webdav::types::ConnectionProfile;
use crate::webdav::AppState;
use tauri::State;
use tauri_plugin_store::StoreExt;

/// 建立 WebDAV 连接并设为活动连接
#[tauri::command]
pub async fn connect(state: State<'_, AppState>, profile: ConnectionProfile) -> Result<(), AppError> {
    let client = WebDavClient::new(&profile)?;
    // 将新连接加入连接池
    let mut conns = state.connections.lock().map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    conns.insert(profile.id.clone(), client);
    // 设为当前活动连接
    let mut active = state.active_connection_id.lock().map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    *active = Some(profile.id);
    Ok(())
}

/// 断开指定的 WebDAV 连接
#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>, profile_id: String) -> Result<(), AppError> {
    // 从连接池移除
    let mut conns = state.connections.lock().map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    conns.remove(&profile_id);
    // 如果是当前活动连接，清除活动状态
    let mut active = state.active_connection_id.lock().map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    if active.as_deref() == Some(&profile_id) {
        *active = None;
    }
    Ok(())
}

/// 测试连接配置是否有效
#[tauri::command]
pub async fn test_connection(profile: ConnectionProfile) -> Result<bool, AppError> {
    let client = WebDavClient::new(&profile)?;
    client.test().await
}

/// 使用指定连接配置列出远程根目录下的所有文件夹名称
///
/// 创建临时客户端，不依赖活动连接状态
#[tauri::command]
pub async fn list_remote_root_dirs(profile: ConnectionProfile) -> Result<Vec<String>, AppError> {
    let client = WebDavClient::new(&profile)?;
    let items = client.list_dir("/").await?;
    Ok(items.into_iter().filter(|i| i.is_dir).map(|i| i.name).collect())
}

/// 保存连接配置到本地存储
///
/// 如果 ID 已存在则更新，否则添加新配置
#[tauri::command]
pub async fn save_profile(
    app: tauri::AppHandle,
    profile: ConnectionProfile,
) -> Result<(), AppError> {
    let store = app.store("connections.json").map_err(|e| AppError::Serialization(e.to_string()))?;
    // 加载现有配置
    let mut profiles: Vec<ConnectionProfile> = store
        .get("profiles")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // 更新或添加配置
    if let Some(idx) = profiles.iter().position(|p| p.id == profile.id) {
        profiles[idx] = profile;
    } else {
        profiles.push(profile);
    }

    // 保存到磁盘
    store.set("profiles", serde_json::to_value(&profiles).map_err(|e| AppError::Serialization(e.to_string()))?);
    store.save().map_err(|e| AppError::Serialization(e.to_string()))?;
    Ok(())
}

/// 从本地存储加载所有连接配置
#[tauri::command]
pub async fn load_profiles(app: tauri::AppHandle) -> Result<Vec<ConnectionProfile>, AppError> {
    let store = app.store("connections.json").map_err(|e| AppError::Serialization(e.to_string()))?;
    let profiles: Vec<ConnectionProfile> = store
        .get("profiles")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    Ok(profiles)
}

/// 删除指定的连接配置
#[tauri::command]
pub async fn delete_profile(app: tauri::AppHandle, profile_id: String) -> Result<(), AppError> {
    let store = app.store("connections.json").map_err(|e| AppError::Serialization(e.to_string()))?;
    let mut profiles: Vec<ConnectionProfile> = store
        .get("profiles")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // 过滤掉要删除的配置
    profiles.retain(|p| p.id != profile_id);

    // 保存更改
    store.set("profiles", serde_json::to_value(&profiles).map_err(|e| AppError::Serialization(e.to_string()))?);
    store.save().map_err(|e| AppError::Serialization(e.to_string()))?;
    Ok(())
}
