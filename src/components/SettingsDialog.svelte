<script lang="ts">
  import type { Settings, ThemeName } from "../lib/api/types";
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
  let theme = $state<ThemeName>(settings.theme as ThemeName);

  let dialogEl: HTMLDivElement;

  onMount(() => {
    llmStore.refresh();

    const focusable = dialogEl?.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (!focusable?.length) return () => llmStore.cleanup();
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    first?.focus();
    function handleKeydown(e: KeyboardEvent) {
      if (e.key === "Escape") { onClose(); return; }
      if (e.key !== "Tab") return;
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last?.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
    }
    dialogEl.addEventListener("keydown", handleKeydown);
    return () => {
      dialogEl.removeEventListener("keydown", handleKeydown);
      llmStore.cleanup();
    };
  });

  function handleSave() {
    onSave({
      ...settings,
      board_size: boardSize,
      show_coordinates: showCoordinates,
      ai_strength: aiStrength,
      sound_enabled: soundEnabled,
      feedback_timing: feedbackTiming,
      theme,
    });
  }
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape') onClose(); }} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onClose}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div bind:this={dialogEl} class="w-80 rounded-lg p-6 shadow-xl" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} style="background-color: var(--surface-card);">

    <h2 class="mb-4 text-lg font-semibold" style="color: var(--text-heading);">Settings</h2>

    <div class="mb-4">
      <label class="mb-1 block text-sm" style="color: var(--text-secondary);">Board Size
        <select
          bind:value={boardSize}
          class="w-full rounded px-3 py-2"
          style="background-color: var(--surface-input); color: var(--text-heading);"
        >
          <option value={9}>9x9</option>
          <option value={13}>13x13</option>
          <option value={19}>19x19</option>
        </select>
      </label>
    </div>

    <div class="mb-4">
      <label class="mb-1 block text-sm" style="color: var(--text-secondary);">AI Strength
        <select
          bind:value={aiStrength}
          class="w-full rounded px-3 py-2"
          style="background-color: var(--surface-input); color: var(--text-heading);"
        >
          <option value="beginner">Beginner (25-20 kyu)</option>
          <option value="intermediate">Intermediate (19-10 kyu)</option>
          <option value="advanced">Advanced (9-1 kyu)</option>
          <option value="dan">Dan (1 dan+)</option>
        </select>
      </label>
    </div>

    <div class="mb-4">
      <label class="flex items-center gap-2 text-sm" style="color: var(--text-secondary);">
        <input type="checkbox" bind:checked={showCoordinates} class="rounded" />
        Show coordinates
      </label>
    </div>

    <div class="mb-4">
      <label class="flex items-center gap-2 text-sm" style="color: var(--text-secondary);">
        <input type="checkbox" bind:checked={soundEnabled} class="rounded" />
        Sound effects
      </label>
    </div>

    <div class="mb-4">
      <label class="mb-1 block text-sm" style="color: var(--text-secondary);">Theme
        <select
          bind:value={theme}
          class="w-full rounded px-3 py-2"
          style="background-color: var(--surface-input); color: var(--text-heading);"
        >
          <option value="study">Study (warm wood)</option>
          <option value="grid">Grid (deep ink)</option>
        </select>
      </label>
    </div>

    <div class="mb-6">
      <label class="mb-1 block text-sm" style="color: var(--text-secondary);">Coaching Feedback
        <select
          bind:value={feedbackTiming}
          class="w-full rounded px-3 py-2"
          style="background-color: var(--surface-input); color: var(--text-heading);"
        >
          <option value="immediate">Immediate (show after each move)</option>
          <option value="on_demand">On demand (click to reveal)</option>
          <option value="post_game">Post-game only (review only)</option>
        </select>
      </label>
    </div>

    {#if llmStore.status !== "disabled"}
      <div class="mb-6 rounded p-3" style="background-color: var(--surface-input); opacity: 0.85;">
        <label class="mb-2 block text-sm font-medium" style="color: var(--text-on-card);"
          >AI Coach Model</label
        >
        {#if llmStore.status === "ready"}
          <div class="flex items-center gap-2 text-sm" style="color: var(--success);">
            <span class="inline-block h-2 w-2 rounded-full" style="background-color: var(--success);"
            ></span>
            Model loaded
          </div>
        {:else if llmStore.status === "loading"}
          <div class="space-y-2">
            <div class="text-sm" style="color: var(--accent-primary);">
              {llmStore.downloadProgress
                ? "Downloading model..."
                : "Loading model..."}
            </div>
            {#if llmStore.downloadProgress}
              <div class="h-2 w-full overflow-hidden rounded-full" style="background-color: var(--border-subtle);">
                <div
                  class="h-full rounded-full transition-all"
                  style="background-color: var(--accent-primary); width: {llmStore.downloadPercent}%"
                ></div>
              </div>
              <div class="text-xs" style="color: var(--text-secondary);">
                {llmStore.downloadPercent}% ({Math.round(
                  llmStore.downloadProgress.downloaded / 1048576,
                )} / {Math.round(llmStore.downloadProgress.total / 1048576)} MB)
              </div>
            {/if}
          </div>
        {:else}
          <div class="space-y-2">
            <p class="text-xs" style="color: var(--text-secondary);">
              Download Gemma 3 1B for enhanced coaching explanations (~2 GB).
            </p>
            {#if llmStore.error}
              <p class="text-xs" style="color: var(--danger);">{llmStore.error}</p>
            {/if}
            <button
              onclick={() => llmStore.startDownload()}
              class="btn btn-sm"
              style="background-color: var(--info); color: var(--text-heading);"
            >
              Download AI Coach Model
            </button>
          </div>
        {/if}
      </div>
    {/if}

    <div class="flex justify-end gap-2">
      <button onclick={onClose} class="btn btn-secondary">
        Cancel
      </button>
      <button onclick={handleSave} class="btn btn-primary">
        Save
      </button>
    </div>
  </div>
</div>
