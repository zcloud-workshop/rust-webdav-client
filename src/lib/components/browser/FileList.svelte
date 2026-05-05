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
  import { getActiveId, getProfiles } from "../../stores/connections.svelte";
  import FileItem from "./FileItem.svelte";
  import ContextMenu from "../common/ContextMenu.svelte";
  import { Pencil, Copy, ArrowRight, Download, Trash2, HardDrive } from "lucide-svelte";

  function tr(key: string, options?: { values?: Record<string, string | number> }): string {
    return get(t)(key, options)?.toString() || "";
  }

  /** 右键菜单状态 */
  let contextMenu = $state<{ x: number; y: number; path: string } | null>(null);

  /** 映射弹窗状态 */
  let mountDialog = $state<{ path: string; input: string } | null>(null);

  /** 是否为根目录 - 根目录禁止选择和右键操作 */
  let isRoot = $derived(getCurrentPath() === "/");

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

  /** 判断目录是否已挂载 */
  function isMounted(path: string): boolean {
    const id = getActiveId();
    if (!id) return false;
    const profile = getProfiles().find((p) => p.id === id);
    return profile?.mounts.some((m) => m.remote_path === path) ?? false;
  }

  /** 右键菜单操作 - 映射到本地（弹出输入框） */
  function handleContextMount() {
    if (!contextMenu) return;
    const path = contextMenu.path;
    closeContextMenu();
    const dirName = decodeURIComponent(path.split("/").filter(Boolean).pop() ?? "mount");
    const defaultPath = navigator.userAgent.includes("Windows")
      ? "Z:"
      : `/Volumes/${dirName}`;
    mountDialog = { path, input: defaultPath };
  }

  /** 确认映射 */
  async function confirmMount() {
    if (!mountDialog) return;
    const { path, input } = mountDialog;
    mountDialog = null;
    const connectionId = getActiveId();
    if (!connectionId) return;

    try {
      const localPath = await api.mount.mountDirectory(connectionId, path, input || undefined);
      showToast(tr("mount.success", { values: { path: localPath } }), "success");
      await refresh();
    } catch (e) {
      const msg = String(e);
      if (msg.includes("not empty")) {
        showToast(tr("mount.targetNotEmpty", { values: { path: input || "" } }), "error");
      } else {
        showToast(tr("mount.failed", { values: { error: msg } }), "error");
      }
    }
  }

  /** 浏览选择本地目录 */
  async function browseLocalDir() {
    if (!mountDialog) return;
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      mountDialog = { ...mountDialog, input: selected };
    }
  }

  /** 右键菜单操作 - 取消映射 */
  async function handleContextUnmount() {
    if (!contextMenu) return;
    const path = contextMenu.path;
    closeContextMenu();
    const connectionId = getActiveId();
    if (!connectionId) return;
    try {
      await api.mount.unmountDirectory(connectionId, path);
      showToast(tr("mount.unmountSuccess"), "success");
    } catch (e) {
      showToast(tr("mount.unmountFailed", { values: { error: String(e) } }), "error");
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
    <div class="sticky top-0 {isRoot ? 'grid-cols-[1fr_100px_160px]' : 'grid-cols-[auto_1fr_100px_160px]'} grid gap-4 border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-4 py-2 text-xs font-medium text-[var(--color-text-secondary)] items-center">
      <!-- 全选框 -->
      {#if !isRoot}
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
      {/if}
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
        readonly={isRoot}
      />
    {/each}
  {/if}
</div>

<!-- 右键菜单 -->
{#if contextMenu}
  {@const ctxItem = getItems().find((i) => i.path === contextMenu!.path)}
  {@const ctxMounted = ctxItem?.is_dir ? isMounted(contextMenu!.path) : false}
  <ContextMenu
    x={contextMenu!.x}
    y={contextMenu!.y}
    items={isRoot
      ? ctxItem?.is_dir
        ? [{ label: ctxMounted ? tr("mount.unmap") : tr("mount.mapToLocal"), icon: HardDrive, action: ctxMounted ? handleContextUnmount : handleContextMount }]
        : []
      : [
        ...(ctxItem?.is_dir
          ? [{ label: ctxMounted ? tr("mount.unmap") : tr("mount.mapToLocal"), icon: HardDrive, action: ctxMounted ? handleContextUnmount : handleContextMount }]
          : []),
        { label: "重命名", icon: Pencil, action: handleContextRename },
        { label: "复制", icon: Copy, action: handleContextCopy },
        { label: "移动", icon: ArrowRight, action: handleContextMove },
        { label: "下载", icon: Download, action: handleContextDownload },
        { label: "删除", icon: Trash2, action: handleContextDelete },
      ]
    }
    onClose={closeContextMenu}
  />
{/if}

<!-- 映射路径输入弹窗 -->
{#if mountDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={() => { mountDialog = null; }}>
    <div
      class="w-full max-w-sm rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] shadow-xl"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="px-4 py-3 text-sm font-medium text-[var(--color-text-primary)]">{$_("mount.localPathPrompt")}</div>
      <div class="px-4 pb-2 flex gap-2">
        <input
          type="text"
          bind:value={mountDialog.input}
          class="flex-1 min-w-0 rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
          onkeydown={(e) => { if (e.key === "Enter") confirmMount(); if (e.key === "Escape") mountDialog = null; }}
        />
        <button
          class="shrink-0 rounded-md border border-[var(--color-border)] px-3 py-2 text-sm hover:bg-[var(--color-bg-secondary)]"
          onclick={browseLocalDir}
        >
          ...
        </button>
      </div>
      <div class="flex justify-end gap-2 px-4 py-3 border-t border-[var(--color-border)]">
        <button
          class="rounded-md px-3 py-1.5 text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
          onclick={() => { mountDialog = null; }}
        >
          {$_("connection.cancel")}
        </button>
        <button
          class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-sm text-white hover:bg-[var(--color-accent-hover)]"
          onclick={confirmMount}
        >
          {$_("connection.save")}
        </button>
      </div>
    </div>
  </div>
{/if}
