/**
 * 应用入口文件
 * 初始化 Svelte 应用并挂载到 DOM
 */

import "./app.css";
import App from "./App.svelte";
import { mount } from "svelte";
import { initLocale, waitLocale } from "./lib/i18n";
import { initTheme } from "./lib/stores/theme.svelte";
import { checkForUpdate } from "./lib/stores/update.svelte";
import { loadProfiles } from "./lib/stores/connections.svelte";
import { api } from "./lib/api";

async function main() {
  initTheme();

  await initLocale();
  await waitLocale();

  await loadProfiles();
  // 自动重新挂载所有已保存的映射，失败不阻塞启动
  api.mount.autoMount().catch(() => {});

  checkForUpdate();

  const app = mount(App, {
    target: document.getElementById("app")!,
  });

  return app;
}

export default main();
