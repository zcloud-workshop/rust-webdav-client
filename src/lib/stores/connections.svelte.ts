/**
 * WebDAV 连接状态管理
 * 使用 Svelte 5 Runes 实现响应式状态
 */

import type { ConnectionProfile } from "../types";
import { api } from "../api";

/** 所有已保存的连接配置列表 */
let profiles = $state<ConnectionProfile[]>([]);
/** 当前活动连接的 ID */
let activeId = $state<string | null>(null);
/** 加载状态 */
let loading = $state(false);
/** 错误信息 */
let error = $state<string | null>(null);

/** 获取所有连接配置 */
export function getProfiles() {
  return profiles;
}
/** 获取当前活动连接 ID */
export function getActiveId() {
  return activeId;
}
/** 获取当前活动连接配置 */
export function getActiveProfile(): ConnectionProfile | undefined {
  return profiles.find((p) => p.id === activeId);
}
/** 获取加载状态 */
export function getLoading() {
  return loading;
}
/** 获取错误信息 */
export function getError() {
  return error;
}
/** 检查是否已连接 */
export function isConnected() {
  return activeId !== null;
}

/** 从本地存储加载所有连接配置 */
export async function loadProfiles() {
  loading = true;
  error = null;
  try {
    profiles = await api.connection.loadProfiles();
  } catch (e) {
    error = String(e);
  } finally {
    loading = false;
  }
}

/** 保存或更新连接配置 */
export async function saveProfile(profile: ConnectionProfile) {
  await api.connection.saveProfile(profile);
  await loadProfiles();
}

/** 删除连接配置 */
export async function deleteProfile(id: string) {
  // 先断开再删除，防止 activeId 指向已删除的 profile、后端遗留活跃 client
  if (id === activeId) {
    await disconnect();
  }
  await api.connection.deleteProfile(id);
  await loadProfiles();
}

/** 建立 WebDAV 连接 */
export async function connect(id: string) {
  const profile = profiles.find((p) => p.id === id);
  if (!profile) throw new Error("Profile not found");
  loading = true;
  error = null;
  try {
    await api.connection.connect(profile);
    activeId = id;
  } catch (e) {
    error = String(e);
    throw e;
  } finally {
    loading = false;
  }
}

/** 断开当前 WebDAV 连接 */
export async function disconnect() {
  if (activeId) {
    try {
      await api.connection.disconnect(activeId);
    } catch {
      // 忽略断开连接时的网络错误（服务器可能已不可达）。
      // 无论后端断开是否成功，前端都清空 activeId，避免 UI 卡在"已连接但不可达"状态。
    }
  }
  activeId = null;
}

/** 测试连接配置是否有效 */
export async function testConnection(profile: ConnectionProfile): Promise<boolean> {
  return api.connection.testConnection(profile);
}
