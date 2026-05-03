//! 本地 HTTP 流式代理服务器
//!
//! 在 localhost 上启动，将视频 Range 请求流式代理到 WebDAV 服务器。
//! WKWebView 的 video 元素不支持自定义 URI scheme，但完全支持 http://localhost。

use crate::webdav::SharedStreams;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// 启动 HTTP 流式代理服务器，返回绑定的端口号
pub fn start_http_server(streams: SharedStreams) -> u16 {
    let listener = std::net::TcpListener::bind("localhost:0")
        .expect("Failed to bind streaming server");
    let port = listener.local_addr().unwrap().port();
    eprintln!("[streaming] bound to localhost:{}", port);
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

    let stream_id = path.trim_start_matches("/stream/");

    let stream_info = {
        let map = streams.lock().unwrap();
        match map.get(stream_id) {
            Some(s) => Some((
                s.webdav_path.clone(),
                s.base_url.clone(),
                s.auth_header.clone(),
            )),
            None => None,
        }
    };

    let (webdav_path, base_url, auth_header) = match stream_info {
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

    let client = reqwest::Client::new();
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

            let content_type = resp_headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_else(|| mime_from_path(&webdav_path))
                .to_string();

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
