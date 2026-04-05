<script lang="ts">
  type Props = {
    currentMove: number;
    totalMoves: number;
    hasMistakes: boolean;
    onFirst: () => void;
    onPrev: () => void;
    onNext: () => void;
    onLast: () => void;
    onPrevMistake: () => void;
    onNextMistake: () => void;
  };

  let {
    currentMove,
    totalMoves,
    hasMistakes,
    onFirst,
    onPrev,
    onNext,
    onLast,
    onPrevMistake,
    onNextMistake,
  }: Props = $props();
</script>

<div class="flex flex-col gap-2">
  <div class="text-center text-sm" style="color: var(--text-secondary);">
    Move {currentMove} / {totalMoves}
  </div>

  <div class="flex items-center justify-center gap-1">
    <button
      onclick={onFirst}
      disabled={currentMove === 0}
      class="rounded px-2 py-1 text-sm hover:opacity-90 disabled:opacity-30"
      style="background-color: var(--surface-input); color: var(--text-on-card);"
      title="First move"
      aria-label="First move"
    >|&laquo;</button>
    <button
      onclick={onPrev}
      disabled={currentMove === 0}
      class="rounded px-3 py-1 text-sm hover:opacity-90 disabled:opacity-30"
      style="background-color: var(--surface-input); color: var(--text-on-card);"
      title="Previous move"
      aria-label="Previous move"
    >&lsaquo;</button>
    <button
      onclick={onNext}
      disabled={currentMove >= totalMoves}
      class="rounded px-3 py-1 text-sm hover:opacity-90 disabled:opacity-30"
      style="background-color: var(--surface-input); color: var(--text-on-card);"
      title="Next move"
      aria-label="Next move"
    >&rsaquo;</button>
    <button
      onclick={onLast}
      disabled={currentMove >= totalMoves}
      class="rounded px-2 py-1 text-sm hover:opacity-90 disabled:opacity-30"
      style="background-color: var(--surface-input); color: var(--text-on-card);"
      title="Last move"
      aria-label="Last move"
    >&raquo;|</button>
  </div>

  {#if hasMistakes}
    <div class="flex items-center justify-center gap-2">
      <button
        onclick={onPrevMistake}
        class="rounded px-2 py-1 text-xs hover:opacity-90"
        style="background-color: var(--severity-blunder); color: var(--severity-blunder-text);"
        title="Previous mistake"
        aria-label="Previous mistake"
      >&lsaquo; Mistake</button>
      <button
        onclick={onNextMistake}
        class="rounded px-2 py-1 text-xs hover:opacity-90"
        style="background-color: var(--severity-blunder); color: var(--severity-blunder-text);"
        title="Next mistake"
        aria-label="Next mistake"
      >Mistake &rsaquo;</button>
    </div>
  {/if}
</div>
