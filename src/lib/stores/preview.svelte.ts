/**
 * 文件预览状态管理
 * 视频文件通过本地 HTTP 代理服务器实时流式播放。
 * 二进制数据通过 AbortController 支持取消在途请求。
 */

import type { FileCategory } from "../types";
import { api } from "../api";

let isOpen = $state(false);
let filePath = $state<string | null>(null);
let fileName = $state<string | null>(null);
let fileType = $state<FileCategory | null>(null);
let isLoading = $state(false);
let isEditing = $state(false);
let data = $state<ArrayBuffer | string | null>(null);
let error = $state<string | null>(null);
let abortController = $state<AbortController | null>(null);
let videoSrc = $state<string | null>(null);
let streamId = $state<string | null>(null);

export function getIsOpen() { return isOpen; }
export function getFilePath() { return filePath; }
export function getFileName() { return fileName; }
export function getFileType() { return fileType; }
export function getIsLoading() { return isLoading; }
export function getIsEditing() { return isEditing; }
export function getData() { return data; }
export function getError() { return error; }
export function getVideoSrc() { return videoSrc; }

// 防止将大文件完全加载到 ArrayBuffer 中导致渲染进程内存不足
const PREVIEW_SIZE_LIMIT = 50 * 1024 * 1024;

export async function openFile(
  path: string,
  name: string,
  category: FileCategory,
  size?: number | null,
) {
  // 终止上一个预览的请求，防止旧响应在新预览打开后到达导致状态错乱
  if (abortController) {
    abortController.abort();
  }
  abortController = new AbortController();

  // 清理上一个视频流（服务端 HashMap 条目），防止泄露
  if (streamId) {
    await api.preview.stopVideoStream(streamId);
    streamId = null;
    videoSrc = null;
  }

  if (category === "video") {
    filePath = path;
    fileName = name;
    fileType = category;
    isOpen = true;
    isLoading = true;
    isEditing = false;
    error = null;
    data = new ArrayBuffer(0);

    try {
      const url = await api.preview.startVideoStream(path);
      // URL 格式必须与 commands/preview.rs 中 start_video_stream 的格式一致：
      // http://localhost:PORT/stream/UUID
      streamId = url.split("/stream/").pop() || null;
      videoSrc = url;
    } catch (e) {
      console.error("Video stream error:", e);
      if (!(e instanceof Error && e.name === "AbortError")) {
        error = String(e);
      }
    } finally {
      isLoading = false;
      abortController = null;
    }
    return;
  }

  if (size && size > PREVIEW_SIZE_LIMIT) {
    filePath = path;
    fileName = name;
    fileType = category;
    isOpen = true;
    isLoading = false;
    isEditing = false;
    error = `File too large to preview (${(size / 1024 / 1024).toFixed(1)} MB). Please download to view locally.`;
    data = null;
    abortController = null;
    return;
  }

  filePath = path;
  fileName = name;
  fileType = category;
  isOpen = true;
  isLoading = true;
  isEditing = false;
  error = null;
  data = null;

  try {
    if (category === "text") {
      data = await api.edit.getTextContent(path, size ?? null, abortController.signal);
    } else {
      const result = await api.preview.getPreviewData(path, size ?? null, abortController.signal);
      // Tauri IPC 将 Vec<u8> 序列化为不同格式取决于传输层和版本：
      // Uint8Array（现代 Tauri v2）、ArrayBuffer（标准）、number[]（旧版）、
      // 或 JSON 对象 {0: 65, 1: 66, ...}（二进制传输失败时的回退）
      let bytes: Uint8Array;
      if (result instanceof Uint8Array) {
        bytes = result;
      } else if (result instanceof ArrayBuffer) {
        bytes = new Uint8Array(result);
      } else if (Array.isArray(result)) {
        bytes = new Uint8Array(result);
      } else if (typeof result === "object" && result !== null) {
        bytes = new Uint8Array(Object.values(result) as number[]);
      } else {
        throw new Error("Unsupported data type received");
      }
      data = bytes.buffer as ArrayBuffer;
    }
  } catch (e) {
    console.error("Preview error:", e);
    // AbortError 是用户切换文件时的正常取消，不是真正的错误，不应展示给用户
    if (!(e instanceof Error && e.name === "AbortError")) {
      error = String(e);
    }
  } finally {
    isLoading = false;
    abortController = null;
  }
}

export async function saveContent(content: string) {
  if (!filePath) return;
  isLoading = true;
  error = null;
  try {
    await api.edit.saveTextContent(filePath, content);
    data = content;
    isEditing = false;
  } catch (e) {
    error = String(e);
  } finally {
    isLoading = false;
  }
}

export function setEditing(editing: boolean) {
  isEditing = editing;
}

export async function close() {
  if (abortController) {
    abortController.abort();
    abortController = null;
  }
  if (streamId) {
    await api.preview.stopVideoStream(streamId);
    streamId = null;
    videoSrc = null;
  }
  isOpen = false;
  filePath = null;
  fileName = null;
  fileType = null;
  data = null;
  error = null;
  isEditing = false;
}
