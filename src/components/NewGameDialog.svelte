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

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onClose}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="w-80 rounded-lg bg-stone-800 p-6 shadow-xl" role="dialog" aria-modal="true" tabindex="-1" onkeydown={(e) => { if (e.key === 'Escape') onClose(); }} onclick={(e) => e.stopPropagation()}>

    <h2 class="mb-4 text-lg font-semibold text-stone-100">New Game</h2>

    <div class="mb-4">
      <label class="mb-1 block text-sm text-stone-400">Board Size</label>
      <select
        bind:value={boardSize}
        class="w-full rounded bg-stone-700 px-3 py-2 text-stone-100"
      >
        <option value={9}>9×9</option>
        <option value={13}>13×13</option>
        <option value={19}>19×19</option>
      </select>
    </div>

    <div class="mb-4">
      <label class="mb-2 block text-sm text-stone-400">Your Color</label>
      <div class="flex gap-2">
        {#each [
          { value: "black", label: "Black" },
          { value: "white", label: "White" },
          { value: "auto", label: "Auto" },
        ] as option}
          <button
            onclick={() => (colorChoice = option.value as "black" | "white" | "auto")}
            class="flex-1 rounded px-3 py-2 text-sm font-medium transition {colorChoice ===
            option.value
              ? 'bg-amber-700 text-white'
              : 'bg-stone-700 text-stone-300 hover:bg-stone-600'}"
          >
            {option.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="mb-6">
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

    <div class="flex justify-end gap-2">
      <button
        onclick={onClose}
        class="rounded bg-stone-700 px-4 py-2 text-sm text-stone-100 hover:bg-stone-600"
      >
        Cancel
      </button>
      <button
        onclick={handleStart}
        class="rounded bg-amber-700 px-4 py-2 text-sm text-stone-100 hover:bg-amber-600"
      >
        Start Game
      </button>
    </div>
  </div>
</div>
