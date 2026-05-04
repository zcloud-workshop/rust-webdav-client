/**
 * Toast 通知消息状态管理
 */

export interface Toast {
  id: number;
  message: string;
  type: "success" | "error" | "info";
  link?: string;
  linkLabel?: string;
}

let toasts = $state<Toast[]>([]);
let nextId = 0;

export function getToasts() {
  return toasts;
}

export function showToast(
  message: string,
  type: Toast["type"] = "info",
  duration = 3000,
  link?: string,
  linkLabel?: string,
) {
  const id = nextId++;
  // Svelte 5 rune 模式：$state 仅追踪变量赋值，不追踪数组 mutation。
  // 展开运算符创建新数组引用以触发响应式更新。
  toasts = [...toasts, { id, message, type, link, linkLabel }];
  // duration=0 表示持久通知（不自动消失），用于更新提醒等需要用户手动关闭的场景
  if (duration > 0) {
    setTimeout(() => {
      dismissToast(id);
    }, duration);
  }
}

export function dismissToast(id: number) {
  toasts = toasts.filter((t) => t.id !== id);
}
