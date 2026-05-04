<script lang="ts">
  import type { FileMetadata } from "../../types";
  import { getFileCategory, formatFileSize, formatDate, getFileIcon } from "../../utils/file-types";
  import { Folder, FileText } from "lucide-svelte";

  let { item, selected = false, onselect, ondblclick, oncheckbox, oncontextmenu, readonly = false } = $props<{
    item: FileMetadata;
    selected: boolean;
    onselect: () => void;
    ondblclick: () => void;
    oncheckbox: () => void;
    oncontextmenu: (e: MouseEvent) => void;
    readonly?: boolean;
  }>();

  const category = getFileCategory(item.name, item.content_type);
  const icon = getFileIcon(category, item.is_dir);

  function handleCheckboxClick(e: Event) {
    e.stopPropagation();
    oncheckbox();
  }
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<div
  class="grid {readonly ? 'grid-cols-[1fr_100px_160px]' : 'grid-cols-[auto_1fr_100px_160px]'} gap-4 border-b border-[var(--color-border)] px-4 py-2 text-left text-sm transition-colors items-center {selected
    ? 'bg-[var(--color-accent)]/10 text-[var(--color-accent)]'
    : 'text-[var(--color-text-primary)] hover:bg-[var(--color-bg-secondary)]'}"
  role="row"
  oncontextmenu={readonly ? undefined : oncontextmenu}
>
  <!-- 选择框 -->
  {#if !readonly}
    <div class="shrink-0">
      <input
        type="checkbox"
        bind:checked={selected}
        onclick={handleCheckboxClick}
      />
    </div>
  {/if}
  <!-- 文件图标和名称 -->
  <button class="flex items-center gap-2 truncate w-full text-left" onclick={onselect} ondblclick={ondblclick}>
    {#if item.is_dir}
      <Folder class="h-4 w-4 shrink-0 text-yellow-500" />
    {:else}
      <FileText class="h-4 w-4 shrink-0 text-[var(--color-text-secondary)]" />
    {/if}
    <span class="truncate">{item.name}</span>
  </button>
  <!-- 文件大小 -->
  <span class="text-right text-[var(--color-text-secondary)]">{item.is_dir ? "-" : formatFileSize(item.size)}</span>
  <!-- 修改时间 -->
  <span class="text-[var(--color-text-secondary)]">{formatDate(item.modified)}</span>
</div>
