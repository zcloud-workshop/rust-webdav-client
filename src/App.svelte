<!-- 应用根组件 - 整体布局容器 -->
<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { _ } from "svelte-i18n";
  import { showConfirm } from "./lib/stores/dialog.svelte";
  import Sidebar from "./lib/components/layout/Sidebar.svelte";
  import MainContent from "./lib/components/layout/MainContent.svelte";
  import TitleBar from "./lib/components/layout/TitleBar.svelte";
  import ConfirmDialog from "./lib/components/common/ConfirmDialog.svelte";
  import ToastContainer from "./lib/components/common/ToastContainer.svelte";

  /** 是否已连接到 WebDAV 服务器 */
  let connected = $state(false);
  /** 是否为 Windows 平台（需要自定义标题栏） */
  let isWindows = $state(false);

  const MIN_WIDTH = 200;
  const MAX_WIDTH = 400;
  // 必须与 Sidebar.svelte 收起态按钮尺寸保持同步（h-8 + padding）
  const COLLAPSED_WIDTH = 48;
  const DEFAULT_WIDTH = 256;

  let sidebarWidth = $state(DEFAULT_WIDTH);
  let collapsed = $state(false);
  // 拖拽时禁用 CSS transition，避免宽度更新滞后于鼠标
  let dragging = $state(false);

  function effectiveWidth() {
    return collapsed ? COLLAPSED_WIDTH : sidebarWidth;
  }

  function toggleSidebar() {
    collapsed = !collapsed;
  }

  // 监听器绑定在 document 而非拖拽手柄上，确保鼠标快速移动离开 4px 宽的手柄时不会「脱手」
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
    // Windows 上移除原生标题栏，使用自定义 TitleBar
    if (navigator.userAgent.includes("Windows")) {
      isWindows = true;
      getCurrentWindow().setDecorations(false);
    }

    // 退出确认流程：Rust 侧发射 close-requested → 前端弹确认框 → 确认后调用 confirm_exit
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
    // listen() 返回 Promise<UnlistenFn>，但 onMount cleanup 必须同步。
    // 使用 .then() 异步注销，窗口销毁时事件系统会自动清理
    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

<!-- 全屏应用容器 - 标题栏 + 侧边栏 + 主内容区布局 -->
<div class="flex flex-col h-screen w-screen overflow-hidden bg-[var(--color-bg-primary)]">
  {#if isWindows}
    <TitleBar />
  {/if}
  <div class="flex flex-1 overflow-hidden">

    <div
      class="shrink-0 overflow-hidden {collapsed ? '' : ''}"
      style="width: {effectiveWidth()}px; transition: width {dragging ? '0s' : '0.2s'} ease;"
    >
      <Sidebar bind:connected {collapsed} onToggle={toggleSidebar} />
    </div>

    {#if !collapsed}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
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
</div>

<!-- 全局确认对话框 -->
<ConfirmDialog />

<!-- 全局 Toast 通知 -->
<ToastContainer />
