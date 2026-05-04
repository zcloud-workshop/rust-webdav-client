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
  import ConnectionForm from "../connection/ConnectionForm.svelte";
  import ContextMenu from "../common/ContextMenu.svelte";
  import SettingsModal from "../common/SettingsModal.svelte";

  let { connected = $bindable(false) } = $props();
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

<aside class="flex h-full w-64 flex-col border-r border-[var(--color-border)] bg-[var(--color-bg-sidebar)]">
  <div class="border-b border-[var(--color-border)] px-4 py-3">
    <h2 class="text-center text-sm font-semibold text-[var(--color-text-primary)]">{$_("connection.title")}</h2>
  </div>

  <div class="flex-1 overflow-y-auto p-2" oncontextmenu={handleBlankContext}>
    {#if getLoading()}
      <div class="px-2 py-4 text-center text-sm text-[var(--color-text-secondary)]">{$_("connection.loading")}</div>
    {:else if getProfiles().length === 0}
      <div class="flex flex-col items-center gap-3 px-2 py-6">
        <span class="text-sm text-[var(--color-text-secondary)]">{$_("connection.noConnections")}</span>
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
          <svg class="h-4 w-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          <span class="truncate">{profile.name}</span>
        </button>
      {/each}
    {/if}
  </div>

  {#if showForm}
    <ConnectionForm
      editId={editingId}
      onClose={() => { showForm = false; editingId = null; }}
    />
  {/if}

  <div class="border-t border-[var(--color-border)] px-4 py-2">
    <button
      class="flex w-full items-center justify-center gap-1.5 rounded-md px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-accent)]/10 hover:text-[var(--color-accent)]"
      onclick={() => showSettings = true}
    >
      <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
      {$_("settings.title")}
    </button>
  </div>

  {#if ctxMenu}
    {@const isActive = getActiveId() === ctxMenu.profileId}
    <ContextMenu
      x={ctxMenu.x}
      y={ctxMenu.y}
      items={[
        ...(isActive
          ? [{ label: $_("connection.ctxDisconnect"), icon: "⏏", action: () => { handleDisconnect(); ctxMenu = null; } }]
          : [{ label: $_("connection.ctxConnect"), icon: "→", action: () => { handleConnect(ctxMenu!.profileId); ctxMenu = null; } }]
        ),
        { label: $_("connection.ctxEdit"), icon: "✎", action: () => { handleEdit(ctxMenu!.profileId); ctxMenu = null; } },
        { label: $_("connection.ctxTest"), icon: "⚡", action: () => { const id = ctxMenu!.profileId; ctxMenu = null; handleTest(id); } },
        { label: $_("connection.ctxDelete"), icon: "🗑", action: () => { const id = ctxMenu!.profileId; ctxMenu = null; handleDelete(id); } },
      ]}
      onClose={() => { ctxMenu = null; }}
    />
  {/if}

  {#if blankCtx}
    <ContextMenu
      x={blankCtx.x}
      y={blankCtx.y}
      items={[
        { label: $_("connection.ctxNew"), icon: "+", action: () => { editingId = null; showForm = true; blankCtx = null; } },
      ]}
      onClose={() => { blankCtx = null; }}
    />
  {/if}

  {#if showSettings}
    <SettingsModal onClose={() => { showSettings = false; }} />
  {/if}
</aside>
