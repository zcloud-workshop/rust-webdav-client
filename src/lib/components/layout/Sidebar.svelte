<script lang="ts">
  import { _ } from "svelte-i18n";
  import {
    getProfiles,
    getActiveId,
    getLoading,
    loadProfiles,
    connect,
    disconnect,
    deleteProfile,
    testConnection,
  } from "../../stores/connections.svelte";
  import { showConfirm } from "../../stores/dialog.svelte";
  import { showToast } from "../../stores/toast.svelte";
  import { PanelLeftClose, PanelLeftOpen, Settings, Server, Plug, Unplug, Pencil, Zap, Trash2, Plus } from "lucide-svelte";
  import ConnectionForm from "../connection/ConnectionForm.svelte";
  import ContextMenu from "../common/ContextMenu.svelte";
  import SettingsModal from "../common/SettingsModal.svelte";

  let {
    connected = $bindable(false),
    collapsed = false,
    onToggle = () => {},
  } = $props<{
    connected?: boolean;
    collapsed?: boolean;
    onToggle?: () => void;
  }>();
  let showForm = $state(false);
  let editingId = $state<string | null>(null);
  let ctxMenu = $state<{ x: number; y: number; profileId: string } | null>(null);
  let blankCtx = $state<{ x: number; y: number } | null>(null);
  let showSettings = $state(false);

  $effect(() => {
    loadProfiles();
  });

  async function handleConnect(id: string) {
    try {
      await connect(id);
      connected = true;
    } catch {
      // error handled in store
    }
  }

  async function handleDisconnect() {
    await disconnect();
    connected = false;
  }

  function handleEdit(id: string) {
    editingId = id;
    showForm = true;
  }

  async function handleTest(id: string) {
    const profile = getProfiles().find((p) => p.id === id);
    if (!profile) return;
    try {
      const ok = await testConnection(profile);
      if (ok) {
        showToast($_("connection.testSuccess"), "success");
      } else {
        showToast($_("connection.testFailed"), "error");
      }
    } catch (e) {
      showToast($_("connection.testError", { values: { error: String(e) } }), "error");
    }
  }

  async function handleDelete(id: string) {
    const profile = getProfiles().find((p) => p.id === id);
    const confirmed = await showConfirm(
      $_("connection.deleteConfirm", { values: { name: profile?.name ?? "" } }),
      $_("dialog.confirmTitle"),
    );
    if (confirmed) {
      await deleteProfile(id);
      if (getActiveId() === id) {
        connected = false;
      }
    }
  }

  function handleContextMenu(e: MouseEvent, profileId: string) {
    e.preventDefault();
    e.stopPropagation();
    blankCtx = null;
    ctxMenu = { x: e.clientX, y: e.clientY, profileId };
  }

  function handleBlankContext(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    ctxMenu = null;
    blankCtx = { x: e.clientX, y: e.clientY };
  }
</script>

<aside class="flex h-full flex-col border-r border-[var(--color-border)] bg-[var(--color-bg-sidebar)]">
  <div
    class="flex items-center border-b border-[var(--color-border)] px-3 py-3 {collapsed ? 'justify-center' : 'justify-between'}"
  >
    {#if !collapsed}
      <h2 class="flex-1 text-center text-sm font-semibold text-[var(--color-text-primary)]">{$_("connection.title")}</h2>
    {/if}
    <button
      class="shrink-0 rounded-md p-1.5 text-[var(--color-text-secondary)] hover:bg-[var(--color-accent)]/10 hover:text-[var(--color-accent)] transition-colors"
      onclick={onToggle}
      title={collapsed ? $_("sidebar.expand") : $_("sidebar.collapse")}
    >
      {#if collapsed}
        <PanelLeftOpen class="h-4 w-4" />
      {:else}
        <PanelLeftClose class="h-4 w-4" />
      {/if}
    </button>
  </div>

  {#if collapsed}
    <!-- collapsed: icons only -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="flex flex-1 flex-col items-center gap-1 overflow-y-auto py-2" oncontextmenu={handleBlankContext}>
      {#each getProfiles() as profile (profile.id)}
        <button
          class="flex h-8 w-8 items-center justify-center rounded-md text-xs font-medium transition-colors {getActiveId() === profile.id
            ? 'bg-[var(--color-accent)] text-white'
            : 'text-[var(--color-text-primary)] hover:bg-[var(--color-accent)]/10'}"
          onclick={() => {
            if (getActiveId() === profile.id) { handleDisconnect(); } else { handleConnect(profile.id); }
          }}
          oncontextmenu={(e) => handleContextMenu(e, profile.id)}
          title={profile.name}
        >
          {profile.name.charAt(0).toUpperCase()}
        </button>
      {/each}
    </div>

    <div class="border-t border-[var(--color-border)] p-2">
      <button
        class="flex h-8 w-full items-center justify-center rounded-md text-[var(--color-text-secondary)] hover:bg-[var(--color-accent)]/10 hover:text-[var(--color-accent)]"
        onclick={() => showSettings = true}
        title={$_("settings.title")}
      >
        <Settings class="h-4 w-4" />
      </button>
    </div>
  {:else}
    <!-- expanded: full sidebar -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="flex-1 overflow-y-auto p-2" oncontextmenu={handleBlankContext}>
      {#if getLoading()}
        <div class="px-2 py-4 text-center text-sm text-[var(--color-text-secondary)]">{$_("connection.loading")}</div>
      {:else if getProfiles().length === 0}
        <div class="flex flex-col items-center gap-3 px-2 py-6">
          <button
            class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs text-white hover:bg-[var(--color-accent-hover)]"
            onclick={() => { editingId = null; showForm = true; }}
          >
            {$_("connection.add")}
          </button>
        </div>
      {:else}
        {#each getProfiles() as profile (profile.id)}
          <button
            class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm transition-colors {getActiveId() === profile.id
              ? 'bg-[var(--color-accent)] text-white'
              : 'text-[var(--color-text-primary)] hover:bg-[var(--color-accent)]/10'}"
            onclick={() => {
              if (getActiveId() === profile.id) {
                handleDisconnect();
              } else {
                handleConnect(profile.id);
              }
            }}
            oncontextmenu={(e) => handleContextMenu(e, profile.id)}
          >
            <Server class="h-4 w-4 shrink-0" />
            <span class="truncate">{profile.name}</span>
          </button>
        {/each}
      {/if}
    </div>

    <div class="border-t border-[var(--color-border)] px-4 py-2">
      <button
        class="flex w-full items-center justify-center gap-1.5 rounded-md px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-accent)]/10 hover:text-[var(--color-accent)]"
        onclick={() => showSettings = true}
      >
        <Settings class="h-4 w-4" />
        {$_("settings.title")}
      </button>
    </div>
  {/if}

  {#if showForm}
    <ConnectionForm
      editId={editingId}
      onClose={() => { showForm = false; editingId = null; }}
    />
  {/if}

  {#if ctxMenu}
    {@const isActive = getActiveId() === ctxMenu.profileId}
    <ContextMenu
      x={ctxMenu.x}
      y={ctxMenu.y}
      items={[
        ...(isActive
          ? [{ label: $_("connection.ctxDisconnect"), icon: Unplug, action: () => { handleDisconnect(); ctxMenu = null; } }]
          : [{ label: $_("connection.ctxConnect"), icon: Plug, action: () => { handleConnect(ctxMenu!.profileId); ctxMenu = null; } }]
        ),
        { label: $_("connection.ctxEdit"), icon: Pencil, action: () => { handleEdit(ctxMenu!.profileId); ctxMenu = null; } },
        { label: $_("connection.ctxTest"), icon: Zap, action: () => { const id = ctxMenu!.profileId; ctxMenu = null; handleTest(id); } },
        { label: $_("connection.ctxDelete"), icon: Trash2, action: () => { const id = ctxMenu!.profileId; ctxMenu = null; handleDelete(id); } },
      ]}
      onClose={() => { ctxMenu = null; }}
    />
  {/if}

  {#if blankCtx}
    <ContextMenu
      x={blankCtx.x}
      y={blankCtx.y}
      items={[
        { label: $_("connection.ctxNew"), icon: Plus, action: () => { editingId = null; showForm = true; blankCtx = null; } },
      ]}
      onClose={() => { blankCtx = null; }}
    />
  {/if}

  {#if showSettings}
    <SettingsModal onClose={() => { showSettings = false; }} />
  {/if}
</aside>
