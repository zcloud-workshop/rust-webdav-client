<script lang="ts">
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
      <svg class="h-8 w-8" viewBox="0 0 24 24" fill="currentColor">
        <path d="M8 5v14l11-7z" />
      </svg>
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
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z" />
          </svg>
        {:else}
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M8 5v14l11-7z" />
          </svg>
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
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51A8.796 8.796 0 0021 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06a8.99 8.99 0 003.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z"
              />
            </svg>
          {:else if volume < 0.5}
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M18.5 12c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM5 9v6h4l5 5V4L9 9H5z"
              />
            </svg>
          {:else}
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"
              />
            </svg>
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
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z"
          />
        </svg>
      </button>
    </div>
  </div>
</div>
