<script lang="ts">
  import type { DifficultySuggestion } from "../lib/api/types";

  type Props = {
    suggestion: DifficultySuggestion;
    onAccept: () => void;
    onDismiss: () => void;
  };

  let { suggestion, onAccept, onDismiss }: Props = $props();
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
  <div class="w-72 rounded-lg bg-stone-800 p-5 shadow-xl text-center">
    <div class="mb-3 text-2xl">
      {suggestion.direction === "up" ? "🎯" : "📖"}
    </div>
    <p class="mb-4 text-sm text-stone-300">{suggestion.message}</p>
    <div class="flex justify-center gap-2">
      <button
        onclick={onDismiss}
        class="rounded bg-stone-700 px-4 py-1.5 text-sm text-stone-300 hover:bg-stone-600"
      >
        Not now
      </button>
      <button
        onclick={onAccept}
        class="rounded bg-amber-700 px-4 py-1.5 text-sm font-semibold text-white hover:bg-amber-600"
      >
        {suggestion.direction === "up" ? "Try harder" : "Easier opponent"}
      </button>
    </div>
  </div>
</div>
