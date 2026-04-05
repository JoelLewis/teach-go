<script lang="ts">
  import type { DifficultySuggestion } from "../lib/api/types";

  type Props = {
    suggestion: DifficultySuggestion;
    onAccept: () => void;
    onDismiss: () => void;
  };

  let { suggestion, onAccept, onDismiss }: Props = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onDismiss}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="w-72 rounded-lg p-5 shadow-xl text-center" role="dialog" aria-modal="true" tabindex="-1" onkeydown={(e) => { if (e.key === 'Escape') onDismiss(); }} onclick={(e) => e.stopPropagation()} style="background-color: var(--surface-card);">
    <div class="mb-3 text-2xl">
      {suggestion.direction === "up" ? "\ud83c\udfaf" : "\ud83d\udcd6"}
    </div>
    <p class="mb-4 text-sm" style="color: var(--text-on-card);">{suggestion.message}</p>
    <div class="flex justify-center gap-2">
      <button
        onclick={onDismiss}
        class="rounded px-4 py-1.5 text-sm hover:opacity-90"
        style="background-color: var(--surface-input); color: var(--text-on-card);"
      >
        Not now
      </button>
      <button
        onclick={onAccept}
        class="rounded px-4 py-1.5 text-sm font-semibold hover:opacity-90"
        style="background-color: var(--btn-bg); color: var(--btn-text);"
      >
        {suggestion.direction === "up" ? "Try harder" : "Easier opponent"}
      </button>
    </div>
  </div>
</div>
