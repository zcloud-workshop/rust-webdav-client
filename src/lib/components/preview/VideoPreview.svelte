<script lang="ts">
  import { Play, Pause, VolumeX, Volume1, Volume2, Maximize } from "lucide-svelte";

  let {
    videoSrc,
    fileName,
  } = $props<{
    videoSrc: string | null;
    fileName: string;
  }>();

  let videoEl: HTMLVideoElement | undefined = $state();
  let containerEl: HTMLDivElement | undefined = $state();
  let progressEl: HTMLDivElement | undefined = $state();

  let isPlaying = $state(false);
  let isVideoLoading = $state(true);
  let playError = $state<string | null>(null);
  let currentTime = $state(0);
  let duration = $state(0);
  let volume = $state(1);
  let isMuted = $state(false);
  let isDragging = $state(false);
  let showControls = $state(true);
  let controlsTimeout: ReturnType<typeof setTimeout> | undefined;

  function formatTime(s: number): string {
    if (!isFinite(s)) return "0:00";
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = Math.floor(s % 60);
    return h > 0
      ? `${h}:${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`
      : `${m}:${String(sec).padStart(2, "0")}`;
  }

  let progress = $derived(duration > 0 ? (currentTime / duration) * 100 : 0);

  function resetControlsTimer() {
    showControls = true;
    clearTimeout(controlsTimeout);
    if (isPlaying) {
      controlsTimeout = setTimeout(() => {
        showControls = false;
      }, 3000);
    }
  }

  function handleMouseMove() {
    resetControlsTimer();
  }

  function togglePlay() {
    if (!videoEl) return;
    if (videoEl.paused) {
      videoEl.play().catch(() => {});
    } else {
      videoEl.pause();
    }
  }

  function handleProgressClick(e: MouseEvent) {
    if (!videoEl || !progressEl || !videoEl.duration) return;
    const rect = progressEl.getBoundingClientRect();
    const pct = Math.max(0, Math.min((e.clientX - rect.left) / rect.width, 1));
    videoEl.currentTime = pct * videoEl.duration;
  }

  function handleThumbMouseDown(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    isDragging = true;

    const onMove = (ev: MouseEvent) => {
      if (!videoEl || !progressEl || !videoEl.duration) return;
      const rect = progressEl.getBoundingClientRect();
      const pct = Math.max(0, Math.min((ev.clientX - rect.left) / rect.width, 1));
      videoEl.currentTime = pct * videoEl.duration;
    };

    const onUp = () => {
      isDragging = false;
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
    };

    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  }

  function toggleMute() {
    if (!videoEl) return;
    videoEl.muted = !videoEl.muted;
  }

  function handleVolumeInput(e: Event) {
    if (!videoEl) return;
    const val = parseFloat((e.target as HTMLInputElement).value);
    videoEl.volume = val;
  }

  function toggleFullscreen() {
    if (!containerEl) return;
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      containerEl.requestFullscreen();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === " " || e.key === "Space") {
      e.preventDefault();
      togglePlay();
    }
  }

  $effect(() => {
    if (videoSrc) {
      isVideoLoading = true;
      isPlaying = false;
      currentTime = 0;
      duration = 0;
      playError = null;
    }
  });

  $effect(() => {
    const el = videoEl;
    if (!el || !videoSrc) return;

    el.addEventListener("timeupdate", () => {
      if (!isDragging) currentTime = el.currentTime;
    });
    el.addEventListener("loadedmetadata", () => {
      duration = el.duration;
      isVideoLoading = false;
    });
    el.addEventListener("waiting", () => {
      isVideoLoading = true;
    });
    el.addEventListener("canplay", () => {
      isVideoLoading = false;
    });
    el.addEventListener("seeking", () => {
      isVideoLoading = true;
    });
    el.addEventListener("seeked", () => {
      isVideoLoading = false;
    });
    el.addEventListener("play", () => {
      isPlaying = true;
      resetControlsTimer();
    });
    el.addEventListener("pause", () => {
      isPlaying = false;
      showControls = true;
    });
    el.addEventListener("ended", () => {
      isPlaying = false;
    });
    el.addEventListener("error", () => {
      isVideoLoading = false;
      const err = el.error;
      if (err) {
        playError =
          err.code === 3
            ? "Video decoding error"
            : err.code === 4
              ? "Video format not supported"
              : `Playback error (code ${err.code})`;
      } else {
        playError = "Unknown playback error";
      }
    });
    el.addEventListener("volumechange", () => {
      volume = el.volume;
      isMuted = el.muted;
    });

    return () => {
      el.pause();
      el.removeAttribute("src");
      el.load();
    };
  });

  $effect(() => {
    function onFSChange() {
      if (!document.fullscreenElement) showControls = true;
    }
    document.addEventListener("fullscreenchange", onFSChange);
    return () => document.removeEventListener("fullscreenchange", onFSChange);
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  bind:this={containerEl}
  class="group relative flex h-full w-full items-center justify-center bg-black"
  onmousemove={handleMouseMove}
  role="application"
  aria-label="Video player"
>
  <video
    bind:this={videoEl}
    class="max-h-full max-w-full"
    preload="auto"
    playsinline
    src={videoSrc}
    onclick={togglePlay}
  >
    <track kind="captions" />
  </video>

  {#if playError}
    <div class="absolute left-1/2 top-1/2 z-10 -translate-x-1/2 -translate-y-1/2 text-center">
      <p class="text-sm text-red-400">{playError}</p>
    </div>
  {:else if isVideoLoading}
    <div class="absolute left-1/2 top-1/2 z-10 -translate-x-1/2 -translate-y-1/2 rounded-full bg-black/60 p-4">
      <div class="h-8 w-8 animate-spin rounded-full border-2 border-white/30 border-t-white"></div>
    </div>
  {:else if !isPlaying}
    <button
      class="absolute left-1/2 top-1/2 z-10 -translate-x-1/2 -translate-y-1/2 rounded-full bg-black/60 p-4 text-white transition-transform hover:scale-110"
      onclick={togglePlay}
      aria-label="Play"
    >
      <Play class="h-8 w-8" />
    </button>
  {/if}

  <div
    class="absolute bottom-0 left-0 right-0 transition-opacity duration-300"
    class:opacity-0={!showControls}
    class:opacity-100={showControls}
  >
    <div
      bind:this={progressEl}
      class="group/progress relative mx-3 h-1.5 cursor-pointer rounded-full bg-white/30 hover:h-2"
      role="slider"
      aria-label="Seek"
      aria-valuemin={0}
      aria-valuemax={duration}
      aria-valuenow={currentTime}
      tabindex="0"
      onmousedown={handleProgressClick}
    >
      <div
        class="pointer-events-none h-full rounded-full bg-[var(--color-accent)] transition-all duration-100"
        style="width: {progress}%"
      ></div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="absolute top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-[var(--color-accent)] transition-all group-hover/progress:scale-150"
        style="left: {progress}%; width: 12px; height: 12px;"
        onmousedown={handleThumbMouseDown}
      ></div>
    </div>

    <div
      class="flex items-center gap-2 bg-gradient-to-t from-black/80 to-transparent px-3 pb-2 pt-6"
    >
      <button
        class="rounded p-1 text-white hover:bg-white/20"
        onclick={togglePlay}
        aria-label={isPlaying ? "Pause" : "Play"}
      >
        {#if isPlaying}
          <Pause class="h-5 w-5" />
        {:else}
          <Play class="h-5 w-5" />
        {/if}
      </button>

      <span class="min-w-[80px] text-xs text-white/80">
        {formatTime(currentTime)} / {formatTime(duration)}
      </span>

      <div class="flex-1"></div>

      <div class="flex items-center gap-1">
        <button
          class="rounded p-1 text-white hover:bg-white/20"
          onclick={toggleMute}
          aria-label={isMuted ? "Unmute" : "Mute"}
        >
          {#if isMuted || volume === 0}
            <VolumeX class="h-4 w-4" />
          {:else if volume < 0.5}
            <Volume1 class="h-4 w-4" />
          {:else}
            <Volume2 class="h-4 w-4" />
          {/if}
        </button>
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          value={isMuted ? 0 : volume}
          oninput={handleVolumeInput}
          class="w-16 accent-[var(--color-accent)]"
          aria-label="Volume"
        />
      </div>

      <button
        class="rounded p-1 text-white hover:bg-white/20"
        onclick={toggleFullscreen}
        aria-label="Fullscreen"
      >
        <Maximize class="h-4 w-4" />
      </button>
    </div>
  </div>
</div>
