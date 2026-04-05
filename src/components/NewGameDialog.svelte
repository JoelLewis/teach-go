<script lang="ts">
  import type { Settings, NewGameConfig } from "../lib/api/types";

  type Props = {
    settings: Settings;
    onClose: () => void;
    onStart: (config: NewGameConfig) => void;
  };

  let { settings, onClose, onStart }: Props = $props();

  let boardSize = $state(settings.board_size);
  let colorChoice = $state<"black" | "white" | "auto">("black");
  let aiStrength = $state(settings.ai_strength);

  function handleStart() {
    const playerColor: "black" | "white" =
      colorChoice === "auto"
        ? (Math.random() < 0.5 ? "black" : "white")
        : colorChoice;
    onStart({ boardSize, playerColor, aiStrength });
  }
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape') onClose(); }} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onClose}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="w-80 rounded-lg p-6 shadow-xl" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} style="background-color: var(--surface-card);">

    <h2 class="mb-4 text-lg font-semibold" style="color: var(--text-heading);">New Game</h2>

    <div class="mb-4">
      <label class="mb-1 block text-sm" style="color: var(--text-secondary);">Board Size</label>
      <select
        bind:value={boardSize}
        class="w-full rounded px-3 py-2"
        style="background-color: var(--surface-input); color: var(--text-heading);"
      >
        <option value={9}>9x9</option>
        <option value={13}>13x13</option>
        <option value={19}>19x19</option>
      </select>
    </div>

    <div class="mb-4">
      <label class="mb-2 block text-sm" style="color: var(--text-secondary);">Your Color</label>
      <div class="flex gap-2">
        {#each [
          { value: "black", label: "Black" },
          { value: "white", label: "White" },
          { value: "auto", label: "Auto" },
        ] as option}
          <button
            onclick={() => (colorChoice = option.value as "black" | "white" | "auto")}
            class="flex-1 rounded px-3 py-2 text-sm font-medium transition hover:opacity-90"
            style="{colorChoice === option.value
              ? `background-color: var(--btn-bg); color: var(--btn-text);`
              : `background-color: var(--surface-input); color: var(--text-on-card);`}"
          >
            {option.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="mb-6">
      <label class="mb-1 block text-sm" style="color: var(--text-secondary);">AI Strength</label>
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
    </div>

    <div class="flex justify-end gap-2">
      <button
        onclick={onClose}
        class="rounded px-4 py-2 text-sm hover:opacity-90"
        style="background-color: var(--surface-input); color: var(--text-heading);"
      >
        Cancel
      </button>
      <button
        onclick={handleStart}
        class="rounded px-4 py-2 text-sm hover:opacity-90"
        style="background-color: var(--btn-bg); color: var(--btn-text);"
      >
        Start Game
      </button>
    </div>
  </div>
</div>
