<!-- 应用根组件 - 整体布局容器 -->
<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { _ } from "svelte-i18n";
  import { showConfirm } from "./lib/stores/dialog.svelte";
  import Sidebar from "./lib/components/layout/Sidebar.svelte";
  import MainContent from "./lib/components/layout/MainContent.svelte";
  import ConfirmDialog from "./lib/components/common/ConfirmDialog.svelte";
  import ToastContainer from "./lib/components/common/ToastContainer.svelte";

  /** 是否已连接到 WebDAV 服务器 */
  let connected = $state(false);

  const MIN_WIDTH = 200;
  const MAX_WIDTH = 400;
  const COLLAPSED_WIDTH = 48;
  const DEFAULT_WIDTH = 256;

  let sidebarWidth = $state(DEFAULT_WIDTH);
  let collapsed = $state(false);
  let dragging = $state(false);

  function effectiveWidth() {
    return collapsed ? COLLAPSED_WIDTH : sidebarWidth;
  }

  function toggleSidebar() {
    collapsed = !collapsed;
  }

  function startDrag(e: MouseEvent) {
    if (collapsed) return;
    e.preventDefault();
    dragging = true;
    const startX = e.clientX;
    const startW = sidebarWidth;

    function onMove(ev: MouseEvent) {
      const delta = ev.clientX - startX;
      sidebarWidth = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, startW + delta));
    }
    function onUp() {
      dragging = false;
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
    }
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  }

  onMount(() => {
    const unlisten = getCurrentWindow().listen("close-requested", async () => {
      const confirmed = await showConfirm(
        $_("app.quitConfirm"),
        $_("dialog.confirmTitle"),
        $_("app.quit"),
        $_("app.quitCancel"),
      );
      if (confirmed) {
        await invoke("confirm_exit");
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

<!-- 全屏应用容器 - 侧边栏 + 主内容区布局 -->
<div class="flex h-screen w-screen overflow-hidden bg-[var(--color-bg-primary)]">

  <div
    class="shrink-0 overflow-hidden {collapsed ? '' : ''}"
    style="width: {effectiveWidth()}px; transition: width {dragging ? '0s' : '0.2s'} ease;"
  >
    <Sidebar bind:connected {collapsed} onToggle={toggleSidebar} />
  </div>

  {#if !collapsed}
    <div
      class="group relative z-10 flex w-1 shrink-0 cursor-col-resize items-center justify-center bg-transparent hover:bg-[var(--color-accent)]/20"
      role="separator"
      onmousedown={startDrag}
    >
      <div class="h-8 w-0.5 rounded-full bg-[var(--color-border)] group-hover:bg-[var(--color-accent)]"></div>
    </div>
  {/if}

  <MainContent {connected} />
</div>

<!-- 全局确认对话框 -->
<ConfirmDialog />

<!-- 全局 Toast 通知 -->
<ToastContainer />
