//! WebDAV 数据类型定义

use serde::{Deserialize, Serialize};

/// WebDAV 连接配置文件
///
/// 用于持久化存储用户的连接信息
#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    /// 唯一标识符
    pub id: String,
    /// 用户可见的连接名称
    pub name: String,
    /// WebDAV 服务器 URL
    pub url: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 是否接受不安全的 SSL 证书（自签名等）
    #[serde(default)]
    pub accept_insecure: bool,
    /// 根目录下要隐藏的目录名称列表
    #[serde(default)]
    pub hidden_root_dirs: Vec<String>,
    /// 已保存的目录挂载映射
    #[serde(default)]
    pub mounts: Vec<MountMapping>,
}

/// 目录挂载映射配置
#[derive(Clone, Serialize, Deserialize)]
pub struct MountMapping {
    /// 远程 WebDAV 路径（如 "/documents"）
    pub remote_path: String,
    /// 上次挂载的本地路径（如 "Z:" 或 "/Volumes/documents"）
    #[serde(default)]
    pub local_path: Option<String>,
}

/// 文件元数据
///
/// 表示 WebDAV 服务器上的文件或文件夹信息
/// 仅用于从 Rust 序列化到前端
#[derive(Clone, Serialize)]
pub struct FileMetadata {
    /// 文件/文件夹名称
    pub name: String,
    /// 完整路径
    pub path: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 文件大小（字节），目录为 None
    pub size: Option<u64>,
    /// 最后修改时间（RFC3339 格式）
    pub modified: Option<String>,
    /// MIME 内容类型
    pub content_type: Option<String>,
}
