/**
 * 全局类型定义
 */

/** WebDAV 连接配置 */
export interface ConnectionProfile {
  /** 唯一标识符 */
  id: string;
  /** 用户自定义名称 */
  name: string;
  /** WebDAV 服务器 URL */
  url: string;
  /** 用户名 */
  username: string;
  /** 密码 */
  password: string;
  /** 是否允许不安全的 SSL 证书 */
  accept_insecure: boolean;
}

/** 文件/文件夹元数据 */
export interface FileMetadata {
  /** 文件名称 */
  name: string;
  /** 完整路径 */
  path: string;
  /** 是否为目录 */
  is_dir: boolean;
  /** 文件大小（字节），目录为 null */
  size: number | null;
  /** 最后修改时间（ISO 格式） */
  modified: string | null;
  /** MIME 内容类型 */
  content_type: string | null;
}

/** 文件分类类型 */
export type FileCategory =
  | "text"
  | "image"
  | "pdf"
  | "audio"
  | "video"
  | "docx"
  | "xlsx"
  | "pptx"
  | "archive"
  | "unknown";

/** 文件视图模式 */
export type ViewMode = "list" | "grid";
/** 排序字段 */
export type SortBy = "name" | "size" | "modified";
/** 排序顺序 */
export type SortOrder = "asc" | "desc";
