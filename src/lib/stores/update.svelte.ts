import { showToast } from "./toast.svelte";
import { t } from "svelte-i18n";
// svelte-i18n 使用 Svelte 4 store，在 rune 模式的 .svelte.ts 文件中不能使用 $t 自动订阅语法
import { get } from "svelte/store";
import { getVersion } from "./version";

const CURRENT_VERSION = getVersion();
const RELEASES_API = "https://api.github.com/repos/zcloud-workshop/rust-webdav-client/releases/latest";
const DOWNLOAD_URL = "https://github.com/zcloud-workshop/rust-webdav-client/releases/latest";

// 仅支持 MAJOR.MINOR.PATCH 格式。预发布标签（如 v2.0.0-rc1）会导致 NaN 比较，视为等于当前版本。
// 假设 GitHub latest release 始终是正式版标签。
function parseSemver(tag: string): number[] {
  return tag.replace(/^v/, "").split(".").map(Number);
}

function isNewer(remote: string, local: string): boolean {
  const r = parseSemver(remote);
  const l = parseSemver(local);
  for (let i = 0; i < 3; i++) {
    if ((r[i] ?? 0) > (l[i] ?? 0)) return true;
    if ((r[i] ?? 0) < (l[i] ?? 0)) return false;
  }
  return false;
}

const STORAGE_KEY = "autoCheckUpdate";

export function getAutoCheck(): boolean {
  const val = localStorage.getItem(STORAGE_KEY);
  // 默认开启（opt-out）：只有明确存了 "false" 才关闭。未设置的 key 返回 true。
  return val !== "false";
}

export function setAutoCheck(value: boolean) {
  localStorage.setItem(STORAGE_KEY, String(value));
}

// 启动时静默检查：仅在发现新版本时弹通知，不打扰用户
export async function checkForUpdate() {
  if (!getAutoCheck()) return;
  await doCheckUpdate();
}

// 手动检查（设置页面按钮）：显示所有结果（成功/失败/已是最新）
export async function checkForUpdateNow() {
  try {
    const res = await fetch(RELEASES_API);
    if (!res.ok) {
      showToast(get(t)("update.checkFailed"), "error");
      return;
    }
    const data = await res.json();
    const tag = data.tag_name as string | undefined;
    if (tag && isNewer(tag, CURRENT_VERSION)) {
      // duration=0：通知不会自动消失，用户需要手动关闭
      showToast(get(t)("update.available", { values: { version: tag } }), "info", 0, DOWNLOAD_URL, get(t)("update.download"));
    } else {
      showToast(get(t)("update.upToDate"), "success");
    }
  } catch {
    showToast(get(t)("update.checkFailed"), "error");
  }
}

async function doCheckUpdate() {
  try {
    const res = await fetch(RELEASES_API);
    if (!res.ok) return;
    const data = await res.json();
    const tag = data.tag_name as string | undefined;
    if (!tag || !isNewer(tag, CURRENT_VERSION)) return;
    showToast(get(t)("update.available", { values: { version: tag } }), "info", 0, DOWNLOAD_URL, get(t)("update.download"));
  } catch {
    // silent fail
  }
}
