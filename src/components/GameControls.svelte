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

<div class="flex flex-wrap gap-1.5">
  <button onclick={onPass} {disabled} class="btn btn-secondary">
    Pass
  </button>
  <button onclick={onUndo} {disabled} class="btn btn-secondary">
    Undo
  </button>
  <button onclick={onResign} {disabled} class="btn btn-danger">
    Resign
  </button>
  <button onclick={onNewGame} class="btn btn-primary">
    New Game
  </button>
  <button onclick={onSave} class="btn btn-secondary">
    Save
  </button>
  <button onclick={onLoad} class="btn btn-secondary">
    Load
  </button>
</div>
