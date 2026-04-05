<script lang="ts">
  type Props = {
    onPass: () => void;
    onResign: () => void;
    onUndo: () => void;
    onNewGame: () => void;
    onSave: () => void;
    onLoad: () => void;
    disabled: boolean;
  };

  let { onPass, onResign, onUndo, onNewGame, onSave, onLoad, disabled }: Props =
    $props();
</script>

<div class="flex flex-wrap gap-2">
  <button
    onclick={onPass}
    {disabled}
    class="rounded px-4 py-2.5 text-sm font-medium hover:opacity-90 disabled:opacity-50"
    style="background-color: var(--surface-input); color: var(--text-heading);"
  >
    Pass
  </button>
  <button
    onclick={onUndo}
    {disabled}
    class="rounded px-4 py-2.5 text-sm font-medium hover:opacity-90 disabled:opacity-50"
    style="background-color: var(--surface-input); color: var(--text-heading);"
  >
    Undo
  </button>
  <button
    onclick={onResign}
    {disabled}
    class="rounded px-4 py-2.5 text-sm font-medium hover:opacity-90 disabled:opacity-50"
    style="background-color: var(--danger); color: var(--text-heading);"
  >
    Resign
  </button>
  <button
    onclick={onNewGame}
    class="rounded px-4 py-2.5 text-sm font-medium hover:opacity-90"
    style="background-color: var(--btn-bg); color: var(--btn-text);"
  >
    New Game
  </button>
  <button
    onclick={onSave}
    class="rounded px-4 py-2.5 text-sm font-medium hover:opacity-90"
    style="background-color: var(--surface-input); color: var(--text-heading);"
  >
    Save
  </button>
  <button
    onclick={onLoad}
    class="rounded px-4 py-2.5 text-sm font-medium hover:opacity-90"
    style="background-color: var(--surface-input); color: var(--text-heading);"
  >
    Load
  </button>
</div>
