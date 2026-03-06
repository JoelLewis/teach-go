<script lang="ts">
  import { setupStore } from "../lib/stores/setup.svelte";
  import { onDestroy } from "svelte";

  type Props = {
    onComplete: () => void;
    onSkip: () => void;
  };

  let { onComplete, onSkip }: Props = $props();

  let downloading = $state(false);

  async function startDownload() {
    downloading = true;
    await setupStore.startSetup();
    downloading = false;
    if (setupStore.status === "ready") {
      onComplete();
    }
  }

  function formatMB(bytes: number): string {
    return (bytes / 1_048_576).toFixed(1);
  }

  onDestroy(() => setupStore.cleanup());
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onSkip}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="w-96 rounded-lg p-6 shadow-xl" style="background-color: var(--panel-bg, #292524); color: var(--text-primary, #f5f5f4);" role="dialog" aria-modal="true" tabindex="-1" onkeydown={(e) => { if (e.key === 'Escape') onSkip(); }} onclick={(e) => e.stopPropagation()}>
    {#if setupStore.error && !downloading}
      <!-- Error state -->
      <h2 class="mb-3 text-lg font-semibold" style="color: var(--danger, #ef4444);">Download Failed</h2>
      <p class="mb-4 text-sm" style="color: var(--text-secondary, #a8a29e);">{setupStore.error}</p>
      <div class="flex justify-end gap-2">
        <button
          onclick={onSkip}
          class="rounded px-4 py-2 text-sm hover:opacity-80"
          style="background-color: var(--surface-secondary, #44403c); color: var(--text-secondary, #a8a29e);"
        >
          Skip for now
        </button>
        <button
          onclick={startDownload}
          class="rounded px-4 py-2 text-sm font-semibold hover:opacity-90"
          style="background-color: var(--accent-primary, #c9a84c); color: var(--surface-primary, #1c1917);"
        >
          Retry
        </button>
      </div>

    {:else if downloading && setupStore.progress}
      <!-- Downloading state -->
      <h2 class="mb-3 text-lg font-semibold">{setupStore.phaseLabel}</h2>
      <div class="mb-2 h-2 w-full overflow-hidden rounded" style="background-color: var(--surface-secondary, #44403c);">
        <div
          class="h-full rounded transition-all duration-300"
          style="width: {setupStore.downloadPercent}%; background-color: var(--accent-primary, #c9a84c);"
        ></div>
      </div>
      <div class="text-xs" style="color: var(--text-dim, #78716c);">
        {formatMB(setupStore.progress.downloaded)} / {setupStore.progress.total > 0 ? formatMB(setupStore.progress.total) : "?"} MB
        ({setupStore.downloadPercent}%)
      </div>

    {:else}
      <!-- Initial state -->
      <h2 class="mb-3 text-lg font-semibold">KataGo Required</h2>
      <p class="mb-4 text-sm" style="color: var(--text-secondary, #a8a29e);">
        GoSensei needs the KataGo AI engine to play against you and review your games. This is a one-time download (~145 MB).
      </p>
      <div class="flex justify-end gap-2">
        <button
          onclick={onSkip}
          class="rounded px-4 py-2 text-sm hover:opacity-80"
          style="background-color: var(--surface-secondary, #44403c); color: var(--text-secondary, #a8a29e);"
        >
          Skip for now
        </button>
        <button
          onclick={startDownload}
          class="rounded px-4 py-2 text-sm font-semibold hover:opacity-90"
          style="background-color: var(--accent-primary, #c9a84c); color: var(--surface-primary, #1c1917);"
        >
          Download KataGo
        </button>
      </div>
    {/if}
  </div>
</div>
