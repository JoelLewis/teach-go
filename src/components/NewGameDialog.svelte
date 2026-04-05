<script lang="ts">
  import { onMount } from "svelte";
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

  let dialogEl: HTMLDivElement;

  onMount(() => {
    const focusable = dialogEl?.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (!focusable?.length) return;
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
    return () => dialogEl.removeEventListener("keydown", handleKeydown);
  });

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
  <div bind:this={dialogEl} class="w-80 rounded-lg p-6 shadow-xl" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} style="background-color: var(--surface-card);">

    <h2 class="mb-4 text-lg font-semibold" style="color: var(--text-heading);">New Game</h2>

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
      <label class="mb-2 block text-sm" style="color: var(--text-secondary);">Your Color</label>
      <div class="flex gap-2">
        {#each [
          { value: "black", label: "Black" },
          { value: "white", label: "White" },
          { value: "auto", label: "Auto" },
        ] as option}
          <button
            onclick={() => (colorChoice = option.value as "black" | "white" | "auto")}
            class="btn btn-sm flex-1 {colorChoice === option.value ? 'btn-primary' : 'btn-secondary'}"
          >
            {option.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="mb-6">
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

    <div class="flex justify-end gap-2">
      <button onclick={onClose} class="btn btn-secondary">
        Cancel
      </button>
      <button onclick={handleStart} class="btn btn-primary">
        Start Game
      </button>
    </div>
  </div>
</div>
