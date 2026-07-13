<script lang="ts">
  type Props = {
    onPass: () => void;
    onResign: () => void;
    onUndo: () => void;
    onNewGame: () => void;
    onSave: () => void;
    onLoad: () => void;
    disabled: boolean;
    resignLabel?: string;
  };

  let {
    onPass,
    onResign,
    onUndo,
    onNewGame,
    onSave,
    onLoad,
    disabled,
    resignLabel = "Resign",
  }: Props = $props();

  let confirmingResign = $state(false);
</script>

<div class="flex flex-wrap gap-1.5">
  <button onclick={onPass} {disabled} class="btn btn-secondary">
    Pass
  </button>
  <button onclick={onUndo} {disabled} class="btn btn-secondary">
    Undo
  </button>
  {#if confirmingResign}
    <span class="flex items-center gap-1 text-sm">Resign this game?</span>
    <button onclick={() => { confirmingResign = false; onResign(); }} {disabled} class="btn btn-danger">Yes</button>
    <button onclick={() => (confirmingResign = false)} class="btn btn-secondary">No</button>
  {:else}
    <button onclick={() => (confirmingResign = true)} {disabled} class="btn btn-danger">{resignLabel}</button>
  {/if}
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
