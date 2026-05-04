//! WebDAV 客户端实现
//!
//! 封装 reqwest_dav 库，提供所有 WebDAV 文件操作功能

use crate::error::AppError;
use crate::webdav::types::{ConnectionProfile, FileMetadata};
use reqwest_dav::list_cmd::ListEntity;
use reqwest_dav::{Auth, ClientBuilder, Depth};
use std::time::Duration;
use tokio::time::timeout;

/// WebDAV 客户端包装器
///
/// 封装 reqwest_dav::Client，提供路径规范化和超时保护
#[derive(Clone)]
pub struct WebDavClient {
    client: reqwest_dav::Client,
    base_url: String,
    username: String,
    password: String,
    accept_insecure: bool,
}

impl WebDavClient {
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn accept_insecure(&self) -> bool {
        self.accept_insecure
    }

    // 供流媒体代理使用。reqwest_dav::Client 内部已处理 auth，
    // 但代理服务器需要原始 Authorization 头部来转发请求到上游。
    pub fn auth_header(&self) -> String {
        use base64::Engine;
        let credentials = format!("{}:{}", self.username, self.password);
        let encoded =
            base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        format!("Basic {}", encoded)
    }

    /// 创建新的 WebDAV 客户端
    ///
    /// 使用连接配置初始化 HTTP 客户端并设置 Basic Auth
    pub fn new(profile: &ConnectionProfile) -> Result<Self, AppError> {
        let mut builder = ClientBuilder::new()
            .set_host(profile.url.clone())
            .set_auth(Auth::Basic(
                profile.username.clone(),
                profile.password.clone(),
            ));

        if profile.accept_insecure {
            let agent = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(|e| AppError::WebDav(e.to_string()))?;
            builder = builder.set_agent(agent);
        }

        let client = builder.build()?;

        Ok(Self {
            client,
            base_url: profile.url.clone(),
            username: profile.username.clone(),
            password: profile.password.clone(),
            accept_insecure: profile.accept_insecure,
        })
    }

    /// 测试连接是否有效
    ///
    /// 通过列出根目录验证连接，超时时间 10 秒
    pub async fn test(&self) -> Result<bool, AppError> {
        match timeout(Duration::from_secs(10), self.client.list("/", Depth::Number(0))).await {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(e)) => {
                log::warn!("Connection test failed: {}", e);
                Ok(false)
            }
            Err(_) => {
                log::warn!("Connection test timeout");
                Ok(false)
            }
        }
    }

    /// 列出目录内容
    ///
    /// 返回指定路径下的所有文件和文件夹元数据
    pub async fn list_dir(&self, path: &str) -> Result<Vec<FileMetadata>, AppError> {
        let normalized_path = Self::normalize_path(path);
        log::debug!("Listing directory: {} (normalized: {})", path, normalized_path);
        let items = timeout(Duration::from_secs(30), self.client.list(&normalized_path, Depth::Number(1)))
            .await
            .map_err(|_| AppError::WebDav("Request timeout".to_string()))??;
        let base_path = normalized_path.trim_end_matches('/');

        let mut result = Vec::new();
        for entity in items {
            match entity {
                ListEntity::Folder(folder) => {
                    let href_path = folder.href.trim_end_matches('/');
                    log::debug!("Folder href: {}", folder.href);
                    // 跳过目录自身（PROPFIND Depth:1 返回的第一条）。
                    // 依赖字符串相等：如果服务器返回的路径格式（编码/斜杠/大小写）
                    // 与客户端规范化的路径不同，此比较会失败，目录条目会重复出现。
                    if href_path == base_path {
                        continue;
                    }
                    let name = href_path.split('/').last().unwrap_or("").to_string();
                    result.push(FileMetadata {
                        name: urldecode(&name),
                        path: folder.href.clone(),
                        is_dir: true,
                        size: None,
                        modified: Some(folder.last_modified.to_rfc3339()),
                        content_type: None,
                    });
                }
                ListEntity::File(file) => {
                    log::debug!("File href: {}", file.href);
                    let name = file.href.trim_end_matches('/').split('/').last().unwrap_or("").to_string();
                    result.push(FileMetadata {
                        name: urldecode(&name),
                        path: file.href.clone(),
                        is_dir: false,
                        size: Some(file.content_length as u64),
                        modified: Some(file.last_modified.to_rfc3339()),
                        content_type: Some(file.content_type),
                    });
                }
            }
        }

        Ok(result)
    }

    /// 规范化路径
    ///
    /// 确保路径以 / 开头，空路径视为根目录 /
    fn normalize_path(path: &str) -> String {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            "/".to_string()
        } else if !trimmed.starts_with('/') {
            format!("/{}", trimmed)
        } else {
            trimmed.to_string()
        }
    }

    /// 下载文件为字节数组
    ///
    /// 两阶段超时：30 秒建立连接 + 接收响应头，60 秒传输正文。
    /// 正文超时在响应头到达后重新开始计时——慢速服务器不会因为连接阶段的耗时而被截断。
    pub async fn download(&self, path: &str) -> Result<Vec<u8>, AppError> {
        let normalized_path = Self::normalize_path(path);
        log::debug!("Downloading from: {}", normalized_path);
        let response = timeout(Duration::from_secs(30), self.client.get(&normalized_path))
            .await
            .map_err(|_| AppError::WebDav("Request timeout".to_string()))??;
        let bytes = timeout(Duration::from_secs(60), response.bytes())
            .await
            .map_err(|_| AppError::WebDav("Download timeout".to_string()))?
            .map_err(|e| AppError::WebDav(e.to_string()))?;
        Ok(bytes.to_vec())
    }

    /// 上传文件
    ///
    /// 超时时间 300 秒（5 分钟）用于大文件上传
    pub async fn upload(&self, path: &str, data: Vec<u8>) -> Result<(), AppError> {
        let normalized_path = Self::normalize_path(path);
        log::debug!("Uploading to normalized path: {}", normalized_path);
        timeout(Duration::from_secs(300), self.client.put(&normalized_path, data))
            .await
            .map_err(|_| AppError::WebDav("Upload timeout".to_string()))??;
        Ok(())
    }

    /// 删除文件或文件夹
    pub async fn delete(&self, path: &str) -> Result<(), AppError> {
        let normalized_path = Self::normalize_path(path);
        log::debug!("Deleting normalized path: {}", normalized_path);
        timeout(Duration::from_secs(30), self.client.delete(&normalized_path))
            .await
            .map_err(|_| AppError::WebDav("Request timeout".to_string()))??;
        Ok(())
    }

    /// 创建文件夹
    pub async fn create_folder(&self, path: &str) -> Result<(), AppError> {
        let normalized_path = Self::normalize_path(path);
        log::debug!("Creating folder normalized path: {}", normalized_path);
        timeout(Duration::from_secs(10), self.client.mkcol(&normalized_path))
            .await
            .map_err(|_| AppError::WebDav("Request timeout".to_string()))??;
        Ok(())
    }

    /// 重命名文件或文件夹
    ///
    /// 使用 WebDAV MOVE 方法实现
    pub async fn rename(&self, from: &str, to: &str) -> Result<(), AppError> {
        let from_normalized = Self::normalize_path(from);
        let to_normalized = Self::normalize_path(to);
        log::debug!("Rename: {} -> {}", from_normalized, to_normalized);
        timeout(Duration::from_secs(30), self.client.mv(&from_normalized, &to_normalized))
            .await
            .map_err(|_| AppError::WebDav("Request timeout".to_string()))??;
        Ok(())
    }

    /// 复制文件或文件夹
    ///
    /// 使用 WebDAV COPY 方法实现
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), AppError> {
        let from_normalized = Self::normalize_path(from);
        let to_normalized = Self::normalize_path(to);
        log::debug!("Copy: {} -> {}", from_normalized, to_normalized);
        timeout(Duration::from_secs(60), self.client.cp(&from_normalized, &to_normalized))
            .await
            .map_err(|_| AppError::WebDav("Request timeout".to_string()))??;
        Ok(())
    }

    /// 获取文件文本内容
    pub async fn get_text(&self, path: &str) -> Result<String, AppError> {
        let normalized_path = Self::normalize_path(path);
        log::debug!("Get text from: {}", normalized_path);
        let response = timeout(Duration::from_secs(30), self.client.get(&normalized_path))
            .await
            .map_err(|_| AppError::WebDav("Request timeout".to_string()))??;
        let text = timeout(Duration::from_secs(30), response.text())
            .await
            .map_err(|_| AppError::WebDav("Download timeout".to_string()))?
            .map_err(|e| AppError::WebDav(e.to_string()))?;
        Ok(text)
    }

    /// 写入文本内容到文件
    pub async fn put_text(&self, path: &str, content: String) -> Result<(), AppError> {
        let normalized_path = Self::normalize_path(path);
        log::debug!("Put text to: {}", normalized_path);
        self.client.put(&normalized_path, content.into_bytes()).await?;
        Ok(())
    }
}

/// URL 解码
///
/// 使用 application/x-www-form-urlencoded 约定（+ → 空格），不是标准 RFC 3986 的百分号解码。
/// 部分 WebDAV 服务器（Apache mod_dav 等）在 href 中采用此编码。
fn urldecode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let mut iter = s.bytes();
    while let Some(b) = iter.next() {
        if b == b'%' {
            let hi = iter.next().unwrap_or(b'0');
            let lo = iter.next().unwrap_or(b'0');
            bytes.push(hex_val(hi) << 4 | hex_val(lo));
        } else if b == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// 十六进制字符转数值
fn hex_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}
