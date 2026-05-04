<!-- 文件列表组件 - 显示当前目录的所有文件和文件夹 -->
<script lang="ts">
  import { _, t } from "svelte-i18n";
  import { get } from "svelte/store";
  import {
    getItems,
    getLoading,
    getError,
    getSelectedPaths,
    toggleSelect,
    selectAll,
    clearSelection,
    navigateTo,
    renameItem,
    copyItems,
    moveItems,
    getCurrentPath,
    refresh,
  } from "../../stores/browser.svelte";
  import { getFileCategory } from "../../utils/file-types";
  import { openFile } from "../../stores/preview.svelte";
  import { showToast } from "../../stores/toast.svelte";
  import { showConfirm } from "../../stores/dialog.svelte";
  import { api } from "../../api";
  import FileItem from "./FileItem.svelte";
  import ContextMenu from "../common/ContextMenu.svelte";
  import { Pencil, Copy, ArrowRight, Download, Trash2 } from "lucide-svelte";

  function tr(key: string, options?: { values?: Record<string, string | number> }): string {
    return get(t)(key, options)?.toString() || "";
  }

  /** 右键菜单状态 */
  let contextMenu = $state<{ x: number; y: number; path: string } | null>(null);

  /** 关闭右键菜单 */
  function closeContextMenu() {
    contextMenu = null;
  }

  /** 右键菜单操作 - 重命名 */
  async function handleContextRename() {
    if (!contextMenu) return;
    const path = contextMenu.path;
    closeContextMenu();
    const currentName = path.split("/").pop() || "";
    const newName = prompt("请输入新名称:", currentName);
    if (newName && newName.trim() && newName.trim() !== currentName) {
      const confirmed = await showConfirm(
        tr("dialog.renameConfirm", { values: { oldName: currentName, newName: newName.trim() } }),
        tr("dialog.confirmTitle"),
        tr("dialog.confirmOk"),
        tr("dialog.confirmCancel"),
      );
      if (confirmed) {
        await renameItem(path, newName.trim());
      }
    }
  }

  /** 右键菜单操作 - 复制 */
  async function handleContextCopy() {
    if (!contextMenu) return;
    const path = contextMenu.path;
    const name = path.split("/").pop() || "";
    closeContextMenu();
    const confirmed = await showConfirm(
      tr("dialog.copyConfirm", { values: { name } }),
      tr("dialog.confirmTitle"),
      tr("dialog.confirmOk"),
      tr("dialog.confirmCancel"),
    );
    if (confirmed) {
      await copyItems([path], getCurrentPath());
    }
  }

  /** 右键菜单操作 - 移动 */
  async function handleContextMove() {
    if (!contextMenu) return;
    const path = contextMenu.path;
    const name = path.split("/").pop() || "";
    closeContextMenu();
    const confirmed = await showConfirm(
      tr("dialog.moveConfirm", { values: { name } }),
      tr("dialog.confirmTitle"),
      tr("dialog.confirmOk"),
      tr("dialog.confirmCancel"),
    );
    if (confirmed) {
      await moveItems([path], getCurrentPath());
    }
  }

  /** 右键菜单操作 - 删除 */
  async function handleContextDelete() {
    if (!contextMenu) return;
    const path = contextMenu.path;
    const name = path.split("/").pop() || "";
    closeContextMenu();
    const confirmed = await showConfirm(
      tr("dialog.deleteConfirm", { values: { name } }),
      tr("dialog.confirmTitle"),
      tr("dialog.confirmOk"),
      tr("dialog.confirmCancel"),
    );
    if (confirmed) {
      await api.operations.deleteItem(path);
      showToast(tr("toolbar.deleted"), "success");
      await refresh();
    }
  }

  /** 右键菜单操作 - 下载 */
  async function handleContextDownload() {
    if (!contextMenu) return;
    const path = contextMenu.path;
    closeContextMenu();
    const fileName = path.split("/").pop() ?? "download";
    const { save } = await import("@tauri-apps/plugin-dialog");
    const destPath = await save({ defaultPath: fileName });
    if (destPath) {
      await api.download.downloadFileTo(path, destPath);
      showToast("下载成功", "success");
    }
  }

  /** 处理右键点击 */
  function handleContextMenu(e: MouseEvent, path: string) {
    e.preventDefault();
    e.stopPropagation();
    // 如果点击的项目未被选中，先选中它
    if (!getSelectedPaths().has(path)) {
      clearSelection();
      toggleSelect(path, false);
    }
    contextMenu = { x: e.clientX, y: e.clientY, path };
  }

  /** 双击处理 - 目录进入导航，文件打开预览 */
  function handleDoubleClick(item: { path: string; name: string; is_dir: boolean; size: number | null }) {
    if (item.is_dir) {
      navigateTo(item.path);
    } else {
      const category = getFileCategory(item.name);
      openFile(item.path, item.name, category, item.size);
    }
  }

  /** 键盘快捷键处理 - Ctrl+A 全选 */
  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === "a") {
      e.preventDefault();
      selectAll();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="flex-1 overflow-y-auto"
  role="grid"
  tabindex="0"
>
  <!-- 加载状态显示 -->
  {#if getLoading()}
    <div class="flex items-center justify-center py-20">
      <div class="h-6 w-6 animate-spin rounded-full border-2 border-[var(--color-accent)] border-t-transparent"></div>
    </div>
  <!-- 错误状态显示 -->
  {:else if getError()}
    <div class="px-6 py-10 text-center text-red-500">{getError()}</div>
  <!-- 空目录显示 -->
  {:else if getItems().length === 0}
    <div class="px-6 py-10 text-center text-sm text-[var(--color-text-secondary)]">
      {$_("browser.empty")}
    </div>
  <!-- 文件列表显示 -->
  {:else}
    <!-- 表头 - 固定在顶部 -->
    <div class="sticky top-0 grid grid-cols-[auto_1fr_100px_160px] gap-4 border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-4 py-2 text-xs font-medium text-[var(--color-text-secondary)] items-center">
      <!-- 全选框 -->
      <div class="shrink-0">
        <input
          type="checkbox"
          checked={getSelectedPaths().size === getItems().length && getItems().length > 0}
          onchange={() => {
            if (getSelectedPaths().size === getItems().length) {
              clearSelection();
            } else {
              selectAll();
            }
          }}
        />
      </div>
      <span>{$_("browser.name")}</span>
      <span class="text-right">{$_("browser.size")}</span>
      <span>{$_("browser.modified")}</span>
    </div>
    <!-- 文件列表项 -->
    {#each getItems() as item (item.path)}
      <FileItem
        {item}
        selected={getSelectedPaths().has(item.path)}
        onselect={() => toggleSelect(item.path, false)}
        ondblclick={() => handleDoubleClick(item)}
        oncheckbox={() => toggleSelect(item.path, true)}
        oncontextmenu={(e) => handleContextMenu(e, item.path)}
      />
    {/each}
  {/if}
</div>

<!-- 右键菜单 -->
{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={[
      { label: "重命名", icon: Pencil, action: handleContextRename },
      { label: "复制", icon: Copy, action: handleContextCopy },
      { label: "移动", icon: ArrowRight, action: handleContextMove },
      { label: "下载", icon: Download, action: handleContextDownload },
      { label: "删除", icon: Trash2, action: handleContextDelete },
    ]}
    onClose={closeContextMenu}
  />
{/if}
