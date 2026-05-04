<!-- 通用确认对话框组件 -->
<script lang="ts">
  import { _ } from "svelte-i18n";
  import {
    getDialogState,
    handleConfirm,
    handleCancel,
  } from "../../stores/dialog.svelte";

  function handleKeydown(e: KeyboardEvent) {
    const state = getDialogState();
    if (!state.isOpen) return;

    if (e.key === "Enter") {
      e.preventDefault();
      handleConfirm();
    } else if (e.key === "Escape") {
      e.preventDefault();
      if (state.type === "confirm") {
        handleCancel();
      } else {
        handleConfirm();
      }
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.classList.contains("dialog-overlay")) {
      const state = getDialogState();
      if (state.type === "confirm") {
        handleCancel();
      } else {
        handleConfirm();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if getDialogState().isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="dialog-overlay fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    onclick={handleOverlayClick}
  >
    <div
      class="w-full max-w-md rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] shadow-xl"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- 标题 -->
      <div class="border-b border-[var(--color-border)] px-4 py-3">
        <h3 class="text-base font-medium text-[var(--color-text-primary)]">
          {getDialogState().title}
        </h3>
      </div>
      <!-- 消息内容 -->
      <div class="px-4 py-4">
        <p class="text-sm text-[var(--color-text-secondary)] whitespace-pre-wrap">
          {getDialogState().message}
        </p>
      </div>
      <!-- 按钮区域 -->
      <div class="flex justify-end gap-2 border-t border-[var(--color-border)] px-4 py-3">
        {#if getDialogState().type === "confirm"}
          <button
            class="rounded-md px-3 py-1.5 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-secondary)]"
            onclick={handleCancel}
          >
            {getDialogState().cancelText}
          </button>
        {/if}
        <button
          class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-sm text-white hover:bg-[var(--color-accent-hover)]"
          onclick={handleConfirm}
        >
          {getDialogState().confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}
