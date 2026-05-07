<script lang="ts">
  import { _, locale } from "svelte-i18n";
  import { open } from "@tauri-apps/plugin-shell";
  import { setLocale } from "../../i18n";
  import { getTheme, setTheme, type Theme } from "../../stores/theme.svelte";
  import { getAutoCheck, setAutoCheck, checkForUpdateNow } from "../../stores/update.svelte";
  import { isEnabled as isAutostartEnabled, enable as enableAutostart, disable as disableAutostart } from "@tauri-apps/plugin-autostart";
  import { X, ExternalLink, Sun, Moon, Monitor } from "lucide-svelte";
  import { getVersion } from "../../stores/version";

  let { onClose } = $props<{ onClose: () => void }>();

  const GITHUB_URL = "https://github.com/zcloud-workshop/rust-webdav-client";
  const LICENSE_URL = "https://raw.githubusercontent.com/zcloud-workshop/rust-webdav-client/refs/heads/main/License";

  const locales = [
    { code: "en", label: "English" },
    { code: "zh-CN", label: "简体中文" },
    { code: "zh-TW", label: "繁體中文" },
    { code: "ja", label: "日本語" },
    { code: "ko", label: "한국어" },
    { code: "de", label: "Deutsch" },
    { code: "ru", label: "Русский" },
  ];
  let theme = $state<Theme>(getTheme());
  let autoCheck = $state(getAutoCheck());
  let closeToTray = $state(localStorage.getItem("minimizeOnClose") !== "false");
  let autoStart = $state(false);
  let showLicense = $state(false);
  let licenseText = $state("");
  let licenseLoading = $state(false);

  function handleLocale(next: string) {
    setLocale(next);
  }

  function handleTheme(next: Theme) {
    theme = next;
    setTheme(next);
  }

  function handleAutoCheck() {
    autoCheck = !autoCheck;
    setAutoCheck(autoCheck);
  }

  isAutostartEnabled().then((v) => (autoStart = v)).catch(() => {});

  function handleAutoStart() {
    autoStart = !autoStart;
    if (autoStart) enableAutostart(); else disableAutostart();
  }

  async function openLicense() {
    showLicense = true;
    if (licenseText) return;
    licenseLoading = true;
    try {
      const res = await fetch(LICENSE_URL);
      if (res.ok) {
        licenseText = await res.text();
      } else {
        licenseText = "Failed to load license.";
      }
    } catch {
      licenseText = "Failed to load license.";
    } finally {
      licenseLoading = false;
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onClose}>
  <div
    class="w-full max-w-sm rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] shadow-xl"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-3">
      <h3 class="text-sm font-semibold text-[var(--color-text-primary)]">{$_("settings.title")}</h3>
      <button
        class="rounded-md p-1 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
        onclick={onClose}
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <div class="space-y-4 p-5">
      <!-- Language -->
      <div class="flex items-center justify-between">
        <span class="text-sm text-[var(--color-text-primary)]">{$_("settings.language")}</span>
        <select
          class="rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-2 py-1 text-xs text-[var(--color-text-primary)] outline-none focus:border-[var(--color-accent)]"
          value={$locale}
          onchange={(e) => handleLocale((e.target as HTMLSelectElement).value)}
        >
          {#each locales as loc}
            <option value={loc.code}>{loc.label}</option>
          {/each}
        </select>
      </div>

      <!-- Theme -->
      <div class="flex items-center justify-between">
        <span class="text-sm text-[var(--color-text-primary)]">{$_("settings.theme")}</span>
        <div class="flex rounded-md border border-[var(--color-border)] overflow-hidden">
          <button
            class="px-3 py-1 text-xs {theme === 'light'
              ? 'bg-[var(--color-accent)] text-white'
              : 'bg-[var(--color-bg-primary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-accent)]/10'}"
            onclick={() => handleTheme("light")}
          ><Sun class="h-3.5 w-3.5" /></button>
          <button
            class="px-3 py-1 text-xs {theme === 'dark'
              ? 'bg-[var(--color-accent)] text-white'
              : 'bg-[var(--color-bg-primary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-accent)]/10'}"
            onclick={() => handleTheme("dark")}
          ><Moon class="h-3.5 w-3.5" /></button>
          <button
            class="px-3 py-1 text-xs {theme === 'auto'
              ? 'bg-[var(--color-accent)] text-white'
              : 'bg-[var(--color-bg-primary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-accent)]/10'}"
            onclick={() => handleTheme("auto")}
          ><Monitor class="h-3.5 w-3.5" /></button>
        </div>
      </div>

      <!-- Auto check update -->
      <div class="flex items-center justify-between">
        <span class="text-sm text-[var(--color-text-primary)]">{$_("settings.autoUpdate")}</span>
        <div class="flex items-center gap-2">
          <button
            class="rounded-md border border-[var(--color-border)] px-2 py-0.5 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-accent)]/10 hover:text-[var(--color-accent)]"
            onclick={checkForUpdateNow}
          >
            {$_("settings.checkNow")}
          </button>
          <!-- svelte-ignore a11y_consider_explicit_label -->
          <button
            class="relative h-5 w-9 rounded-full transition-colors {autoCheck
              ? 'bg-[var(--color-accent)]'
              : 'bg-[var(--color-border)]'}"
            onclick={handleAutoCheck}
          >
            <span
              class="absolute top-0.5 h-4 w-4 rounded-full bg-white shadow transition-transform {autoCheck
                ? 'left-[18px]'
                : 'left-0.5'}"
            ></span>
          </button>
        </div>
      </div>

      <!-- Close to tray -->
      <!-- svelte-ignore a11y_consider_explicit_label -->
      <div class="flex items-center justify-between">
        <span class="text-sm text-[var(--color-text-primary)]">{$_("settings.closeToTray")}</span>
        <button
          class="relative h-5 w-9 rounded-full transition-colors {closeToTray
            ? 'bg-[var(--color-accent)]'
            : 'bg-[var(--color-border)]'}"
          onclick={() => { closeToTray = !closeToTray; localStorage.setItem("minimizeOnClose", String(closeToTray)); }}
        >
          <span
            class="absolute top-0.5 h-4 w-4 rounded-full bg-white shadow transition-transform {closeToTray
              ? 'left-[18px]'
              : 'left-0.5'}"
          ></span>
        </button>
      </div>

      <!-- Auto start -->
      <!-- svelte-ignore a11y_consider_explicit_label -->
      <div class="flex items-center justify-between">
        <span class="text-sm text-[var(--color-text-primary)]">{$_("settings.autoStart")}</span>
        <button
          class="relative h-5 w-9 rounded-full transition-colors {autoStart
            ? 'bg-[var(--color-accent)]'
            : 'bg-[var(--color-border)]'}"
          onclick={handleAutoStart}
        >
          <span
            class="absolute top-0.5 h-4 w-4 rounded-full bg-white shadow transition-transform {autoStart
              ? 'left-[18px]'
              : 'left-0.5'}"
          ></span>
        </button>
      </div>

      <!-- About -->
      <div class="border-t border-[var(--color-border)] pt-4">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium text-[var(--color-text-primary)]">{$_("settings.about")}</span>
          <span class="text-xs text-[var(--color-text-secondary)]">v{getVersion()}</span>
        </div>
        <div class="mt-2 flex items-center gap-3">
          <button
            class="flex items-center gap-1 text-xs text-[var(--color-accent)] hover:underline"
            onclick={() => open(GITHUB_URL)}
          >
            <ExternalLink class="h-3 w-3" />
            GitHub
          </button>
          <button
            class="flex items-center gap-1 text-xs text-[var(--color-accent)] hover:underline"
            onclick={() => open(`${GITHUB_URL}/issues`)}
          >
            {$_("settings.feedback")}
          </button>
          <button
            class="flex items-center gap-1 text-xs text-[var(--color-accent)] hover:underline"
            onclick={openLicense}
          >
            {$_("settings.license")}
          </button>
        </div>
      </div>
    </div>

    <!-- License dialog -->
    {#if showLicense}
      <div class="fixed inset-0 z-[60] flex items-center justify-center bg-black/50" onclick={() => showLicense = false}>
        <div
          class="max-h-[80vh] w-full max-w-lg overflow-y-auto rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] p-5 text-sm text-[var(--color-text-primary)]"
          onclick={(e) => e.stopPropagation()}
        >
          <div class="flex items-center justify-between mb-3">
            <h4 class="text-sm font-semibold">{$_("settings.license")}</h4>
            <button class="rounded-md p-1 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]" onclick={() => showLicense = false}>
              <X class="h-4 w-4" />
            </button>
          </div>
          {#if licenseLoading}
            <div class="flex items-center justify-center py-8">
              <div class="h-5 w-5 animate-spin rounded-full border-2 border-[var(--color-accent)] border-t-transparent"></div>
            </div>
          {:else}
            <pre class="whitespace-pre-wrap text-xs leading-relaxed text-[var(--color-text-secondary)]">{licenseText}</pre>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
