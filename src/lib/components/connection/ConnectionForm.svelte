<!-- WebDAV 连接表单组件 - 新建/编辑连接配置 -->
<script lang="ts">
  import { untrack } from "svelte";
  import { _, t } from "svelte-i18n";
  import { saveProfile, testConnection, getProfiles, deleteProfile } from "../../stores/connections.svelte";
  import { showToast } from "../../stores/toast.svelte";
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

  /** 编辑模式下加载现有配置 */
  $effect(() => {
    const id = editId;
    untrack(() => {
      const existing = id ? getProfiles().find((p) => p.id === id) : null;
      name = existing?.name ?? "";
      url = existing?.url ?? "";
      username = existing?.username ?? "";
      password = existing?.password ?? "";
    });
  });

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

<div class="border-t border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
  <h3 class="mb-3 text-sm font-semibold">{editId ? $_("connection.edit") : $_("connection.new")}</h3>
  <div class="space-y-2">
    <!-- 连接名称输入 -->
    <input
      type="text"
      placeholder={$_("connection.namePlaceholder")}
      bind:value={name}
      class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-3 py-1.5 text-sm outline-none focus:border-[var(--color-accent)]"
    />
    <!-- WebDAV URL 输入 -->
    <input
      type="url"
      placeholder={$_("connection.urlPlaceholder")}
      bind:value={url}
      class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-3 py-1.5 text-sm outline-none focus:border-[var(--color-accent)]"
    />
    <!-- 用户名输入 -->
    <input
      type="text"
      placeholder={$_("connection.usernamePlaceholder")}
      bind:value={username}
      class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-3 py-1.5 text-sm outline-none focus:border-[var(--color-accent)]"
    />
    <!-- 密码输入 -->
    <input
      type="password"
      placeholder={$_("connection.passwordPlaceholder")}
      bind:value={password}
      class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-3 py-1.5 text-sm outline-none focus:border-[var(--color-accent)]"
    />
    <!-- 操作按钮：保存、测试、删除、取消 -->
    <div class="flex gap-2 pt-1">
      <button
        class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-sm text-white hover:bg-[var(--color-accent-hover)]"
        onclick={handleSave}
      >
        {$_("connection.save")}
      </button>
      <button
        class="rounded-md border border-[var(--color-border)] px-3 py-1.5 text-sm hover:bg-[var(--color-bg-primary)]"
        onclick={handleTest}
        disabled={testing}
      >
        {testing ? $_("connection.testing") : $_("connection.test")}
      </button>
      {#if editId}
        <button
          class="rounded-md border border-[var(--color-danger)] px-3 py-1.5 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10"
          onclick={handleDelete}
        >
          {$_("connection.delete")}
        </button>
      {/if}
      <button
        class="rounded-md px-3 py-1.5 text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
        onclick={onClose}
      >
        {$_("connection.cancel")}
      </button>
    </div>
  </div>
</div>
