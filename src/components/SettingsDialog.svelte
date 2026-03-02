<script lang="ts">
  import type { Settings } from "../lib/api/types";

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

  function handleSave() {
    onSave({
      ...settings,
      board_size: boardSize,
      show_coordinates: showCoordinates,
      ai_strength: aiStrength,
      sound_enabled: soundEnabled,
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

    <div class="mb-6">
      <label class="flex items-center gap-2 text-sm text-stone-400">
        <input type="checkbox" bind:checked={soundEnabled} class="rounded" />
        Sound effects
      </label>
    </div>

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
