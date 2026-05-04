<!-- 自定义窗口标题栏 - 无边框窗口的拖拽区域和窗口控制按钮 -->
<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { _ } from "svelte-i18n";
  import { Minus, Square, X, Copy } from "lucide-svelte";

  let isMaximized = $state(false);

  async function handleMinimize() {
    await getCurrentWindow().minimize();
  }

  async function handleMaximize() {
    await getCurrentWindow().toggleMaximize();
  }

  async function handleClose() {
    await getCurrentWindow().close();
  }

  onMount(() => {
    const win = getCurrentWindow();
    win.isMaximized().then((v) => (isMaximized = v));
    win.onResized(async () => {
      isMaximized = await win.isMaximized();
    });
  });
</script>

<div
  class="flex h-9 shrink-0 select-none items-center border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)]"
>
  <!-- 左侧：图标 + 标题 -->
  <div class="flex items-center gap-2 pl-3 pr-2">
    <img src="/logo.png" alt="" class="h-4 w-4" />
    <span class="text-xs font-medium text-[var(--color-text-secondary)]">{$_("app.title")}</span>
  </div>

  <!-- 中间：拖拽区域 -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex-1 h-full"
    data-tauri-drag-region
    ondblclick={handleMaximize}
  ></div>

  <!-- 右侧：窗口控制按钮 -->
  <div class="flex h-full">
    <button
      class="flex h-full w-11 items-center justify-center text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-primary)]"
      onclick={handleMinimize}
      title={$_("titlebar.minimize")}
    >
      <Minus class="h-4 w-4" />
    </button>
    <button
      class="flex h-full w-11 items-center justify-center text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-primary)]"
      onclick={handleMaximize}
      title={isMaximized ? $_("titlebar.restore") : $_("titlebar.maximize")}
    >
      {#if isMaximized}
        <Copy class="h-3.5 w-3.5" />
      {:else}
        <Square class="h-3.5 w-3.5" />
      {/if}
    </button>
    <button
      class="flex h-full w-11 items-center justify-center text-[var(--color-text-secondary)] hover:bg-red-500 hover:text-white"
      onclick={handleClose}
      title={$_("titlebar.close")}
    >
      <X class="h-4 w-4" />
    </button>
  </div>
</div>
