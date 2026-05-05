<!-- 挂载映射管理弹窗 - 查看、编辑、删除已有映射 -->
<script lang="ts">
  import { _ } from "svelte-i18n";
  import { getProfiles, loadProfiles } from "../../stores/connections.svelte";
  import { showToast } from "../../stores/toast.svelte";
  import { showConfirm } from "../../stores/dialog.svelte";
  import { api } from "../../api";
  import { X, Trash2, Pencil, Check } from "lucide-svelte";
  import type { MountMapping } from "../../types";

  let { onClose } = $props<{
    onClose: () => void;
  }>();

  let removing = $state<string | null>(null);
  let editing = $state<string | null>(null);
  let editValue = $state("");

  /** 展平所有挂载映射，附带连接信息 */
  const allMounts = $derived(
    getProfiles().flatMap((p) =>
      p.mounts.map((m) => ({
        connectionId: p.id,
        connectionName: p.name,
        remotePath: m.remote_path,
        localPath: m.local_path ?? "-",
      }))
    )
  );

  function decodePath(path: string): string {
    try {
      return decodeURIComponent(path);
    } catch {
      return path;
    }
  }

  function startEdit(connectionId: string, remotePath: string, localPath: string) {
    editing = `${connectionId}::${remotePath}`;
    editValue = localPath === "-" ? "" : localPath;
  }

  async function confirmEdit(connectionId: string, remotePath: string) {
    const trimmed = editValue.trim();
    if (!trimmed) return;
    try {
      await api.mount.updateMountLocalPath(connectionId, remotePath, trimmed);
      await loadProfiles();
    } catch (e) {
      showToast(String(e), "error");
    }
    editing = null;
  }

  async function handleRemove(connectionId: string, remotePath: string) {
    const decoded = decodePath(remotePath);
    const confirmed = await showConfirm(
      $_("mount.removeConfirm", { values: { path: decoded } }),
      $_("dialog.confirmTitle"),
      $_("dialog.confirmOk"),
      $_("dialog.confirmCancel"),
    );
    if (!confirmed) return;
    const key = `${connectionId}::${remotePath}`;
    removing = key;
    try {
      await api.mount.removeMount(connectionId, remotePath);
      await loadProfiles();
    } catch (e) {
      showToast(String(e), "error");
    } finally {
      removing = null;
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onClose}>
  <div
    class="w-full max-w-2xl max-h-[70vh] rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] shadow-xl flex flex-col"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- 标题栏 -->
    <div class="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-3 shrink-0">
      <h3 class="text-sm font-semibold text-[var(--color-text-primary)]">{$_("mount.manageTitle")}</h3>
      <button
        class="rounded-md p-1 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
        onclick={onClose}
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <!-- 内容 -->
    <div class="overflow-y-auto p-5">
      {#if allMounts.length === 0}
        <div class="py-6 text-center text-sm text-[var(--color-text-secondary)]">
          {$_("mount.noMounts")}
        </div>
      {:else}
        <!-- 表头 -->
        <div class="grid grid-cols-[1fr_1fr_1fr_auto_auto] gap-3 px-2 pb-2 text-xs font-medium text-[var(--color-text-secondary)] border-b border-[var(--color-border)]">
          <span>{$_("mount.colConnection")}</span>
          <span>{$_("mount.colRemote")}</span>
          <span>{$_("mount.colLocal")}</span>
          <span class="w-7"></span>
          <span class="w-7"></span>
        </div>
        <!-- 行 -->
        {#each allMounts as m (`${m.connectionId}::${m.remotePath}`)}
          {@const key = `${m.connectionId}::${m.remotePath}`}
          {@const isEditing = editing === key}
          {@const isRemoving = removing === key}
          <div class="grid grid-cols-[1fr_1fr_1fr_auto_auto] gap-3 items-center px-2 py-2 text-sm border-b border-[var(--color-border)] hover:bg-[var(--color-bg-secondary)]">
            <!-- 连接名 -->
            <span class="truncate text-[var(--color-text-primary)]">{m.connectionName}</span>
            <!-- 远端目录（URL 解码） -->
            <span class="truncate text-[var(--color-text-primary)]" title={m.remotePath}>{decodePath(m.remotePath)}</span>
            <!-- 本地目录（可编辑） -->
            {#if isEditing}
              <div class="flex items-center gap-1">
                <input
                  type="text"
                  bind:value={editValue}
                  class="w-full min-w-0 rounded border border-[var(--color-accent)] bg-[var(--color-bg-primary)] px-2 py-0.5 text-sm outline-none"
                  onkeydown={(e) => { if (e.key === "Enter") confirmEdit(m.connectionId, m.remotePath); if (e.key === "Escape") editing = null; }}
                />
                <button
                  class="shrink-0 rounded p-0.5 text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10"
                  onclick={() => confirmEdit(m.connectionId, m.remotePath)}
                >
                  <Check class="h-4 w-4" />
                </button>
              </div>
            {:else}
              <span
                class="truncate text-[var(--color-text-primary)] cursor-default"
                title={m.localPath}
              >
                {m.localPath}
              </span>
            {/if}
            <!-- 编辑按钮 -->
            <button
              class="shrink-0 rounded p-1 text-[var(--color-text-secondary)] hover:text-[var(--color-accent)] disabled:opacity-50"
              disabled={isEditing || isRemoving}
              onclick={() => startEdit(m.connectionId, m.remotePath, m.localPath)}
            >
              <Pencil class="h-3.5 w-3.5" />
            </button>
            <!-- 删除按钮 -->
            <button
              class="shrink-0 rounded p-1 text-[var(--color-text-secondary)] hover:text-[var(--color-danger)] disabled:opacity-50"
              disabled={isRemoving}
              onclick={() => handleRemove(m.connectionId, m.remotePath)}
            >
              {#if isRemoving}
                <div class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-[var(--color-text-secondary)] border-t-transparent"></div>
              {:else}
                <Trash2 class="h-3.5 w-3.5" />
              {/if}
            </button>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
