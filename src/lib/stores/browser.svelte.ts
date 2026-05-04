/**
 * 文件浏览器状态管理
 * 处理目录导航、文件选择、排序和视图模式
 */

import type { FileMetadata, SortBy, SortOrder, ViewMode } from "../types";
import { api } from "../api";
import { showToast } from "./toast.svelte";

/** 当前浏览的目录路径 */
let currentPath = $state("/");
/** 当前目录下的文件和文件夹列表 */
let items = $state<FileMetadata[]>([]);
/** 已选中的文件路径集合 */
let selectedPaths = $state<Set<string>>(new Set());
/** 加载状态 */
let loading = $state(false);
/** 错误信息 */
let error = $state<string | null>(null);
/** 排序字段 */
let sortBy = $state<SortBy>("name");
/** 排序顺序 */
let sortOrder = $state<SortOrder>("asc");
/** 视图模式（列表/网格） */
let viewMode = $state<ViewMode>("list");

/** 获取当前目录路径 */
export function getCurrentPath() {
  return currentPath;
}
/** 获取当前目录的文件列表 */
export function getItems() {
  return items;
}
/** 获取已选中的路径集合 */
export function getSelectedPaths() {
  return selectedPaths;
}
/** 获取加载状态 */
export function getLoading() {
  return loading;
}
/** 获取错误信息 */
export function getError() {
  return error;
}
/** 获取排序字段 */
export function getSortBy() {
  return sortBy;
}
/** 获取排序顺序 */
export function getSortOrder() {
  return sortOrder;
}
/** 获取视图模式 */
export function getViewMode() {
  return viewMode;
}

/** 设置排序字段，重复点击切换升/降序 */
export function setSortBy(by: SortBy) {
  if (sortBy === by) {
    sortOrder = sortOrder === "asc" ? "desc" : "asc";
  } else {
    sortBy = by;
    sortOrder = "asc";
  }
  sortItems();
}

/** 设置视图模式 */
export function setViewMode(mode: ViewMode) {
  viewMode = mode;
}

/** 切换文件选中状态，支持多选 */
export function toggleSelect(path: string, multi: boolean) {
  if (multi) {
    if (selectedPaths.has(path)) {
      selectedPaths.delete(path);
    } else {
      selectedPaths.add(path);
    }
    // 创建新 Set 以触发 Svelte 5 $state 响应式更新（Set.delete/Set.add 是 mutation 不触发）
    selectedPaths = new Set(selectedPaths);
  } else {
    selectedPaths = new Set([path]);
  }
}

/** 全选当前目录所有文件 */
export function selectAll() {
  selectedPaths = new Set(items.map((i) => i.path));
}

/** 清除所有选中 */
export function clearSelection() {
  selectedPaths = new Set();
}

/** 导航到指定路径 */
export async function navigateTo(path: string) {
  currentPath = path;
  selectedPaths = new Set();
  await refresh();
}

/** 重置到根目录 */
export async function resetToRoot() {
  currentPath = "/";
  selectedPaths = new Set();
  items = [];
  error = null;
  await refresh();
}

/** 刷新当前目录内容 */
export async function refresh() {
  loading = true;
  error = null;
  try {
    items = await api.files.listDirectory(currentPath);
    sortItems();
  } catch (e) {
    error = String(e);
    items = [];
  } finally {
    loading = false;
  }
}

/** 按当前排序规则对文件列表排序 */
function sortItems() {
  // 展开运算符创建新数组触发 $state 响应式更新（Array.sort() 是 in-place mutation）
  items = [...items].sort((a, b) => {
    // 目录始终排在文件前面（macOS Finder 风格），然后才按排序字段比较
    if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;

    let cmp = 0;
    switch (sortBy) {
      case "name":
        cmp = a.name.localeCompare(b.name);
        break;
      case "size":
        cmp = (a.size ?? 0) - (b.size ?? 0);
        break;
      case "modified":
        cmp = (a.modified ?? "").localeCompare(b.modified ?? "");
        break;
    }
    return sortOrder === "asc" ? cmp : -cmp;
  });
}

/** 生成面包屑导航路径段 */
export function getPathSegments(): { name: string; path: string }[] {
  const parts = currentPath.split("/").filter(Boolean);
  const segments: { name: string; path: string }[] = [
    { name: "Root", path: "/" },
  ];
  let accumulated = "";
  for (const part of parts) {
    accumulated += "/" + part;
    segments.push({ name: decodeURIComponent(part), path: accumulated });
  }
  return segments;
}

/** 重命名文件或文件夹 */
export async function renameItem(oldPath: string, newName: string): Promise<boolean> {
  try {
    const basePath = oldPath.split("/").slice(0, -1).join("/") || "/";
    const newPath = basePath === "/" ? "/" + newName : basePath + "/" + newName;
    await api.operations.renameItem(oldPath, newPath);
    showToast("重命名成功", "success");
    await refresh();
    return true;
  } catch (e) {
    showToast("重命名失败: " + String(e), "error");
    return false;
  }
}

/** 复制单个文件或文件夹到目标路径 */
export async function copyItem(sourcePath: string, targetDir: string): Promise<boolean> {
  try {
    const fileName = sourcePath.split("/").pop() || "file";
    const targetPath = targetDir === "/" ? "/" + fileName : targetDir + "/" + fileName;
    await api.operations.copyItem(sourcePath, targetPath);
    return true;
  } catch (e) {
    showToast("复制失败: " + String(e), "error");
    return false;
  }
}

/** 批量复制文件或文件夹到目标路径 */
export async function copyItems(sourcePaths: string[], targetDir: string): Promise<boolean> {
  try {
    let successCount = 0;
    for (const path of sourcePaths) {
      if (await copyItem(path, targetDir)) {
        successCount++;
      }
    }
    showToast(`成功复制 ${successCount}/${sourcePaths.length} 个项目`, "success");
    await refresh();
    return successCount === sourcePaths.length;
  } catch (e) {
    showToast("复制失败: " + String(e), "error");
    return false;
  }
}

/** 移动单个文件或文件夹到目标路径 */
export async function moveItem(sourcePath: string, targetDir: string): Promise<boolean> {
  try {
    const fileName = sourcePath.split("/").pop() || "file";
    const targetPath = targetDir === "/" ? "/" + fileName : targetDir + "/" + fileName;
    await api.operations.moveItem(sourcePath, targetPath);
    return true;
  } catch (e) {
    showToast("移动失败: " + String(e), "error");
    return false;
  }
}

/** 批量移动文件或文件夹到目标路径 */
export async function moveItems(sourcePaths: string[], targetDir: string): Promise<boolean> {
  try {
    let successCount = 0;
    for (const path of sourcePaths) {
      if (await moveItem(path, targetDir)) {
        successCount++;
      }
    }
    showToast(`成功移动 ${successCount}/${sourcePaths.length} 个项目`, "success");
    await refresh();
    return successCount === sourcePaths.length;
  } catch (e) {
    showToast("移动失败: " + String(e), "error");
    return false;
  }
}
