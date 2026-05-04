<script lang="ts">
  import { _ } from "svelte-i18n";
  import { FolderOpen } from "lucide-svelte";
</script>

<div class="flex flex-1 items-center justify-center">
  <div class="text-center">
    <FolderOpen class="mx-auto h-16 w-16 text-[var(--color-text-secondary)]/30" strokeWidth={1.5} />
    <p class="mt-4 text-lg font-medium text-[var(--color-text-secondary)]">{$_("empty.title")}</p>
    <p class="mt-1 text-sm text-[var(--color-text-secondary)]/60">{$_("empty.subtitle")}</p>
  </div>
</div>
