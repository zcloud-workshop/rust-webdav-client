//! 本地 HTTP 流式代理服务器
//!
//! WKWebView 不支持自定义 URI scheme 的 `<video>` 播放，因此需要在 localhost 上启动本地 HTTP 代理。
//! Range 请求头部原样转发到 WebDAV 上游，响应体以 chunk 流式传输——不会将大文件完全缓冲在内存中。
//!
//! ## 为什么是独立线程 + 独立 tokio runtime？
//!
//! Tauri 主事件循环跑在主线程上。如果把 accept loop 放在 Tauri 的 runtime 里，
//! 大量并发连接可能阻塞 IPC 处理或窗口事件。独立线程 + 2 worker 线程轻量隔离。

use crate::webdav::SharedStreams;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub fn start_http_server(streams: SharedStreams) -> u16 {
    // localhost 不能换成 127.0.0.1：WKWebView 把 localhost 视为潜在可信源，
    // 对自动播放策略和媒体 API 更友好。端口 0 让操作系统分配空闲端口。
    let listener = std::net::TcpListener::bind("localhost:0")
        .expect("Failed to bind streaming server");
    let port = listener.local_addr().unwrap().port();
    eprintln!("[streaming] bound to localhost:{}", port);
    // tokio 要求 socket 在转换前已设非阻塞模式。set_nonblocking 对 TCP socket 不会失败。
    listener.set_nonblocking(true).ok();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_io()
            .enable_time()
            .build()
            .expect("Failed to create streaming runtime");

        rt.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener)
                .expect("Failed to convert listener");
            eprintln!("[streaming] listening on localhost:{}", port);

            loop {
                match listener.accept().await {
                    Ok((socket, _)) => {
                        let streams = streams.clone();
                        tokio::spawn(async move {
                            handle_connection(socket, streams).await;
                        });
                    }
                    Err(e) => {
                        eprintln!("[streaming] accept error: {}", e);
                    }
                }
            }
        });
    });

    port
}

async fn handle_connection(socket: tokio::net::TcpStream, streams: SharedStreams) {
    let (reader_half, mut writer) = tokio::io::split(socket);
    let mut reader = BufReader::new(reader_half);

    let mut request_line = String::new();
    match reader.read_line(&mut request_line).await {
        Ok(0) | Err(_) => return,
        _ => {}
    }

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let path = parts[1];

    let mut range_header: Option<String> = None;
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) | Err(_) => return,
            _ => {}
        }
        if line.trim().is_empty() {
            break;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("range:") {
            range_header = Some(line.trim()[6..].trim().to_string());
        }
    }

    if method != "GET" && method != "HEAD" {
        let _ = writer
            .write_all(b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n")
            .await;
        return;
    }

    // stream_id 来自 UUID v4（由 start_video_stream 命令生成），外部请求无法猜测
    let stream_id = path.trim_start_matches("/stream/");

    // 显式作用域确保锁在 HTTP 请求前释放。上游请求可能持续数分钟（大视频流式传输），
    // 持有锁会阻塞其他线程启动/停止流
    let stream_info = {
        let map = streams.lock().unwrap();
        match map.get(stream_id) {
            Some(s) => Some((
                s.webdav_path.clone(),
                s.base_url.clone(),
                s.auth_header.clone(),
                s.accept_insecure,
            )),
            None => None,
        }
    };

    let (webdav_path, base_url, auth_header, accept_insecure) = match stream_info {
        Some(info) => info,
        None => {
            let _ = writer
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nAccess-Control-Allow-Origin: *\r\n\r\n")
                .await;
            return;
        }
    };

    let url = format!("{}{}", base_url.trim_end_matches('/'), webdav_path);
    let reqwest_method = if method == "HEAD" {
        reqwest::Method::HEAD
    } else {
        reqwest::Method::GET
    };

    let client = if accept_insecure {
        reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    } else {
        reqwest::Client::new()
    };
    let mut req = client
        .request(reqwest_method, &url)
        .header("Authorization", &auth_header);

    if let Some(ref range) = range_header {
        req = req.header("Range", range.as_str());
    }

    match req.send().await {
        Ok(mut response) => {
            let status = response.status().as_u16();
            let resp_headers = response.headers().clone();

            // 很多 WebDAV 服务器不设置 Content-Type 或设为 application/octet-stream，
            // 而 WKWebView 需要识别为 video/* 才能播放。此时从文件扩展名回退检测 MIME。
            let content_type = resp_headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_else(|| mime_from_path(&webdav_path))
                .to_string();

            // Accept-Ranges: bytes 是必须的：浏览器依赖它判断是否支持拖动进度条。
            // 即使上游不支持 Range，我们仍然声明支持（浏览器会回退到全文件加载）。
            // CORS 头必须无条件发送，因为 video 元素加载的 localhost 源与 webview 源不同。
            let mut resp = format!(
                "HTTP/1.1 {}\r\nAccept-Ranges: bytes\r\nContent-Type: {}\r\n",
                status, content_type
            );

            if status == 206 {
                if let Some(cr) = resp_headers.get("content-range").and_then(|v| v.to_str().ok()) {
                    resp.push_str(&format!("Content-Range: {}\r\n", cr));
                }
            }

            if let Some(cl) = resp_headers.get("content-length").and_then(|v| v.to_str().ok()) {
                resp.push_str(&format!("Content-Length: {}\r\n", cl));
            }

            resp.push_str("Access-Control-Allow-Origin: *\r\n\r\n");

            eprintln!("[streaming] response:\n{}", resp);

            if writer.write_all(resp.as_bytes()).await.is_err() {
                return;
            }

            if method == "GET" {
                // 零缓冲流式传输：chunk 到达后立即转发，不累积整个响应体。
                // 多 GB 的视频文件内存占用恒定（~64KB，reqwest 默认 chunk 大小）。
                // 不能用 response.bytes().await —— 会把整个文件加载到内存。
                loop {
                    match response.chunk().await {
                        Ok(Some(chunk)) => {
                            if writer.write_all(&chunk).await.is_err() {
                                return;
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            eprintln!("[streaming] chunk error: {}", e);
                            return;
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("[streaming] upstream error: {}", e);
            let _ = writer
                .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n")
                .await;
        }
    }
}

fn mime_from_path(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".mp4") {
        "video/mp4"
    } else if lower.ends_with(".webm") {
        "video/webm"
    } else if lower.ends_with(".mov") {
        "video/quicktime"
    } else if lower.ends_with(".mkv") {
        "video/x-matroska"
    } else if lower.ends_with(".avi") {
        "video/x-msvideo"
    } else if lower.ends_with(".m4v") {
        "video/x-m4v"
    } else if lower.ends_with(".mp3") {
        "audio/mpeg"
    } else if lower.ends_with(".wav") {
        "audio/wav"
    } else if lower.ends_with(".flac") {
        "audio/flac"
    } else {
        "application/octet-stream"
    }
}
