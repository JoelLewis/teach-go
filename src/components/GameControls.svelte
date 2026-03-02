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
    class="rounded bg-stone-700 px-4 py-2 text-sm font-medium text-stone-100 hover:bg-stone-600 disabled:opacity-50"
  >
    Pass
  </button>
  <button
    onclick={onUndo}
    {disabled}
    class="rounded bg-stone-700 px-4 py-2 text-sm font-medium text-stone-100 hover:bg-stone-600 disabled:opacity-50"
  >
    Undo
  </button>
  <button
    onclick={onResign}
    {disabled}
    class="rounded bg-red-900 px-4 py-2 text-sm font-medium text-stone-100 hover:bg-red-800 disabled:opacity-50"
  >
    Resign
  </button>
  <button
    onclick={onNewGame}
    class="rounded bg-amber-700 px-4 py-2 text-sm font-medium text-stone-100 hover:bg-amber-600"
  >
    New Game
  </button>
  <button
    onclick={onSave}
    class="rounded bg-stone-700 px-4 py-2 text-sm font-medium text-stone-100 hover:bg-stone-600"
  >
    Save
  </button>
  <button
    onclick={onLoad}
    class="rounded bg-stone-700 px-4 py-2 text-sm font-medium text-stone-100 hover:bg-stone-600"
  >
    Load
  </button>
</div>
