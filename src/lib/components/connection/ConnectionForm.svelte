<!-- WebDAV 连接表单组件 - 新建/编辑连接配置 -->
<script lang="ts">
  import { untrack } from "svelte";
  import { _, t } from "svelte-i18n";
  import { saveProfile, testConnection, getProfiles, deleteProfile } from "../../stores/connections.svelte";
  import { showToast } from "../../stores/toast.svelte";
  import { api } from "../../api";
  import { X, Eye, EyeOff } from "lucide-svelte";
  import type { ConnectionProfile } from "../../types";

  /** 组件属性：编辑的连接 ID 和关闭回调 */
  let { editId = null, onClose } = $props<{
    editId: string | null;
    onClose: () => void;
  }>();

  /** 连接名称 */
  let name = $state("");
  /** WebDAV 服务器 URL */
  let url = $state("");
  /** 用户名 */
  let username = $state("");
  /** 密码 */
  let password = $state("");
  /** 是否正在测试连接 */
  let testing = $state(false);
  /** 密码是否可见 */
  let showPwd = $state(false);
  /** 是否允许不安全的 SSL 证书 */
  let acceptInsecure = $state(false);
  /** 隐藏的根目录名称列表 */
  let hiddenRootDirs = $state<string[]>([]);
  /** 隐藏目录输入框的当前值 */
  let hiddenDirInput = $state("");
  /** 从服务器加载的根目录列表 */
  let remoteDirs = $state<string[]>([]);
  /** 是否正在加载远程目录 */
  let loadingDirs = $state(false);
  /** 多选列表是否展开 */
  let showDirPicker = $state(false);

  /** 编辑模式下加载现有配置 */
  $effect(() => {
    const id = editId;
    untrack(() => {
      const existing = id ? getProfiles().find((p) => p.id === id) : null;
      name = existing?.name ?? "";
      url = existing?.url ?? "";
      username = existing?.username ?? "";
      password = existing?.password ?? "";
      acceptInsecure = existing?.accept_insecure ?? false;
      hiddenRootDirs = existing?.hidden_root_dirs ?? [];
    });
  });

  /** 添加一个要隐藏的目录名称 */
  function addHiddenDir() {
    const trimmed = hiddenDirInput.trim();
    if (trimmed && !hiddenRootDirs.includes(trimmed)) {
      hiddenRootDirs = [...hiddenRootDirs, trimmed];
    }
    hiddenDirInput = "";
  }

  /** 移除一个隐藏的目录名称 */
  function removeHiddenDir(name: string) {
    hiddenRootDirs = hiddenRootDirs.filter((d) => d !== name);
  }

  /** 切换目录的隐藏状态（多选列表用） */
  function toggleHiddenDir(dirName: string) {
    if (hiddenRootDirs.includes(dirName)) {
      hiddenRootDirs = hiddenRootDirs.filter((d) => d !== dirName);
    } else {
      hiddenRootDirs = [...hiddenRootDirs, dirName];
    }
  }

  /** 从服务器加载根目录列表 */
  async function loadRemoteDirs() {
    if (!url || !username) {
      showToast($t("connection.requiredFields"), "error");
      return;
    }
    loadingDirs = true;
    try {
      remoteDirs = await api.connection.listRemoteRootDirs({
        id: "temp",
        name,
        url: url.endsWith("/") ? url : url + "/",
        username,
        password,
        accept_insecure: acceptInsecure,
        hidden_root_dirs: [],
      });
      showDirPicker = true;
    } catch (e) {
      showToast($t("connection.testError") + " " + e, "error");
    } finally {
      loadingDirs = false;
    }
  }

  /** 保存连接配置 */
  async function handleSave() {
    if (!name || !url || !username) {
      showToast($t("connection.allRequired"), "error");
      return;
    }
    const existing = editId ? getProfiles().find((p) => p.id === editId) : null;
    const profile: ConnectionProfile = {
      id: existing?.id ?? crypto.randomUUID(),
      name,
      url: url.endsWith("/") ? url : url + "/", // 确保 URL 以 / 结尾
      username,
      password,
      accept_insecure: acceptInsecure,
      hidden_root_dirs: hiddenRootDirs,
    };
    try {
      await saveProfile(profile);
      showToast($t("connection.saveSuccess"), "success");
      onClose();
    } catch (e) {
      showToast($t("connection.saveFailed") + " " + e, "error");
    }
  }

  /** 测试连接配置是否有效 */
  async function handleTest() {
    if (!url || !username) {
      showToast($t("connection.requiredFields"), "error");
      return;
    }
    testing = true;
    try {
      const ok = await testConnection({
        id: "test",
        name,
        url: url.endsWith("/") ? url : url + "/",
        username,
        password,
        accept_insecure: acceptInsecure,
        hidden_root_dirs: hiddenRootDirs,
      });
      if (ok) {
        showToast($t("connection.testSuccess"), "success");
      } else {
        showToast($t("connection.testFailed"), "error");
      }
    } catch (e) {
      showToast($t("connection.testError") + " " + e, "error");
    } finally {
      testing = false;
    }
  }

  /** 删除连接配置 */
  async function handleDelete() {
    if (!editId) return;
    const profileName = getProfiles().find((p) => p.id === editId)?.name;
    if (!confirm($t("dialog.deleteConfirm", { values: { name: profileName || "" } }))) {
      return;
    }
    try {
      await deleteProfile(editId);
      showToast($t("toolbar.deleted"), "success");
      onClose();
    } catch (e) {
      showToast($t("toolbar.deleteFailed", { values: { error: String(e) } }), "error");
    }
  }
</script>

<!-- 遮罩层 -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onClose}>
  <!-- 弹窗主体 -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="w-full max-w-md rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] shadow-xl"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- 标题栏 -->
    <div class="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-3">
      <h3 class="text-sm font-semibold text-[var(--color-text-primary)]">{editId ? $_("connection.edit") : $_("connection.new")}</h3>
      <button
        class="rounded-md p-1 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
        onclick={onClose}
      >
        <X class="h-4 w-4" />
      </button>
    </div>
    <!-- 表单内容 -->
    <div class="space-y-3 p-5">
      <input
        type="text"
        placeholder={$_("connection.namePlaceholder")}
        bind:value={name}
        class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
      />
      <input
        type="url"
        placeholder={$_("connection.urlPlaceholder")}
        bind:value={url}
        class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
      />
      <input
        type="text"
        placeholder={$_("connection.usernamePlaceholder")}
        bind:value={username}
        class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
      />
      <div class="relative">
        <input
          type={showPwd ? "text" : "password"}
          placeholder={$_("connection.passwordPlaceholder")}
          bind:value={password}
          class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] pr-9 px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
        />
        <button
          type="button"
          class="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
          onclick={() => showPwd = !showPwd}
        >
          {#if showPwd}
            <EyeOff class="h-4 w-4" />
          {:else}
            <Eye class="h-4 w-4" />
          {/if}
        </button>
      </div>
      <!-- 允许不安全证书 -->
      <label class="flex items-center gap-2 text-sm text-[var(--color-text-secondary)] select-none">
        <input
          type="checkbox"
          bind:checked={acceptInsecure}
          class="accent-[var(--color-accent)]"
        />
        {$_("connection.acceptInsecure")}
      </label>
      <!-- 隐藏根目录设置 -->
      <div class="space-y-1.5">
        <span class="text-sm text-[var(--color-text-secondary)]">
          {$_("connection.hiddenRootDirs")}
        </span>
        {#if hiddenRootDirs.length > 0}
          <div class="flex flex-wrap gap-1.5">
            {#each hiddenRootDirs as dirName (dirName)}
              <span class="inline-flex items-center gap-1 rounded-md border border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-2 py-0.5 text-xs text-[var(--color-text-primary)]">
                {dirName}
                <button
                  type="button"
                  class="text-[var(--color-text-secondary)] hover:text-[var(--color-danger)]"
                  onclick={() => removeHiddenDir(dirName)}
                >
                  <X class="h-3 w-3" />
                </button>
              </span>
            {/each}
          </div>
        {/if}
        <div class="flex gap-1.5">
          <input
            type="text"
            placeholder={$_("connection.hiddenRootDirsPlaceholder")}
            bind:value={hiddenDirInput}
            onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); addHiddenDir(); } } }
            class="flex-1 rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-3 py-1.5 text-sm outline-none focus:border-[var(--color-accent)]"
          />
          <button
            type="button"
            class="rounded-md border border-[var(--color-border)] px-3 py-1.5 text-sm hover:bg-[var(--color-bg-secondary)]"
            onclick={addHiddenDir}
          >
            +
          </button>
          <button
            type="button"
            class="rounded-md border border-[var(--color-border)] px-3 py-1.5 text-sm hover:bg-[var(--color-bg-secondary)] whitespace-nowrap"
            onclick={loadRemoteDirs}
            disabled={loadingDirs}
          >
            {loadingDirs ? $_("connection.loading") : $_("connection.loadFromServer")}
          </button>
        </div>
        {#if showDirPicker && remoteDirs.length > 0}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="max-h-40 overflow-y-auto rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] p-2 space-y-1">
            {#each remoteDirs as dirName (dirName)}
              <label class="flex items-center gap-2 text-sm text-[var(--color-text-primary)] cursor-pointer select-none">
                <input
                  type="checkbox"
                  checked={hiddenRootDirs.includes(dirName)}
                  onchange={() => toggleHiddenDir(dirName)}
                  class="accent-[var(--color-accent)]"
                />
                {dirName}
              </label>
            {/each}
          </div>
        {/if}
      </div>
      <!-- 操作按钮 -->
      <div class="flex items-center gap-2 pt-2">
        <button
          class="rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm text-white hover:bg-[var(--color-accent-hover)]"
          onclick={handleSave}
        >
          {$_("connection.save")}
        </button>
        <button
          class="rounded-md border border-[var(--color-border)] px-4 py-2 text-sm hover:bg-[var(--color-bg-secondary)]"
          onclick={handleTest}
          disabled={testing}
        >
          {testing ? $_("connection.testing") : $_("connection.test")}
        </button>
        <div class="flex-1"></div>
        {#if editId}
          <button
            class="rounded-md border border-[var(--color-danger)] px-4 py-2 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10"
            onclick={handleDelete}
          >
            {$_("connection.delete")}
          </button>
        {/if}
        <button
          class="rounded-md px-4 py-2 text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
          onclick={onClose}
        >
          {$_("connection.cancel")}
        </button>
      </div>
    </div>
  </div>
</div>
