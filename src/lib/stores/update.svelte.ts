import { showToast } from "./toast.svelte";
import { t } from "svelte-i18n";
import { get } from "svelte/store";
import { getVersion } from "./version";

const CURRENT_VERSION = getVersion();
const RELEASES_API = "https://api.github.com/repos/zcloud-workshop/rust-webdav-client/releases/latest";
const DOWNLOAD_URL = "https://github.com/zcloud-workshop/rust-webdav-client/releases/latest";

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
  return val !== "false";
}

export function setAutoCheck(value: boolean) {
  localStorage.setItem(STORAGE_KEY, String(value));
}

export async function checkForUpdate() {
  if (!getAutoCheck()) return;
  await doCheckUpdate();
}

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
