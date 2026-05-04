<script lang="ts">
  import { _ } from "svelte-i18n";
  import { Monitor } from "lucide-svelte";
</script>

<div class="flex h-full items-center justify-center">
  <div class="text-center">
    <Monitor class="mx-auto h-12 w-12 text-[var(--color-text-secondary)]/30" strokeWidth={1.5} />
    <p class="mt-3 text-sm text-[var(--color-text-secondary)]">
      {$_("preview.notAvailable")}
    </p>
    <p class="mt-1 text-xs text-[var(--color-text-secondary)]/60">
      {$_("preview.downloadHint")}
    </p>
  </div>
</div>
