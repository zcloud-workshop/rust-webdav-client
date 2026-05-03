/**
 * Tauri IPC API 封装
 * 将所有后端命令封装为前端友好的调用接口
 */

import { invoke } from "@tauri-apps/api/core";
import type { ConnectionProfile, FileMetadata } from "./types";

export const api = {
  /** 连接管理相关 API */
  connection: {
    /** 建立 WebDAV 连接 */
    connect: (profile: ConnectionProfile) => invoke("connect", { profile }),
    /** 断开 WebDAV 连接 */
    disconnect: (profileId: string) => invoke("disconnect", { profileId }),
    /** 测试连接配置 */
    testConnection: (profile: ConnectionProfile) =>
      invoke<boolean>("test_connection", { profile }),
    /** 保存连接配置到本地 */
    saveProfile: (profile: ConnectionProfile) =>
      invoke("save_profile", { profile }),
    /** 加载所有已保存的连接配置 */
    loadProfiles: () => invoke<ConnectionProfile[]>("load_profiles"),
    /** 删除连接配置 */
    deleteProfile: (profileId: string) =>
      invoke("delete_profile", { profileId }),
  },

  /** 文件浏览相关 API */
  files: {
    /** 列出指定目录的文件 */
    listDirectory: (path: string) =>
      invoke<FileMetadata[]>("list_directory", { path }),
  },

  /** 文件下载相关 API */
  download: {
    /** 下载文件为 ArrayBuffer */
    downloadFile: (path: string) =>
      invoke<ArrayBuffer>("download_file", { path }),
    /** 下载文件并保存到本地路径 */
    downloadFileTo: (path: string, localPath: string) =>
      invoke("download_file_to", { path, localPath }),
  },

  /** 文件上传相关 API */
  upload: {
    /** 上传数据到 WebDAV 服务器 */
    uploadFile: (remotePath: string, data: number[]) =>
      invoke("upload_file", { remotePath, data }),
    /** 从本地文件直接上传 */
    uploadLocalFile: (remotePath: string, localPath: string) =>
      invoke("upload_local_file", { remotePath, localPath }),
  },

  /** 文件操作相关 API */
  operations: {
    /** 创建文件夹 */
    createFolder: (path: string) => invoke("create_folder", { path }),
    /** 删除文件或文件夹 */
    deleteItem: (path: string) => invoke("delete_item", { path }),
    /** 重命名文件或文件夹 */
    renameItem: (from: string, to: string) =>
      invoke("rename_item", { from, to }),
    /** 移动文件或文件夹 */
    moveItem: (from: string, to: string) =>
      invoke("move_item", { from, to }),
    /** 复制文件或文件夹 */
    copyItem: (from: string, to: string) =>
      invoke("copy_item", { from, to }),
  },

  /** 文件预览相关 API */
  preview: {
    /** 获取文件预览数据，支持取消请求 */
    getPreviewData: (path: string, size: number | null, signal?: AbortSignal) =>
      invoke<ArrayBuffer | number[]>("get_preview_data", { path, size }, { signal }),
    /** 启动视频流，返回 HTTP URL */
    startVideoStream: (path: string) =>
      invoke<string>("start_video_stream", { path }),
    /** 停止视频流 */
    stopVideoStream: (streamId: string) =>
      invoke("stop_video_stream", { streamId }),
  },

  /** 文本编辑相关 API */
  edit: {
    /** 获取文件文本内容，支持取消请求 */
    getTextContent: (path: string, size: number | null, signal?: AbortSignal) =>
      invoke<string>("get_text_content", { path, size }, { signal }),
    /** 保存文本内容到文件 */
    saveTextContent: (path: string, content: string) =>
      invoke("save_text_content", { path, content }),
  },
};
