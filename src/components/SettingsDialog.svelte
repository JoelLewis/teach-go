<script lang="ts">
  import type { Settings } from "../lib/api/types";
  import { llmStore } from "../lib/stores/llm.svelte";
  import { onMount } from "svelte";

  type Props = {
    settings: Settings;
    onClose: () => void;
    onSave: (settings: Settings) => void;
  };

  let { settings, onClose, onSave }: Props = $props();

  let boardSize = $state(settings.board_size);
  let showCoordinates = $state(settings.show_coordinates);
  let aiStrength = $state(settings.ai_strength);
  let soundEnabled = $state(settings.sound_enabled);
  let feedbackTiming = $state(settings.feedback_timing);

  onMount(() => {
    llmStore.refresh();
    return () => llmStore.cleanup();
  });

  function handleSave() {
    onSave({
      ...settings,
      board_size: boardSize,
      show_coordinates: showCoordinates,
      ai_strength: aiStrength,
      sound_enabled: soundEnabled,
      feedback_timing: feedbackTiming,
    });
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
  <div class="w-80 rounded-lg bg-stone-800 p-6 shadow-xl">
    <h2 class="mb-4 text-lg font-semibold text-stone-100">Settings</h2>

    <div class="mb-4">
      <label class="mb-1 block text-sm text-stone-400">Board Size</label>
      <select
        bind:value={boardSize}
        class="w-full rounded bg-stone-700 px-3 py-2 text-stone-100"
      >
        <option value={9}>9x9</option>
        <option value={13}>13x13</option>
        <option value={19}>19x19</option>
      </select>
    </div>

    <div class="mb-4">
      <label class="mb-1 block text-sm text-stone-400">AI Strength</label>
      <select
        bind:value={aiStrength}
        class="w-full rounded bg-stone-700 px-3 py-2 text-stone-100"
      >
        <option value="beginner">Beginner (25-20 kyu)</option>
        <option value="intermediate">Intermediate (19-10 kyu)</option>
        <option value="advanced">Advanced (9-1 kyu)</option>
        <option value="dan">Dan (1 dan+)</option>
      </select>
    </div>

    <div class="mb-4">
      <label class="flex items-center gap-2 text-sm text-stone-400">
        <input type="checkbox" bind:checked={showCoordinates} class="rounded" />
        Show coordinates
      </label>
    </div>

    <div class="mb-4">
      <label class="flex items-center gap-2 text-sm text-stone-400">
        <input type="checkbox" bind:checked={soundEnabled} class="rounded" />
        Sound effects
      </label>
    </div>

    <div class="mb-6">
      <label class="mb-1 block text-sm text-stone-400">Coaching Feedback</label>
      <select
        bind:value={feedbackTiming}
        class="w-full rounded bg-stone-700 px-3 py-2 text-stone-100"
      >
        <option value="immediate">Immediate (show after each move)</option>
        <option value="on_demand">On demand (click to reveal)</option>
        <option value="post_game">Post-game only (review only)</option>
      </select>
    </div>

    {#if llmStore.status !== "disabled"}
      <div class="mb-6 rounded bg-stone-700/50 p-3">
        <label class="mb-2 block text-sm font-medium text-stone-300"
          >AI Coach Model</label
        >
        {#if llmStore.status === "ready"}
          <div class="flex items-center gap-2 text-sm text-emerald-400">
            <span class="inline-block h-2 w-2 rounded-full bg-emerald-400"
            ></span>
            Model loaded
          </div>
        {:else if llmStore.status === "loading"}
          <div class="space-y-2">
            <div class="text-sm text-amber-400">
              {llmStore.downloadProgress
                ? "Downloading model..."
                : "Loading model..."}
            </div>
            {#if llmStore.downloadProgress}
              <div class="h-2 w-full overflow-hidden rounded-full bg-stone-600">
                <div
                  class="h-full rounded-full bg-amber-500 transition-all"
                  style="width: {llmStore.downloadPercent}%"
                ></div>
              </div>
              <div class="text-xs text-stone-400">
                {llmStore.downloadPercent}% ({Math.round(
                  llmStore.downloadProgress.downloaded / 1048576,
                )} / {Math.round(llmStore.downloadProgress.total / 1048576)} MB)
              </div>
            {/if}
          </div>
        {:else}
          <div class="space-y-2">
            <p class="text-xs text-stone-400">
              Download Gemma 3 1B for enhanced coaching explanations (~2 GB).
            </p>
            {#if llmStore.error}
              <p class="text-xs text-red-400">{llmStore.error}</p>
            {/if}
            <button
              onclick={() => llmStore.startDownload()}
              class="rounded bg-teal-700 px-3 py-1.5 text-sm text-stone-100 hover:bg-teal-600"
            >
              Download AI Coach Model
            </button>
          </div>
        {/if}
      </div>
    {/if}

    <div class="flex justify-end gap-2">
      <button
        onclick={onClose}
        class="rounded bg-stone-700 px-4 py-2 text-sm text-stone-100 hover:bg-stone-600"
      >
        Cancel
      </button>
      <button
        onclick={handleSave}
        class="rounded bg-amber-700 px-4 py-2 text-sm text-stone-100 hover:bg-amber-600"
      >
        Save
      </button>
    </div>
  </div>
</div>
