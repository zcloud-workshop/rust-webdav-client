<script lang="ts">
  import { getToasts, dismissToast } from "../../stores/toast.svelte";
  import { open } from "@tauri-apps/plugin-shell";
  import { X } from "lucide-svelte";
</script>

{#if getToasts().length > 0}
  <div class="fixed bottom-4 right-4 z-[100] flex flex-col gap-2">
    {#each getToasts() as toast (toast.id)}
      <div
        class="flex items-center gap-2 rounded-lg border px-4 py-2.5 text-sm shadow-lg {toast.type === 'success'
          ? 'border-green-500/30 bg-green-500/10 text-green-400'
          : toast.type === 'error'
            ? 'border-red-500/30 bg-red-500/10 text-red-400'
            : 'border-[var(--color-border)] bg-[var(--color-bg-primary)] text-[var(--color-text-primary)]'}"
      >
        <span class="flex-1">{toast.message}</span>
        {#if toast.link}
          <button
            class="shrink-0 rounded px-1.5 py-0.5 text-xs font-medium text-[var(--color-accent)] hover:underline"
            onclick={() => { open(toast.link!); dismissToast(toast.id); }}
          >{toast.linkLabel}</button>
        {/if}
        <button
          class="shrink-0 opacity-60 hover:opacity-100"
          onclick={() => dismissToast(toast.id)}
        >
          <X class="h-4 w-4" />
        </button>
      </div>
    {/each}
  </div>
{/if}
