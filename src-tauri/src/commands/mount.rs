//! WebDAV 目录挂载命令
//!
//! 将远程 WebDAV 目录映射到本地文件系统：
//! - Windows: 使用 `net use` 映射为网络驱动器
//! - macOS: 使用 `osascript mount volume` 挂载到 /Volumes/

use crate::error::AppError;
use crate::webdav::types::{ConnectionProfile, MountMapping};
use crate::webdav::AppState;
use std::collections::HashMap;
use std::process::Command;
use tauri::State;
use tauri_plugin_store::StoreExt;

/// 挂载远程目录到本地
#[tauri::command]
pub async fn mount_directory(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    connection_id: String,
    remote_path: String,
    local_path: Option<String>,
) -> Result<String, AppError> {
    let profile = load_profile_by_id(&app, &connection_id)?;

    // 如果用户指定了本地路径，检查目标是否非空
    if let Some(ref lp) = local_path {
        if is_path_non_empty(lp) {
            return Err(AppError::WebDav(format!(
                "target directory is not empty: {}",
                lp
            )));
        }
    }

    let url = build_webdav_url(&profile.url, &remote_path);

    let local_path = platform_mount(&url, &profile.username, &profile.password, local_path.as_deref())?;

    // 记录活动挂载
    let key = format!("{}::{}", connection_id, remote_path);
    let mut mounts = state.active_mounts.lock().map_err(mutex_err)?;
    mounts.insert(key, local_path.clone());

    // 持久化挂载配置到 profile
    let mut profile = profile;
    if let Some(existing) = profile.mounts.iter_mut().find(|m| m.remote_path == remote_path) {
        existing.local_path = Some(local_path.clone());
    } else {
        profile.mounts.push(MountMapping {
            remote_path: remote_path.clone(),
            local_path: Some(local_path.clone()),
        });
    }
    save_profile(&app, &profile)?;

    Ok(local_path)
}

/// 卸载指定远程目录
#[tauri::command]
pub async fn unmount_directory(
    state: State<'_, AppState>,
    connection_id: String,
    remote_path: String,
) -> Result<(), AppError> {
    let key = format!("{}::{}", connection_id, remote_path);
    let local_path = {
        let mut mounts = state.active_mounts.lock().map_err(mutex_err)?;
        mounts.remove(&key)
    };

    if let Some(path) = local_path {
        platform_unmount(&path)?;
    }

    Ok(())
}

/// 删除挂载映射：卸载并从 profile 中移除
#[tauri::command]
pub async fn remove_mount(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    connection_id: String,
    remote_path: String,
) -> Result<(), AppError> {
    // 卸载
    let key = format!("{}::{}", connection_id, remote_path);
    let local_path = {
        let mut mounts = state.active_mounts.lock().map_err(mutex_err)?;
        mounts.remove(&key)
    };
    if let Some(path) = local_path {
        let _ = platform_unmount(&path);
    }

    // 从 profile 中移除
    let mut profile = load_profile_by_id(&app, &connection_id)?;
    profile.mounts.retain(|m| m.remote_path != remote_path);
    save_profile(&app, &profile)?;

    Ok(())
}

/// 更新挂载映射的本地路径
#[tauri::command]
pub async fn update_mount_local_path(
    app: tauri::AppHandle,
    connection_id: String,
    remote_path: String,
    local_path: String,
) -> Result<(), AppError> {
    let mut profile = load_profile_by_id(&app, &connection_id)?;
    if let Some(mount) = profile.mounts.iter_mut().find(|m| m.remote_path == remote_path) {
        mount.local_path = Some(local_path);
        save_profile(&app, &profile)?;
    }
    Ok(())
}

/// 卸载所有活动挂载（内部实现，供 confirm_exit 同步调用）
pub fn unmount_all_inner(state: &State<'_, AppState>) -> Result<(), AppError> {
    let mounts: HashMap<String, String> = {
        let mut m = state.active_mounts.lock().map_err(mutex_err)?;
        std::mem::take(&mut *m)
    };

    for (_key, local_path) in &mounts {
        if let Err(e) = platform_unmount(local_path) {
            log::warn!("Failed to unmount {}: {}", local_path, e);
        }
    }

    Ok(())
}

/// 卸载所有活动挂载
#[tauri::command]
pub async fn unmount_all(state: State<'_, AppState>) -> Result<(), AppError> {
    unmount_all_inner(&state)
}

/// 应用启动时自动重新挂载所有已保存的映射
#[tauri::command]
pub async fn auto_mount(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut profiles = load_all_profiles(&app)?;
    let mut any_changed = false;

    for profile in &mut profiles {
        if profile.mounts.is_empty() {
            continue;
        }
        for mapping in &mut profile.mounts {
            let url = build_webdav_url(&profile.url, &mapping.remote_path);
            match platform_mount(&url, &profile.username, &profile.password, None) {
                Ok(local_path) => {
                    let key = format!("{}::{}", profile.id, mapping.remote_path);
                    let mut mounts = state.active_mounts.lock().map_err(mutex_err)?;
                    mounts.insert(key, local_path.clone());
                    mapping.local_path = Some(local_path);
                    any_changed = true;
                }
                Err(e) => {
                    log::warn!(
                        "Auto-mount failed for {}{}: {}",
                        profile.url,
                        mapping.remote_path,
                        e
                    );
                }
            }
        }
    }

    if any_changed {
        let store = app
            .store("connections.json")
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        store.set(
            "profiles",
            serde_json::to_value(&profiles).map_err(|e| AppError::Serialization(e.to_string()))?,
        );
        let _ = store.save();
    }

    Ok(())
}

// ---- 辅助函数 ----

/// 拼接 WebDAV URL：base_url + remote_path（处理斜杠拼接）
fn build_webdav_url(base_url: &str, remote_path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = remote_path.trim_start_matches('/');
    format!("{}/{}", base, path)
}

fn load_profile_by_id(app: &tauri::AppHandle, id: &str) -> Result<ConnectionProfile, AppError> {
    let profiles = load_all_profiles(app)?;
    profiles
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::WebDav("Connection profile not found".into()))
}

fn load_all_profiles(app: &tauri::AppHandle) -> Result<Vec<ConnectionProfile>, AppError> {
    let store = app
        .store("connections.json")
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    let profiles: Vec<ConnectionProfile> = store
        .get("profiles")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    Ok(profiles)
}

fn save_profile(app: &tauri::AppHandle, profile: &ConnectionProfile) -> Result<(), AppError> {
    let store = app
        .store("connections.json")
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    let mut profiles: Vec<ConnectionProfile> = store
        .get("profiles")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    if let Some(idx) = profiles.iter().position(|p| p.id == profile.id) {
        profiles[idx] = profile.clone();
    }

    store.set(
        "profiles",
        serde_json::to_value(&profiles).map_err(|e| AppError::Serialization(e.to_string()))?,
    );
    store
        .save()
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    Ok(())
}

/// 检查路径是否存在且不为空（有子项）
fn is_path_non_empty(path: &str) -> bool {
    std::fs::read_dir(path).map_or(false, |mut d| d.next().is_some())
}

fn mutex_err(e: std::sync::PoisonError<std::sync::MutexGuard<'_, HashMap<String, String>>>) -> AppError {
    AppError::Io(std::io::Error::other(e.to_string()))
}

// ---- 平台相关 ----

#[cfg(target_os = "macos")]
fn platform_mount(url: &str, username: &str, password: &str, preferred_path: Option<&str>) -> Result<String, AppError> {
    // 如果用户指定了挂载点，先用 mount_webdav 尝试
    if let Some(mount_point) = preferred_path {
        // 确保挂载点目录存在
        std::fs::create_dir_all(mount_point)
            .map_err(|e| AppError::WebDav(format!("cannot create mount point {}: {}", mount_point, e)))?;

        // 先尝试用 mount_webdav（需要写入 Keychain 凭据）
        // 实际使用 osascript mount volume，系统会自动选择挂载点名称
        let script = format!(
            "mount volume \"{}\" as user name \"{}\" with password \"{}\"",
            url, username, password
        );
        let output = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| AppError::WebDav(format!("osascript failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::WebDav(format!("mount failed: {}", stderr.trim())));
        }

        // osascript mount volume 不支持指定挂载点，系统自行决定
        // 找到新挂载点后检查是否与用户期望的名称匹配
        // 返回实际挂载路径
        let after = list_volumes()?;
        // 取 basename 做简单匹配
        let preferred_name = mount_point.trim_end_matches('/').rsplit('/').next().unwrap_or("");
        let found = after.iter().find(|v| {
            let name = v.trim_end_matches('/').rsplit('/').next().unwrap_or("");
            name == preferred_name
        });
        return Ok(found.cloned().unwrap_or_else(|| mount_point.to_string()));
    }

    // 自动模式：比较 /Volumes/ 前后差异
    let before = list_volumes()?;

    let script = format!(
        "mount volume \"{}\" as user name \"{}\" with password \"{}\"",
        url, username, password
    );
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| AppError::WebDav(format!("osascript failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::WebDav(format!("mount failed: {}", stderr.trim())));
    }

    let after = list_volumes()?;
    let local_path = after
        .into_iter()
        .find(|v| !before.contains(v))
        .ok_or_else(|| AppError::WebDav("could not determine mount point".into()))?;

    Ok(local_path)
}

#[cfg(target_os = "macos")]
fn platform_unmount(local_path: &str) -> Result<(), AppError> {
    let output = Command::new("diskutil")
        .args(["unmount", local_path])
        .output()
        .map_err(|e| AppError::WebDav(format!("diskutil failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::WebDav(format!("unmount failed: {}", stderr.trim())));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn list_volumes() -> Result<Vec<String>, AppError> {
    let entries = std::fs::read_dir("/Volumes")
        .map_err(|e| AppError::Io(e))?;
    let mut names = Vec::new();
    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            names.push(format!("/Volumes/{}", name));
        }
    }
    Ok(names)
}

#[cfg(target_os = "windows")]
fn platform_mount(url: &str, username: &str, password: &str, preferred_path: Option<&str>) -> Result<String, AppError> {
    // 用户指定盘符时使用指定值，否则自动分配
    let drive = preferred_path.unwrap_or("*");

    let output = Command::new("net")
        .args([
            "use",
            drive,
            url,
            &format!("/user:{}", username),
            password,
            "/persistent:no",
        ])
        .output()
        .map_err(|e| AppError::WebDav(format!("net use failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::WebDav(format!("mount failed: {}", stderr.trim())));
    }

    // 如果用户指定了盘符，直接返回
    if preferred_path.is_some() {
        return Ok(drive.to_uppercase());
    }

    // 自动分配：解析 "驱动器 Z: 现在已连接到 ..." 或 "Drive Z: is now connected to ..."
    let stdout = String::from_utf8_lossy(&output.stdout);
    let drive_letter = stdout
        .lines()
        .find_map(|line| {
            let line = line.trim();
            if line.len() >= 2 && line.chars().nth(1) == Some(':') {
                let letter = line.chars().next().unwrap().to_ascii_uppercase();
                if letter.is_ascii_alphabetic() {
                    return Some(format!("{}:", letter));
                }
            }
            None
        })
        .ok_or_else(|| AppError::WebDav("could not parse drive letter from net use output".into()))?;

    Ok(drive_letter)
}

#[cfg(target_os = "windows")]
fn platform_unmount(local_path: &str) -> Result<(), AppError> {
    let output = Command::new("net")
        .args(["use", local_path, "/delete", "/yes"])
        .output()
        .map_err(|e| AppError::WebDav(format!("net use delete failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::WebDav(format!("unmount failed: {}", stderr.trim())));
    }
    Ok(())
}

// 非 Windows/macOS 平台的 stub
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_mount(_url: &str, _username: &str, _password: &str, _preferred_path: Option<&str>) -> Result<String, AppError> {
    Err(AppError::WebDav("mount is not supported on this platform".into()))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_unmount(_local_path: &str) -> Result<(), AppError> {
    Err(AppError::WebDav("mount is not supported on this platform".into()))
}
