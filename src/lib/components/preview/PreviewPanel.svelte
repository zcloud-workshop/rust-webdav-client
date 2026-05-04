<!-- 文件预览面板组件 - 根据文件类型渲染不同的预览器 -->
<script lang="ts">
  import { _ } from "svelte-i18n";
  import {
    getFileName,
    getFileType,
    getIsLoading,
    getIsEditing,
    getData,
    getError,
    getVideoSrc,
    close,
    setEditing,
  } from "../../stores/preview.svelte";
  import TextPreview from "./TextPreview.svelte";
  import ImagePreview from "./ImagePreview.svelte";
  import MediaPreview from "./MediaPreview.svelte";
  import VideoPreview from "./VideoPreview.svelte";
  import PdfPreview from "./PdfPreview.svelte";
  import OfficePreview from "./OfficePreview.svelte";
  import UnsupportedPreview from "./UnsupportedPreview.svelte";
  import { X } from "lucide-svelte";

  /** 键盘事件处理 - ESC 关闭预览 */
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- 全屏预览覆盖层 -->
<div class="absolute inset-0 z-10 flex flex-col bg-[var(--color-bg-primary)]">
  <!-- 顶部工具栏 - 关闭按钮、文件名、编辑按钮 -->
  <div class="flex shrink-0 items-center gap-3 border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-4 py-2">
    <button
      class="shrink-0 rounded-md p-1.5 text-[var(--color-text-secondary)] hover:bg-[var(--color-accent)]/10 hover:text-[var(--color-accent)]"
      onclick={close}
      title={$_("preview.close")}
    >
      <X class="h-4 w-4" />
    </button>
    <span class="flex-1 truncate text-sm font-medium">{getFileName()}</span>
    <!-- 文本文件显示编辑/预览切换按钮 -->
    {#if getFileType() === "text"}
      <button
        class="rounded-md px-3 py-1 text-xs {getIsEditing()
          ? 'bg-green-500 text-white hover:bg-green-600'
          : 'bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)]'}"
        onclick={() => setEditing(!getIsEditing())}
      >
        {getIsEditing() ? $_("preview.preview") : $_("preview.edit")}
      </button>
    {/if}
  </div>

  <!-- 预览内容区域 - 根据文件类型分发到不同的预览组件 -->
  <div class="flex-1 overflow-auto">
    <!-- 加载中状态 -->
    {#if getIsLoading()}
      <div class="flex items-center justify-center py-20">
        <div class="h-6 w-6 animate-spin rounded-full border-2 border-[var(--color-accent)] border-t-transparent"></div>
      </div>
    <!-- 错误状态 -->
    {:else if getError()}
      <div class="px-6 py-10 text-center text-red-500">{getError()}</div>
    <!-- 数据加载完成 - 按类型分发预览 -->
    {:else if getData()}
      {#if getFileType() === "text"}
        <TextPreview data={getData() as string} editing={getIsEditing()} />
      {:else if getFileType() === "image"}
        <ImagePreview data={getData() as ArrayBuffer} />
      {:else if getFileType() === "pdf"}
        <PdfPreview data={getData() as ArrayBuffer} />
      {:else if getFileType() === "audio"}
        <MediaPreview data={getData() as ArrayBuffer} type="audio" fileName={getFileName()!} />
      {:else if getFileType() === "video"}
        <VideoPreview videoSrc={getVideoSrc()} fileName={getFileName()!} />
      {:else if getFileType() === "docx" || getFileType() === "xlsx"}
        <OfficePreview data={getData() as ArrayBuffer} type={(getFileType() as "docx" | "xlsx")} />
      {:else}
        <UnsupportedPreview />
      {/if}
    {/if}
  </div>
</div>
